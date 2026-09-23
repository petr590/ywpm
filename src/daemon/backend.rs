use std::cell::RefCell;
use std::error::Error;
use std::path::Path;
use std::process::{Child, Command};

use indoc::formatdoc;

use crate::state::{Alignment, FitMode, Wallpaper};

thread_local! {
    static CHILD: RefCell<Option<Child>> = RefCell::new(None);
}

pub(crate) fn is_running() -> bool {
    CHILD.with_borrow(|child| child.is_some())
}

pub(crate) fn run(wallpaper: &Wallpaper) -> Result<(), Box<dyn Error>> {
    stop();

    assert!(Path::new(wallpaper.path()).is_file());

    CHILD.with_borrow_mut(|opt| -> Result<(), Box<dyn Error>> {
        let mode = wallpaper.mode();

        println!("Wallpaper: '{}'", wallpaper.path());
        println!("Mode: '{mode}'");

        let mode_opt = match mode.fit_mode {
            FitMode::Cover   => "panscan=1",
            FitMode::Contain => "panscan=0",
            FitMode::Stretch => "video-aspect=0",
        };

        let (align_x, align_y) = match mode.alignment {
            Alignment::Center => ( "0",  "0"),
            Alignment::Top    => ( "0", "-1"),
            Alignment::Bottom => ( "0",  "1"),
            Alignment::Left   => ("-1",  "0"),
            Alignment::Right  => ( "1",  "0"),
        };

        let child = Command::new("mpvpaper")
            .arg("-o") // spline36
            .arg(formatdoc! {"
                --loop=inf --image-display-duration=inf --ao=null
                --vo=gpu --hwdec=auto --video-sync=display-resample
                --scale=bilinear --cscale=bilinear --dscale=bilinear
                --stop-screensaver=no --osc=no --config=no --load-scripts=no
                term-status-msg= video-unscaled=no
                {mode_opt} video-align-x={align_x} video-align-y={align_y}
            "})
            .arg("ALL")
            .arg(wallpaper.path())
            .spawn()?;

        *opt = Some(child);

        println!("Process 'mpvpaper' started");

        Ok(())
    })?;

    Ok(())
}

pub fn stop() {
    CHILD.with_borrow_mut(|opt| {
        if let Some(child) = opt {
            match child.kill() {
                Ok(()) => {}
                Err(err) => eprintln!("Error while killing 'mpvpaper': {err}"),
            }

            match child.wait() {
                Ok(status) => {
                    if !status.success() {
                        eprintln!("Process 'mpvpaper' returned status: {status}");
                    }
                }

                Err(err) => eprintln!("Error while waiting for 'mpvpaper': {err}"),
            }

            println!("Process 'mpvpaper' stopped");
        }
    });
}
