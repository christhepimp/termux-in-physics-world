//! Capture real Android framebuffer into CPU buffers for the physics-world screen.
//!
//! Primary path: `adb exec-out screencap -p` (PNG). Works without scrcpy.
//! Capture runs on a background thread so the physics loop stays responsive.

use std::io::Cursor;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use bevy::prelude::*;
use image::ImageReader;

#[derive(Clone)]
pub struct FramePixels {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

#[derive(Resource, Clone)]
pub struct FramebufferState {
    inner: Arc<Mutex<Option<FramePixels>>>,
    pub serial: Arc<Mutex<Option<String>>>,
}

impl FramebufferState {
    pub fn latest_frame(&self) -> Option<FramePixels> {
        self.inner.lock().ok().and_then(|g| g.clone())
    }
}

pub struct FramebufferPlugin;

impl Plugin for FramebufferPlugin {
    fn build(&self, app: &mut App) {
        let serial = Arc::new(Mutex::new(None::<String>));
        let frames = Arc::new(Mutex::new(None::<FramePixels>));

        let serial_thread = serial.clone();
        let frames_thread = frames.clone();

        thread::spawn(move || {
            println!("[Framebuffer] Capture thread started (adb screencap)");
            loop {
                let serial_opt = serial_thread.lock().ok().and_then(|g| g.clone());
                match capture_once(serial_opt.as_deref()) {
                    Ok(frame) => {
                        if let Ok(mut g) = frames_thread.lock() {
                            *g = Some(frame);
                        }
                    }
                    Err(e) => {
                        // Avoid spam: occasional log
                        static mut TICK: u32 = 0;
                        unsafe {
                            TICK = TICK.wrapping_add(1);
                            if TICK % 20 == 1 {
                                println!("[Framebuffer] capture: {e}");
                            }
                        }
                    }
                }
                thread::sleep(Duration::from_millis(100)); // ~10 FPS foundation
            }
        });

        app.insert_resource(FramebufferState {
            inner: frames,
            serial,
        })
        .add_systems(Update, sync_serial_from_bridge);
    }
}

fn sync_serial_from_bridge(
    bridge: Res<crate::android_bridge::AndroidBridge>,
    fb: Res<FramebufferState>,
) {
    if let Ok(mut g) = fb.serial.lock() {
        *g = bridge.serial.clone();
    }
}

fn capture_once(serial: Option<&str>) -> Result<FramePixels, String> {
    let mut cmd = Command::new("adb");
    if let Some(s) = serial {
        cmd.arg("-s").arg(s);
    }
    cmd.args(["exec-out", "screencap", "-p"]);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::null());

    let output = cmd.output().map_err(|e| format!("adb: {e}"))?;
    if !output.status.success() || output.stdout.is_empty() {
        return Err("screencap failed or empty (is a device online?)".into());
    }

    let img = ImageReader::new(Cursor::new(output.stdout))
        .with_guessed_format()
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?
        .to_rgba8();

    let width = img.width();
    let height = img.height();
    Ok(FramePixels {
        width,
        height,
        rgba: img.into_raw(),
    })
}
