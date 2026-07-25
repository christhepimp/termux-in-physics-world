//! In-process shared state between physics systems and runtime service threads.

use std::sync::{Arc, Mutex};

use bevy::prelude::*;

#[derive(Clone)]
pub struct FramePixels {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

/// Virtual display buffer living inside the physics process.
#[derive(Resource, Clone)]
pub struct SharedFrameBuffer {
    inner: Arc<Mutex<Option<FramePixels>>>,
}

impl Default for SharedFrameBuffer {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
        }
    }
}

impl SharedFrameBuffer {
    pub fn handle(&self) -> Arc<Mutex<Option<FramePixels>>> {
        self.inner.clone()
    }

    pub fn latest(&self) -> Option<FramePixels> {
        self.inner.lock().ok().and_then(|g| g.clone())
    }

    pub fn publish(handle: &Arc<Mutex<Option<FramePixels>>>, frame: FramePixels) {
        if let Ok(mut g) = handle.lock() {
            *g = Some(frame);
        }
    }
}

#[derive(Clone, Debug)]
pub enum InputCmd {
    Text(String),
    Key(i32),
}

/// Virtual input queue living inside the physics process.
#[derive(Resource, Clone)]
pub struct InputQueue {
    inner: Arc<Mutex<Vec<InputCmd>>>,
}

impl Default for InputQueue {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl InputQueue {
    pub fn handle(&self) -> Arc<Mutex<Vec<InputCmd>>> {
        self.inner.clone()
    }

    pub fn push_text(&self, s: &str) {
        if let Ok(mut g) = self.inner.lock() {
            g.push(InputCmd::Text(s.to_string()));
        }
    }

    pub fn push_key(&self, code: i32) {
        if let Ok(mut g) = self.inner.lock() {
            g.push(InputCmd::Key(code));
        }
    }

    pub fn drain(handle: &Arc<Mutex<Vec<InputCmd>>>) -> Vec<InputCmd> {
        handle
            .lock()
            .map(|mut g| std::mem::take(&mut *g))
            .unwrap_or_default()
    }
}
