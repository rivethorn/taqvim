use tray_icon::{TrayIconBuilder, Icon};
use tray_icon::menu::{Menu, MenuItem, MenuEvent};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, channel};
use std::thread;

static QUIT_REQUESTED: AtomicBool = AtomicBool::new(false);

pub fn quit_requested() -> bool {
    QUIT_REQUESTED.load(Ordering::Relaxed)
}

pub fn start_tray() -> std::sync::mpsc::Sender<u32> {
    let (tx, rx): (std::sync::mpsc::Sender<u32>, Receiver<u32>) = channel();
    thread::spawn(move || {
        let menu = Menu::new();
        let quit_item = MenuItem::with_id("quit", "Quit", true, None);
        let _ = menu.append(&quit_item);

        let icon = make_icon(0);
        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("Taqvim")
            .with_icon(icon)
            .build()
            .expect("failed to build tray icon");
        loop {
            if let Ok(event) = MenuEvent::receiver().try_recv() {
                if event.id() == "quit" {
                    QUIT_REQUESTED.store(true, Ordering::Relaxed);
                    break;
                }
            }
            match rx.recv_timeout(std::time::Duration::from_secs(1)) {
                Ok(day) => {
                    let _ = tray.set_icon(Some(make_icon(day)));
                    let pd = crate::calendar::today_persian();
                    let tip = format!(
                        "{} - {}",
                        crate::calendar::weekday_name(pd),
                        crate::calendar::format_persian(pd)
                    );
                    let _ = tray.set_tooltip(Some(tip));
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
            if QUIT_REQUESTED.load(Ordering::Relaxed) {
                break;
            }
        }
    });
    tx
}

fn make_icon(day: u32) -> Icon {
    let w = 32u32;
    let h = 32u32;
    let wi = w as i32;
    let hi = h as i32;
    let mut buf = vec![0u8; (w * h * 4) as usize];

    // Dark slate background.
    for i in (0..buf.len()).step_by(4) {
        buf[i] = 34;
        buf[i + 1] = 45;
        buf[i + 2] = 62;
        buf[i + 3] = 255;
    }

    // White border.
    for y in 0..h {
        for x in 0..w {
            if x < 3 || y < 3 || x >= w - 3 || y >= h - 3 {
                let idx = ((y * w + x) * 4) as usize;
                buf[idx] = 226;
                buf[idx + 1] = 226;
                buf[idx + 2] = 232;
                buf[idx + 3] = 255;
            }
        }
    }

    let s = day.to_string();
    let scale: u32 = if s.len() == 1 { 4 } else { 3 };
    let digit_w = 5 * scale;
    let gap = 2 * scale;
    let total_w = s.len() as u32 * digit_w + s.len().saturating_sub(1) as u32 * gap;
    let total_h = 7 * scale;
    let mut cx = ((wi) - (total_w as i32)) / 2;
    let start_y = ((hi) - (total_h as i32)) / 2;
    const DIGIT_BITMAPS: [[u8; 7]; 10] = [
        // 0
        [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        // 1
        [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        // 2
        [0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111],
        // 3
        [0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110],
        // 4
        [0b00011, 0b00101, 0b01001, 0b10001, 0b11111, 0b00001, 0b00001],
        // 5
        [0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b10001, 0b01110],
        // 6
        [0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        // 7
        [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b10000],
        // 8
        [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        // 9
        [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b10001, 0b01110],
    ];
    fn draw_digit(
        buf: &mut [u8],
        w: i32,
        h: i32,
        dx: i32,
        dy: i32,
        digit: u8,
        scale: u32,
        color: [u8; 4],
    ) {
        let bm = DIGIT_BITMAPS[digit as usize];
        let sc = scale as i32;
        for row in 0..7i32 {
            let bits = bm[row as usize];
            for col in 0..5i32 {
                if bits & (1 << (4 - col)) != 0 {
                    for sy in 0..sc {
                        for sx in 0..sc {
                            let px = dx + col * sc + sx;
                            let py = dy + row * sc + sy;
                            if px >= 0 && py >= 0 && px < w && py < h {
                                let idx = ((py as u32 * w as u32 + px as u32) * 4) as usize;
                                buf[idx] = color[0];
                                buf[idx + 1] = color[1];
                                buf[idx + 2] = color[2];
                                buf[idx + 3] = color[3];
                            }
                        }
                    }
                }
            }
        }
    }

    let s = day.to_string();
    let scale: u32 = if s.len() == 1 { 4 } else { 3 };
    let digit_w = 5 * scale;
    let gap = 2 * scale;
    let total_w = s.len() as u32 * digit_w + s.len().saturating_sub(1) as u32 * gap;
    let total_h = 7 * scale;
    let mut cx = ((wi) - (total_w as i32)) / 2;
    let start_y = ((hi) - (total_h as i32)) / 2;
    for ch in s.chars() {
        if let Some(d) = ch.to_digit(10) {
            draw_digit(&mut buf, wi, hi, cx, start_y, d as u8, scale, [255, 255, 255, 255]);
            cx += digit_w as i32 + gap as i32;
        }
    }

    Icon::from_rgba(buf, w, h).expect("icon")
}

