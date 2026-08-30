use std::env;
use std::io::{self, BufReader, BufWriter};
use std::error::Error;
use std::os::fd::AsFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::process::exit;
use nix::errno::Errno;
use nix::sys::signal::{Signal, SigSet};
use nix::sys::signalfd::{SignalFd, SfdFlags};
use nix::poll::{self, PollFd, PollFlags};

use ywpm::daemon::arg_parsing::arg_parser;
use ywpm::daemon::backend;
use ywpm::daemon::state::State;
use ywpm::reader;
use ywpm::writer;
use ywpm::util;

fn main() -> Result<(), Box<dyn Error>> {
    let mut state = State::read_or_create_empty(util::get_config_path())?;

    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        state.restore_wallpaper()?;
    } else {
        perform_action(&args, &mut state);
    }

    main_loop(state)
}


fn perform_action(args: &Vec<String>, state: &mut State) {
    match arg_parser::parse_args(&args) {
        Ok(action) => {
            match action.perform(&args[0], state) {
                Ok(message) => println!("{}", message),
                Err(err) => {
                    eprintln!("{}", err.message());
                    exit(1);
                },
            }
        },

        Err(err) => {
            eprintln!("{}", err.message());
            exit(1);
        },
    }
}


fn main_loop(mut state: State) -> Result<(), Box<dyn Error>> {
    let socket_path = util::get_socket_path();

    let listener = UnixListener::bind(&socket_path)?;
    listener.set_nonblocking(true)?;

    let signal_fd = get_signal_fd()?;

    println!("Daemon started. Waiting for events...");

    loop {
        let mut poll_fds = [
            PollFd::new(listener.as_fd(), PollFlags::POLLIN),
            PollFd::new(signal_fd.as_fd(), PollFlags::POLLIN),
        ];

        match poll::poll(&mut poll_fds, state.get_timeout()) {
            Ok(0) => {
                println!("\nTimeout reached! Updating wallpaper...");
                state.set_random_wallpaper()?;
            }

            Ok(_) => {
                if let Some(revents) = poll_fds[0].revents() &&
                    revents.contains(PollFlags::POLLIN) {
                    
                    match listener.accept() {
                        Ok((stream, _)) => handle_client(stream, &mut state)?,
                        Err(err) if err.kind() == io::ErrorKind::WouldBlock => {},
                        Err(err) => eprintln!("Accept error: {err}"),
                    }
                }

                if let Some(revents) = poll_fds[1].revents() &&
                    revents.contains(PollFlags::POLLIN) {

                    println!("\nReceived shutdown signal! Exiting gracefully...");

                    let _ = std::fs::remove_file(&socket_path);

                    state.clear_expired_peroids();

                    if let Err(err) = state.write_to(util::get_config_path()) {
                        eprintln!("Error while writing config: {}", err.to_string());
                    }
                    
                    backend::stop();
                    break; 
                }
            }

            Err(nix::errno::Errno::EINTR) => {
                println!("\npoll returned EINTR. Retrying...");
                continue;
            }

            Err(errno) => {
                eprintln!("\nPoll error: {errno}");
                return Err(Box::new(io::Error::new(io::ErrorKind::Other, errno)));
            }
        }
    }

    println!("Daemon stopped");
    Ok(())
}


fn get_signal_fd() -> Result<SignalFd, Errno> {
    let mut mask = SigSet::empty();
    mask.add(Signal::SIGINT);
    mask.add(Signal::SIGTERM);
    mask.thread_block()?;

    Ok(SignalFd::with_flags(&mask, SfdFlags::SFD_NONBLOCK)?)
}


fn handle_client(mut stream: UnixStream, state: &mut State) -> Result<(), Box<dyn Error>> {
    let args = reader::read_string_vec(&mut BufReader::new(&mut stream))?;

    let mut writer = BufWriter::new(&mut stream);
    
    match arg_parser::parse_args(&args) {
        Ok(action) => {
            match action.perform(&args[0], state) {
                Ok(message) => writer::write_response(&mut writer, true, &message)?,
                Err(err)    => writer::write_response(&mut writer, false, err.message())?,
            }
        },

        Err(err) => writer::write_response(&mut writer, false, err.message())?,
    }

    Ok(())
}
