#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use eframe::egui;
use std::fmt::Write as _;

mod app;
mod assets;
mod data;
mod ecs;
mod font;
mod grid;
mod indicator;
mod platform;
mod pointer;
mod script;
mod theme;
mod ui;

#[cfg(target_os = "windows")]
mod win32;

use crate::app::*;
use crate::grid::*;

fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let text = panic_text(info);
        let _ = std::fs::write(crash_log_path(), &text);
        report_crash(&text);
    }));
}

fn panic_text(info: &std::panic::PanicHookInfo<'_>) -> String {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut out = String::new();
        let payload = info.payload();
        let msg = payload
            .downcast_ref::<&str>()
            .copied()
            .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
            .unwrap_or("(读不出 panic 正文)");
        let _ = writeln!(out, "panic: {msg}");
        if let Some(loc) = info.location() {
            let _ = writeln!(out, "位置: {}:{}", loc.file(), loc.line());
        }
        let _ = writeln!(
            out,
            "线程: {}",
            std::thread::current().name().unwrap_or("(无名)")
        );
        let _ = writeln!(out, "时间: {:?}", std::time::SystemTime::now());
        let _ = writeln!(out, "版本: {}", env!("CARGO_PKG_VERSION"));
        let _ = writeln!(
            out,
            "\n--- backtrace ---\n{}",
            std::backtrace::Backtrace::force_capture()
        );
        out
    }))
    .unwrap_or_else(|_| "panic（连 panic 钩子自己都炸了）".to_string())
}

fn crash_log_path() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("roundtable-crash.log")))
        .unwrap_or_else(|| std::path::PathBuf::from("roundtable-crash.log"))
}

fn report_crash(text: &str) {
    eprintln!("{text}");
    if cfg!(debug_assertions) {
        return;
    }
    let body = format!(
        "圆桌崩了，这一句记在：\n{}\n\n{}",
        crash_log_path().display(),
        text
    );
    platform::message_box("圆桌 - 崩溃", &body);
}

fn main() -> eframe::Result {
    install_panic_hook();
    if !platform::acquire_single_instance() {
        return Ok(());
    }
    let result = run();

    if let Err(e) = &result {
        let body = format!("圆桌起不来：\n{e}");
        eprintln!("{body}");
        if !cfg!(debug_assertions) {
            platform::message_box("圆桌 - 启动失败", &body);
        }
    }
    result
}

fn run() -> eframe::Result {
    let (win_x, win_y, win_w, win_h) = match platform::screen_area().or_else(platform::work_area) {
        Some((x, y, w, h)) => (x as f32, y as f32, w as f32, h as f32),
        None => (0.0, 0.0, WINDOW_W, WINDOW_H),
    };
    let mut wgpu_options = eframe::WgpuConfiguration::default();
    wgpu_options.surface.present_mode = eframe::wgpu::PresentMode::AutoNoVsync;
    wgpu_options.surface.desired_maximum_frame_latency = Some(1);
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(APP_NAME)
            .with_decorations(false)
            .with_position([win_x, win_y])
            .with_inner_size([win_w, win_h])
            .with_min_inner_size([WINDOW_MIN_W, WINDOW_MIN_H])
            .with_fullscreen(true),
        wgpu_options,
        ..Default::default()
    };

    eframe::run_native(
        APP_NAME,
        native_options,
        Box::new(|cc| Ok(Box::new(StockApp::new(cc)))),
    )
}

#[cfg(test)]
mod tests;
