#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::fmt::Write as _;

mod application;
mod domain;
mod engine;
mod platform;
mod ui;

#[cfg(test)]
use crate::application::app::*;
#[cfg(test)]
use crate::domain::layout::*;

fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        let text = panic_text(info);
        let log_path = crash_log_path();
        let log_error = std::fs::write(&log_path, &text).err();
        report_crash(&text, &log_path, log_error);
    }));
}

fn panic_text(info: &std::panic::PanicHookInfo<'_>) -> String {
    let mut out = String::new();
    let payload = info.payload();
    let msg = payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        .unwrap_or("(panic payload is not valid UTF-8 text)");
    let _ = writeln!(out, "panic: {msg}");
    if let Some(loc) = info.location() {
        let _ = writeln!(out, "location: {}:{}", loc.file(), loc.line());
    }
    let _ = writeln!(
        out,
        "thread: {}",
        std::thread::current().name().unwrap_or("(unnamed)")
    );
    let _ = writeln!(out, "time: {:?}", std::time::SystemTime::now());
    let _ = writeln!(out, "version: {}", env!("CARGO_PKG_VERSION"));
    let _ = writeln!(out, "executable: {:?}", std::env::current_exe());
    let _ = writeln!(
        out,
        "\n--- backtrace ---\n{}",
        std::backtrace::Backtrace::force_capture()
    );
    out
}

fn crash_log_path() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("roundtable-crash.log")))
        .unwrap_or_else(|| std::path::PathBuf::from("roundtable-crash.log"))
}

fn report_crash(text: &str, log_path: &std::path::Path, log_error: Option<std::io::Error>) {
    eprintln!("{text}");
    if cfg!(debug_assertions) {
        return;
    }
    let persistence = match log_error {
        Some(error) => format!(
            "Crash log could not be written to {}:\n{error}",
            log_path.display()
        ),
        None => format!("Crash log: {}", log_path.display()),
    };
    let body = format!("RoundTable crashed.\n{persistence}\n\n{text}");
    platform::message_box("RoundTable - Crash", &body);
}

fn main() {
    install_panic_hook();
    if !platform::acquire_single_instance() {
        return;
    }
    let result = run();

    if let Err(e) = &result {
        let body = format!("RoundTable could not start:\n{e}");
        eprintln!("{body}");
        if !cfg!(debug_assertions) {
            platform::message_box("RoundTable - Startup Failed", &body);
        }
    }
}

fn run() -> Result<(), String> {
    platform::run_app()
}

#[cfg(test)]
mod tests;
