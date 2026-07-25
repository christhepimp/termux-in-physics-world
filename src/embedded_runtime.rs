//! Embedded OE runtime service — runs inside the physics engine process.
//!
//! Desktop: in-process threads + optional world-owned headless guest backend.
//! Android: in-process native path (see android_native).

use std::io::Cursor;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use bevy::prelude::*;
use image::ImageReader;

use crate::android_native;
use crate::shared_buffer::{FramePixels, InputCmd, InputQueue, SharedFrameBuffer};

#[derive(Resource)]
pub struct EmbeddedRuntime {
    pub serial: Arc<Mutex<Option<String>>>,
    pub connected: Arc<Mutex<bool>>,
    pub termux_ready: Arc<Mutex<bool>>,
    child: Arc<Mutex<Option<Child>>>,
    pub in_process: bool,
}

impl Default for EmbeddedRuntime {
    fn default() -> Self {
        Self {
            serial: Arc::new(Mutex::new(None)),
            connected: Arc::new(Mutex::new(false)),
            termux_ready: Arc::new(Mutex::new(false)),
            child: Arc::new(Mutex::new(None)),
            in_process: true,
        }
    }
}

impl EmbeddedRuntime {
    pub fn is_connected(&self) -> bool {
        *self.connected.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn request_launch_termux(&self) {
        #[cfg(target_os = "android")]
        {
            android_native::launch_termux_in_process();
            *self.termux_ready.lock().unwrap() = true;
            return;
        }
        #[cfg(not(target_os = "android"))]
        {
            let serial = self.serial.lock().unwrap().clone();
            let mut cmd = Command::new("adb");
            if let Some(s) = serial {
                cmd.arg("-s").arg(s);
            }
            let _ = cmd
                .args(["shell", "am", "start", "-n", "com.termux/.HomeActivity"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            println!("[EmbeddedRuntime] Termux launch (in-process control path)");
        }
    }

    pub fn shutdown(&self) {
        if let Ok(mut g) = self.child.lock() {
            if let Some(mut c) = g.take() {
                let _ = c.kill();
                println!("[EmbeddedRuntime] Stopped world-owned guest process");
            }
        }
    }
}

pub struct EmbeddedRuntimePlugin;

impl Plugin for EmbeddedRuntimePlugin {
    fn build(&self, app: &mut App) {
        let frames = SharedFrameBuffer::default();
        let input_q = InputQueue::default();
        let runtime = EmbeddedRuntime::default();

        let frame_handle = frames.handle();
        let input_handle = input_q.handle();
        let serial_h = runtime.serial.clone();
        let connected_h = runtime.connected.clone();
        let child_h = runtime.child.clone();

        // --- in-process runtime service thread ---
        thread::Builder::new()
            .name("embedded-oe-runtime".into())
            .spawn(move || {
                println!("[EmbeddedRuntime] Service thread started inside physics process");

                #[cfg(not(target_os = "android"))]
                desktop_bootstrap(&serial_h, &connected_h, &child_h);

                #[cfg(target_os = "android")]
                {
                    android_native::init_in_process();
                    *connected_h.lock().unwrap() = true;
                }

                loop {
                    // Pump input queue → guest
                    for cmd in InputQueue::drain(&input_handle) {
                        dispatch_input(&serial_h, cmd);
                    }

                    // Capture frame → shared buffer (in-process)
                    match capture_frame(&serial_h) {
                        Ok(frame) => SharedFrameBuffer::publish(&frame_handle, frame),
                        Err(_) => {}
                    }

                    thread::sleep(Duration::from_millis(50));
                }
            })
            .expect("spawn embedded runtime thread");

        app.insert_resource(frames)
            .insert_resource(input_q)
            .insert_resource(runtime);

        println!("[EmbeddedRuntime] Compiled into physics engine process");
    }
}

#[cfg(not(target_os = "android"))]
fn desktop_bootstrap(
    serial_h: &Arc<Mutex<Option<String>>>,
    connected_h: &Arc<Mutex<bool>>,
    child_h: &Arc<Mutex<Option<Child>>>,
) {
    // Attach or spawn headless under this process tree
    if !refresh_adb(serial_h, connected_h) {
        if let Ok(avd) = std::env::var("VCE_AVD_NAME") {
            println!("[EmbeddedRuntime] Spawning headless AVD '{avd}' as child of physics process");
            match Command::new("emulator")
                .args([
                    "-avd",
                    &avd,
                    "-no-window",
                    "-no-audio",
                    "-no-boot-anim",
                    "-gpu",
                    "swiftshader_indirect",
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                Ok(child) => {
                    *child_h.lock().unwrap() = Some(child);
                    let _ = Command::new("adb").args(["wait-for-device"]).status();
                    let _ = refresh_adb(serial_h, connected_h);
                }
                Err(e) => println!("[EmbeddedRuntime] emulator spawn failed: {e}"),
            }
        } else {
            println!("[EmbeddedRuntime] No device. Set VCE_AVD_NAME or start headless emulator.");
        }
    }
    if *connected_h.lock().unwrap() {
        // Auto-launch Termux once
        let serial = serial_h.lock().unwrap().clone();
        let mut cmd = Command::new("adb");
        if let Some(s) = serial {
            cmd.arg("-s").arg(s);
        }
        let installed = cmd
            .args(["shell", "pm", "path", "com.termux"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("package:"))
            .unwrap_or(false);
        if installed {
            let serial = serial_h.lock().unwrap().clone();
            let mut cmd = Command::new("adb");
            if let Some(s) = serial {
                cmd.arg("-s").arg(s);
            }
            let _ = cmd
                .args(["shell", "am", "start", "-n", "com.termux/.HomeActivity"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            println!("[EmbeddedRuntime] Termux start requested");
        }
    }
}

#[cfg(not(target_os = "android"))]
fn refresh_adb(serial_h: &Arc<Mutex<Option<String>>>, connected_h: &Arc<Mutex<bool>>) -> bool {
    let output = Command::new("adb").arg("devices").output();
    let Ok(output) = output else {
        *connected_h.lock().unwrap() = false;
        return false;
    };
    for line in String::from_utf8_lossy(&output.stdout).lines().skip(1) {
        let mut p = line.split_whitespace();
        if let (Some(serial), Some("device")) = (p.next(), p.next()) {
            *serial_h.lock().unwrap() = Some(serial.to_string());
            *connected_h.lock().unwrap() = true;
            return true;
        }
    }
    *connected_h.lock().unwrap() = false;
    false
}

fn dispatch_input(serial_h: &Arc<Mutex<Option<String>>>, cmd: InputCmd) {
    #[cfg(target_os = "android")]
    {
        match cmd {
            InputCmd::Text(t) => android_native::inject_text(&t),
            InputCmd::Key(c) => android_native::inject_key(c),
        }
        return;
    }
    #[cfg(not(target_os = "android"))]
    {
        let serial = serial_h.lock().unwrap().clone();
        let mut adb = Command::new("adb");
        if let Some(s) = serial {
            adb.arg("-s").arg(s);
        }
        match cmd {
            InputCmd::Text(t) => {
                let esc = t.replace(' ', "%s");
                let _ = adb
                    .args(["shell", "input", "text", &esc])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
            InputCmd::Key(c) => {
                let _ = adb
                    .args(["shell", "input", "keyevent", &c.to_string()])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
        }
    }
}

fn capture_frame(serial_h: &Arc<Mutex<Option<String>>>) -> Result<FramePixels, String> {
    #[cfg(target_os = "android")]
    {
        return android_native::capture_frame_in_process();
    }
    #[cfg(not(target_os = "android"))]
    {
        let serial = serial_h.lock().unwrap().clone();
        let mut cmd = Command::new("adb");
        if let Some(s) = serial {
            cmd.arg("-s").arg(s);
        }
        cmd.args(["exec-out", "screencap", "-p"]);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::null());
        let output = cmd.output().map_err(|e| e.to_string())?;
        if !output.status.success() || output.stdout.is_empty() {
            return Err("no frame".into());
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
}
