use clap::Parser;
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
use std::path::Path;
use std::process::exit;

use ywpm::cli::Cli;
use ywpm::core::ActionPerformError;
use ywpm::daemon::{backend, service};
use ywpm::daemon::service::{config, wallpaper};
use ywpm::state::State;
use ywpm::util::{self, ReadWriteError};

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
        Ok(socket) => socket,

        Err(err) => {
            eprintln!("{}", err.to_string());
            exit(1);
        }
    }
}

fn perform_initial_action(state: &mut State) -> Result<Option<String>, Box<dyn Error>> {
    let mut cli = Cli::parse();

    cli.canonicalize_paths(
        env::current_dir()?
            .to_str().unwrap_or_default()
    )?;

    let socket_path = cli.socket_path().clone();
    service::perform_action_and_update_config(cli, state, util::get_terminal_width())?;
    Ok(socket_path)
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
                let listener_poll_fd = &poll_fds[0];
                let signal_poll_fd = &poll_fds[1];

                if listener_poll_fd.revents().is_some_and(|revents| revents.contains(PollFlags::POLLIN)) {
                    accept_client(&listener, &mut state);
                }

                if signal_poll_fd.revents().is_some_and(|revents| revents.contains(PollFlags::POLLIN)) {
                    shutdown(socket_path, &mut state);
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

fn accept_client(listener: &UnixListener, state: &mut State) {
    match listener.accept() {
        Ok((stream, _)) => {
            match handle_client(stream, state) {
                Ok(()) => {}
                Err(rw_err) => eprintln!("Client handle error: {rw_err}"),
            }
        }

        Err(io_err) if io_err.kind() == io::ErrorKind::WouldBlock => {}
        Err(io_err) => eprintln!("Accept I/O error: {io_err}"),
    }
}

fn handle_client(mut stream: UnixStream, state: &mut State) -> Result<(), ReadWriteError> {
    let mut reader = BufReader::new(&mut stream);

    if !util::read_bool(&mut reader)? {
        return Ok(());
    }

    let cwd = util::read_string(&mut reader)?;
    let args = util::read_string_vec(&mut reader)?;
    let term_width = util::read_u16(&mut reader)?;

    let mut writer = BufWriter::new(&mut stream);

    match Cli::try_parse_from(&args) {
        Ok(mut cli) => {
            if cli.subcommand().changes_state() {
                println!("Command: {}", args.join(" "))
            }

            let result = cli.canonicalize_paths(&cwd)
                .map_err(|err| ActionPerformError::new(err.message()))
                .and_then(|()| service::perform_action_and_update_config(cli, state, term_width));

            match result {
                Ok(success) => util::write_ok(&mut writer, success)?,
                Err(error) => util::write_error(&mut writer, error.message())?,
            }
        },

        Err(error) => {
            util::write_error(&mut writer, &error.render().to_string())?;
        },
    }

    Ok(())
}

fn shutdown(socket_path: &Path, state: &mut State) {
    println!("\nReceived shutdown signal! Exiting gracefully...");

    let _ = std::fs::remove_file(socket_path);

    if let Err(err) = config::write(state.normalize(), util::get_config_path()) {
        eprintln!("Error while writing config: {}", err.to_string());
    }

    backend::stop();
}