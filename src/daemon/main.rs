use clap::{CommandFactory, Parser};
use clap::error::ErrorKind;
use nix::errno::Errno;
use nix::poll::{self, PollFd, PollFlags};
use nix::sys::signal::{SigSet, Signal};
use nix::sys::signalfd::{SfdFlags, SignalFd};
use sd_notify::NotifyState;
use ywpm::daemon::arg_parsing::Cli;
use std::env;
use std::error::Error;
use std::io::{self, BufReader, BufWriter};
use std::os::fd::AsFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::process::exit;

use ywpm::daemon::action::ActionPerformError;
use ywpm::daemon::backend;
use ywpm::daemon::service::{config, wallpaper};
use ywpm::daemon::state::State;
use ywpm::{reader, util, writer};

fn main() -> Result<(), Box<dyn Error>> {
    let mut state = config::read_or_create_empty(util::get_config_path())?;

    let socket = if env::args().count() > 1 {
        perform_initial_action_or_exit(&mut state)
    } else {
        None
    };

    let socket = socket.unwrap_or_else(util::get_socket_path);

    main_loop(state, socket)
}

fn perform_initial_action_or_exit(state: &mut State) -> Option<String> {

    match perform_initial_action(state) {
        Ok(socket) => return socket,

        Err(ref err) if let Some(err) = err.downcast_ref::<clap::Error>() => {
            match err.kind() {
                ErrorKind::DisplayHelp |
                ErrorKind::DisplayVersion => {
                    println!("{}", err.render());
                    exit(0);
                }

                _ => {
                    eprintln!("{}", err.render());
                    exit(1);
                }
            }
        }

        Err(err) => {
            eprintln!("{}", err.to_string());
            exit(1);
        }
    }
}

fn perform_initial_action(state: &mut State) -> Result<Option<String>, Box<dyn Error>> {
    let mut cli = Cli::try_parse()?;

    let socket = cli.socket().clone();

    if cli.subcommand().is_some() {
        cli.canonicalize_paths(
            env::current_dir()?
                .to_str().unwrap_or_default()
        )?;

        cli.perform_and_update_config(state)?;
    }

    Ok(socket)
}


fn main_loop(mut state: State, socket_path: impl AsRef<Path>) -> Result<(), Box<dyn Error>> {
    let socket_path = socket_path.as_ref();

    println!(
        "Connecting to socket: '{}'...",
        socket_path.to_string_lossy()
    );

    let listener = UnixListener::bind(socket_path)?;
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

                    let _ = std::fs::remove_file(socket_path);

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
    let mut buf_reader = BufReader::new(&mut stream);

    let cwd = reader::read_string(&mut buf_reader)?;
    let args = reader::read_string_vec(&mut buf_reader)?;

    let mut writer = BufWriter::new(&mut stream);

    match Cli::try_parse_from(&args) {
        Ok(cli) if cli.subcommand().is_none() => {
            writer::write_error(&mut writer, &Cli::command().render_long_help().to_string())?;
        }

        Ok(mut cli) => {
            let result = cli.canonicalize_paths(&cwd)
                .map_err(|err| ActionPerformError::new(err.message()))
                .and_then(|()| cli.perform_and_update_config(state));

            match result {
                Ok(success) => writer::write_ok(&mut writer, success)?,
                Err(error) => writer::write_error(&mut writer, error.message())?,
            }
        },

        Err(error) => {
            writer::write_error(&mut writer, &error.render().to_string())?;
        },
    }

    Ok(())
}