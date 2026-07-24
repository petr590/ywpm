use std::io::{self, BufReader, BufWriter};
use std::error::Error;
use std::os::fd::AsFd;
use std::os::unix::net::{UnixListener, UnixStream};
use nix::sys::signal::{Signal, SigSet};
use nix::sys::signalfd::{SignalFd, SfdFlags};
use nix::poll::{PollFd, PollFlags, PollTimeout, poll};

use ywpm::server::backend::stop_backend;
use ywpm::server::state::State;
use ywpm::server::args_parser::parse_args;
use ywpm::reader::read_string_vec;
use ywpm::writer::write_response;
use ywpm::util::{get_socket_path, get_config_path};

fn main() -> Result<(), Box<dyn Error>> {
    let mut state = State::read_or_create_empty(get_config_path())?;
    state.start_backend()?;
    main_loop(state)
}


fn main_loop(mut state: State) -> Result<(), Box<dyn Error>> {
    let socket_path = get_socket_path();

    let listener = UnixListener::bind(&socket_path)?;
    listener.set_nonblocking(true)?;

    let signal_fd = get_signal_fd()?;

    println!("Daemon started. Waiting for events...");

    let mut timeout_ms: i32 = -1; 

    loop {

        let mut poll_fds = [
            PollFd::new(listener.as_fd(), PollFlags::POLLIN),
            PollFd::new(signal_fd.as_fd(), PollFlags::POLLIN),
        ];

        match poll(&mut poll_fds, PollTimeout::try_from(timeout_ms)?) {
            Ok(0) => {
                println!("Timeout reached! Changing wallpaper...");
                
                timeout_ms = -1; // TODO
            }

            Ok(_) => {
                if let Some(revents) = poll_fds[0].revents() {
                    if revents.contains(PollFlags::POLLIN) {
                        match listener.accept() {
                            Ok((stream, _)) => handle_client(stream, &mut state)?,
                            Err(err) if err.kind() == io::ErrorKind::WouldBlock => (),
                            Err(err) => eprintln!("Accept error: {err}"),
                        }
                    }
                }

                if let Some(revents) = poll_fds[1].revents() {
                    if revents.contains(PollFlags::POLLIN) {
                        println!("\nReceived shutdown signal! Exiting gracefully...");

                        let _ = std::fs::remove_file(&socket_path);

                        if let Err(err) = state.write_to(get_config_path()) {
                            eprintln!("Error while writing config: {}", err.to_string());
                        }
                        
                        stop_backend();
                        break; 
                    }
                }
            }

            Err(nix::errno::Errno::EINTR) => {
                println!("poll returned EINTR. Retrying...");
                continue;
            }

            Err(errno) => {
                eprintln!("Poll error: {errno}");
                return Err(Box::new(io::Error::new(io::ErrorKind::Other, errno)));
            }
        }
    }

    println!("Daemon stopped");
    Ok(())
}


fn get_signal_fd() -> Result<SignalFd, Box<dyn Error>> {
    let mut mask = SigSet::empty();
    mask.add(Signal::SIGINT);
    mask.add(Signal::SIGTERM);
    mask.thread_block()?;

    Ok(SignalFd::with_flags(&mask, SfdFlags::SFD_NONBLOCK)?)
}


fn handle_client(mut stream: UnixStream, state: &mut State) -> Result<(), Box<dyn Error>> {
    let args = read_string_vec(&mut BufReader::new(&mut stream))?;

    let mut writer = BufWriter::new(&mut stream);
    
    match parse_args(&args) {
        Err(err) => write_response(&mut writer, false, err.message())?,

        Ok(action) => {
            match action.perform(args[0].as_str(), state) {
                Ok(message) => write_response(&mut writer, true, &message)?,
                Err(err)    => write_response(&mut writer, false, err.message())?,
            }
        }
    }

    Ok(())
}