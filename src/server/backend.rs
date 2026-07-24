use std::cell::RefCell;
use std::error::Error;
use std::process::{Child, Command};

use crate::server::wallpaper_node::WallpaperNode;


thread_local! {
    static CHILD: RefCell<Option<Child>> = RefCell::new(Option::None);
}

pub fn run_backend(node: &WallpaperNode) -> Result<(), Box<dyn Error>> {
    stop_backend();

    CHILD.with_borrow_mut(|opt| -> Result<(), Box<dyn Error>> {
        let child = Command::new("mpvpaper")
            .arg("-o")
            .arg("
                --loop=inf --image-display-duration=inf --ao=null
                --vo=gpu --hwdec=auto --video-sync=display-resample
                --scale=spline36 --cscale=spline36
                --stop-screensaver=no --osc=no --config=no
                video-unscaled=no panscan=1.0 video-align-y=-1
            ")
            .arg("ALL")
            .arg(node.path())
            .spawn()?;

        *opt = Option::Some(child);

        println!("Process 'mpvpaper' started");
        
        Ok(())
    })?;

    Ok(())
}


pub fn stop_backend() {
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