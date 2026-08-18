use std::cell::RefCell;
use std::error::Error;
use std::path::Path;
use std::process::{Child, Command};

use indoc::indoc;

use crate::daemon::wallpaper::Wallpaper;


thread_local! {
    static CHILD: RefCell<Option<Child>> = RefCell::new(None);
}

pub fn run(wallpaper: &Wallpaper) -> Result<(), Box<dyn Error>> {
    assert!(Path::new(wallpaper.path()).is_file());

    stop();

    CHILD.with_borrow_mut(|opt| -> Result<(), Box<dyn Error>> {
        println!("Wallpaper: '{}'", wallpaper.path());

        let child = Command::new("mpvpaper")
            .arg("-o")
            .arg(indoc! {"
                --loop=inf --image-display-duration=inf --ao=null
                --vo=gpu --hwdec=auto --video-sync=display-resample
                --scale=spline36 --cscale=spline36
                --stop-screensaver=no --osc=no --config=no
                video-unscaled=no panscan=1.0 video-align-y=-1
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
                Err(err) => eprintln!("Error while killing 'mpvpaper': {err}")
            }

            match child.wait() {
                Ok(status) => {
                    if !status.success() {
                        eprintln!("Process 'mpvpaper' returned status: {status}");
                    }
                }

                Err(err) => eprintln!("Error while waiting for 'mpvpaper': {err}")
            }

            println!("Process 'mpvpaper' stopped");
        }
    });
}