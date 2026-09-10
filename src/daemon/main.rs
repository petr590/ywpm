use nix::errno::Errno;
use nix::poll::{self, PollFd, PollFlags};
use nix::sys::signal::{SigSet, Signal};
use nix::sys::signalfd::{SfdFlags, SignalFd};
use sd_notify::NotifyState;
use std::env;
use std::error::Error;
use std::io::{self, BufReader, BufWriter};
use std::os::fd::AsFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::process::exit;

use ywpm::daemon::arg_parsing;
use ywpm::daemon::backend;
use ywpm::daemon::service::{config, wallpaper};
use ywpm::daemon::state::State;
use ywpm::{reader, util, writer};

fn main() -> Result<(), Box<dyn Error>> {
    let mut state = config::read_or_create_empty(util::get_config_path())?;

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        perform_initial_action(&args, &mut state);
    }

    main_loop(state)
}

fn perform_initial_action(args: &Vec<String>, state: &mut State) {
    match arg_parsing::parse_args(&args) {
        Ok(action) => match action.perform_and_update_config(&args[0], state) {
            Ok(success) => println!("{success}"),
            Err(err) => {
                eprintln!("{}", err.message());
                exit(1);
            }
        },

        Err(err) => {
            eprintln!("{}", err.message());
            exit(1);
        }
    }
}

fn main_loop(mut state: State) -> Result<(), Box<dyn Error>> {
    let socket_path = util::get_socket_path();

    println!(
        "Connecting to socket: '{}'...",
        socket_path.to_str().unwrap_or_default()
    );

    let listener = UnixListener::bind(&socket_path)?;
    listener.set_nonblocking(true)?;

    let signal_fd = get_signal_fd()?;

    println!("Daemon started. Waiting for events...");
    let _ = sd_notify::notify(&[NotifyState::Ready]);

    loop {
        let mut poll_fds = [
            PollFd::new(listener.as_fd(), PollFlags::POLLIN),
            PollFd::new(signal_fd.as_fd(), PollFlags::POLLIN),
        ];

        match poll::poll(&mut poll_fds, state.get_timeout()) {
            Ok(0) => {
                println!("\nTimeout reached! Updating wallpaper...");
                wallpaper::set_random(&mut state)?;
            }

            Ok(_) => {
                if let Some(revents) = poll_fds[0].revents()
                    && revents.contains(PollFlags::POLLIN)
                {
                    match listener.accept() {
                        Ok((stream, _)) => handle_client(stream, &mut state)?,
                        Err(err) if err.kind() == io::ErrorKind::WouldBlock => {}
                        Err(err) => eprintln!("Accept error: {err}"),
                    }
                }

                if let Some(revents) = poll_fds[1].revents()
                    && revents.contains(PollFlags::POLLIN)
                {
                    println!("\nReceived shutdown signal! Exiting gracefully...");

                    let _ = std::fs::remove_file(&socket_path);

                    state.clear_expired_peroids();

                    if let Err(err) = config::write(&state, util::get_config_path()) {
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

    match arg_parsing::parse_args(&args) {
        Ok(action) => match action.perform_and_update_config(&args[0], state) {
            Ok(success) => writer::write_ok(&mut writer, success)?,
            Err(error) => writer::write_error(&mut writer, error.message())?,
        },

        Err(error) => writer::write_error(&mut writer, error.message())?,
    }

    Ok(())
}
