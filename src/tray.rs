use tray_icon::{TrayIconBuilder, Icon};
use std::sync::mpsc::{Receiver, channel};
use std::thread;

pub fn start_tray() -> std::sync::mpsc::Sender<String> {
    let (tx, rx): (std::sync::mpsc::Sender<String>, Receiver<String>) = channel();
    thread::spawn(move || {
        // build icon
        let icon = make_icon();
        let tray = TrayIconBuilder::new()
            .with_tooltip("Taqvim")
            .with_icon(icon)
            .build()
            .expect("failed to build tray icon");
        // keep alive
        loop {
            match rx.recv_timeout(std::time::Duration::from_secs(1)) {
                Ok(text) => {
                    let _ = tray.set_tooltip(Some(text));
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
            // keep tray alive
            let _ = &tray;
        }
    });
    tx
}

fn make_icon() -> Icon {
    // 32x32 simple blue square with white center
    let w = 32u32;
    let h = 32u32;
    let mut rgba = vec![0u8; (w*h*4) as usize];
    for y in 0..h {
        for x in 0..w {
            let idx = ((y*w + x)*4) as usize;
            // border
            if x < 2 || y < 2 || x >= w-2 || y >= h-2 {
                rgba[idx] = 255; rgba[idx+1]=255; rgba[idx+2]=255; rgba[idx+3]=255;
            } else {
                rgba[idx] = 30; rgba[idx+1]=80; rgba[idx+2]=200; rgba[idx+3]=255;
            }
        }
    }
    Icon::from_rgba(rgba, w, h).expect("icon")
}
