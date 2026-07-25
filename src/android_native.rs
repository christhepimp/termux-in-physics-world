//! In-process Android path — used when the physics engine is compiled as an APK.
//!
//! On device, this process *is* the Android app. Termux is launched via intents
//! from this process; frames/input use native hooks (fill in with JNI as needed).

use crate::shared_buffer::FramePixels;

/// Called once from the embedded runtime service thread on Android.
pub fn init_in_process() {
    println!("[AndroidNative] In-process Android path initialized");
    println!("[AndroidNative] Physics process == Android app process");
}

pub fn launch_termux_in_process() {
    println!("[AndroidNative] Launch Termux via in-process Activity intent (JNI bind me)");
    // Future: JNI → Android Context.startActivity(Termux)
}

pub fn inject_text(text: &str) {
    let _ = text;
    // Future: Instrumentation / InputManager from this process
}

pub fn inject_key(code: i32) {
    let _ = code;
}

pub fn capture_frame_in_process() -> Result<FramePixels, String> {
    // Future: ImageReader / MediaProjection / Surface into RGBA
    // Until bound, return a placeholder so the in-process path compiles.
    let w = 64u32;
    let h = 64u32;
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    for px in rgba.chunks_mut(4) {
        px[0] = 20;
        px[1] = 40;
        px[2] = 30;
        px[3] = 255;
    }
    Ok(FramePixels {
        width: w,
        height: h,
        rgba,
    })
}
