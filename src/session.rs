//! Real OS session behind a PTY.
//!
//! Prefers a Termux environment when detected; otherwise launches a host
//! shell so the same physics-world integration works on desktop.

use std::io::{Read, Write};
use std::sync::{Arc, Mutex};
use std::thread;

use bevy::prelude::*;
use portable_pty::{CommandBuilder, NativePtySystem, PtySize, PtySystem};

/// Bytes received from the session (stdout/stderr via PTY).
#[derive(Resource, Default)]
pub struct SessionOutput {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl SessionOutput {
    pub fn drain(&mut self) -> Vec<u8> {
        let mut guard = self.buffer.lock().unwrap();
        std::mem::take(&mut *guard)
    }
}

/// Write side of the PTY + metadata.
#[derive(Resource)]
pub struct SessionBridge {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    pub kind: SessionKind,
    pub label: String,
}

impl SessionBridge {
    pub fn write_str(&mut self, s: &str) -> std::io::Result<()> {
        let mut w = self.writer.lock().unwrap();
        w.write_all(s.as_bytes())?;
        w.flush()?;
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionKind {
    Termux,
    HostShell,
}

pub struct SessionPlugin;

impl Plugin for SessionPlugin {
    fn build(&self, app: &mut App) {
        let output_buf = Arc::new(Mutex::new(Vec::<u8>::new()));
        let (kind, label, writer) = match spawn_session(output_buf.clone()) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[Session] Failed to start PTY: {e}");
                // Provide a dummy writer so the app still runs.
                let dummy: Arc<Mutex<Box<dyn Write + Send>>> =
                    Arc::new(Mutex::new(Box::new(std::io::sink())));
                (
                    SessionKind::HostShell,
                    format!("failed: {e}"),
                    dummy,
                )
            }
        };

        app.insert_resource(SessionOutput {
            buffer: output_buf,
        })
        .insert_resource(SessionBridge {
            writer,
            kind,
            label: label.clone(),
        });

        println!("[Session] Started ({kind:?}) — {label}");
        println!("[Session] Session lives behind PTY; display is inside the physics world");
    }
}

fn detect_termux() -> Option<std::path::PathBuf> {
    if std::env::var_os("TERMUX_VERSION").is_some() {
        if let Ok(prefix) = std::env::var("PREFIX") {
            let sh = std::path::Path::new(&prefix).join("bin/bash");
            if sh.exists() {
                return Some(sh);
            }
            let login = std::path::Path::new(&prefix).join("bin/login");
            if login.exists() {
                return Some(login);
            }
        }
    }
    // Common Termux prefix on device
    let candidates = [
        "/data/data/com.termux/files/usr/bin/bash",
        "/data/data/com.termux/files/usr/bin/login",
    ];
    for c in candidates {
        let p = std::path::PathBuf::from(c);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn spawn_session(
    output_buf: Arc<Mutex<Vec<u8>>>,
) -> Result<(SessionKind, String, Arc<Mutex<Box<dyn Write + Send>>>), String> {
    let pty_system = NativePtySystem::default();
    let pair = pty_system
        .openpty(PtySize {
            rows: 24,
            cols: 80,
            pixel_width: 0,
            pixel_height: 0,
        })
        .map_err(|e| e.to_string())?;

    let (kind, mut cmd, label) = if let Some(termux_shell) = detect_termux() {
        let mut c = CommandBuilder::new(termux_shell.to_string_lossy().as_ref());
        c.env("TERM", "xterm-256color");
        (
            SessionKind::Termux,
            c,
            format!("Termux shell at {}", termux_shell.display()),
        )
    } else {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| {
            if cfg!(windows) {
                "cmd.exe".into()
            } else {
                "/bin/bash".into()
            }
        });
        let mut c = CommandBuilder::new(&shell);
        // Login-ish interactive
        if !cfg!(windows) {
            c.arg("-i");
        }
        c.env("TERM", "xterm-256color");
        c.env("VCE_TERMUX_WORLD", "1");
        (
            SessionKind::HostShell,
            c,
            format!("Host shell {shell} (Termux not detected — same PTY bridge)"),
        )
    };

    let mut child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| e.to_string())?;

    // Reader thread → shared output buffer
    let mut reader = pair.master.try_clone_reader().map_err(|e| e.to_string())?;
    let buf_clone = output_buf.clone();
    thread::spawn(move || {
        let mut tmp = [0u8; 4096];
        loop {
            match reader.read(&mut tmp) {
                Ok(0) => break,
                Ok(n) => {
                    if let Ok(mut g) = buf_clone.lock() {
                        g.extend_from_slice(&tmp[..n]);
                    }
                }
                Err(_) => break,
            }
        }
        let _ = child;
    });

    let writer = pair
        .master
        .take_writer()
        .map_err(|e| e.to_string())?;

    let writer: Arc<Mutex<Box<dyn Write + Send>>> = Arc::new(Mutex::new(writer));

    // Banner into the session so the user sees context immediately
    {
        let mut w = writer.lock().unwrap();
        let banner = format!(
            "\r\n[termux-in-physics-world] session={kind:?}\r\n[termux-in-physics-world] display is a physics entity\r\n\r\n"
        );
        let _ = w.write_all(banner.as_bytes());
        let _ = w.flush();
    }

    Ok((kind, label, writer))
}
