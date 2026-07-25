//! Capture guest frames into world sensory state (not an external viewer).

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
        let serial_t = serial.clone();
        let frames_t = frames.clone();

        thread::spawn(move || {
            let mut ticks = 0u32;
            loop {
                let ser = serial_t.lock().ok().and_then(|g| g.clone());
                match capture_once(ser.as_deref()) {
                    Ok(frame) => {
                        if let Ok(mut g) = frames_t.lock() {
                            *g = Some(frame);
                        }
                    }
                    Err(e) => {
                        ticks = ticks.wrapping_add(1);
                        if ticks % 30 == 1 {
                            println!("[Framebuffer] {e}");
                        }
                    }
                }
                thread::sleep(Duration::from_millis(100));
            }
        });

        app.insert_resource(FramebufferState {
            inner: frames,
            serial,
        })
        .init_resource::<crate::virtual_io::VirtualDisplay>()
        .init_resource::<crate::virtual_io::VirtualInput>()
        .add_systems(Update, sync_serial);

        println!("[VirtualIO] World-owned display + input active");
    }
}

fn sync_serial(
    runtime: Res<crate::android_runtime::AndroidRuntime>,
    fb: Res<FramebufferState>,
) {
    if let Ok(mut g) = fb.serial.lock() {
        *g = runtime.serial.clone();
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
        return Err("waiting for inhabitant framebuffer".into());
    }
    let img = ImageReader::new(Cursor::new(output.stdout))
        .with_guessed_format()
        .map_err(|e| e.to_string())?
        .decode()
        .map_err(|e| e.to_string())?
        .to_rgba8();
    Ok(FramePixels {
        width: img.width(),
        height: img.height(),
        rgba: img.into_raw(),
    })
}
