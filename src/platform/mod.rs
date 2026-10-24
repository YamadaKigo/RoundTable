#[cfg(target_os = "windows")]
mod win32;

pub(crate) mod assets;
pub(crate) mod pointer;

#[cfg(target_os = "windows")]
mod windows {
    pub(crate) use super::win32::*;
}

#[cfg(target_os = "windows")]
pub(crate) use windows::*;

#[cfg(not(target_os = "windows"))]
mod fallback {
    pub(crate) const CF_UNICODETEXT: u32 = 13;
    pub(crate) const SM_CXSCREEN: i32 = 0;
    pub(crate) const SM_CYSCREEN: i32 = 1;

    pub(crate) fn message_box(title: &str, body: &str) {
        eprintln!("{title}: {body}");
    }

    pub(crate) fn acquire_single_instance() -> bool {
        static ACQUIRED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        !ACQUIRED.swap(true, std::sync::atomic::Ordering::AcqRel)
    }

    pub(crate) fn screen_area() -> Option<(i32, i32, i32, i32)> {
        None
    }

    pub(crate) fn enter_fullscreen(_hwnd: isize) -> bool {
        false
    }

    pub(crate) fn cursor_pos() -> Option<(i32, i32)> {
        None
    }

    pub(crate) fn set_cursor_pos(_x: i32, _y: i32) -> bool {
        false
    }

    pub(crate) fn current_time_text() -> String {
        "00:00:00".to_owned()
    }

    pub(crate) fn seconds_to_next_wall_second() -> f64 {
        1.0
    }

    pub(crate) fn client_origin(_hwnd: isize) -> Option<(i32, i32)> {
        None
    }

    pub(crate) fn clipboard_text() -> Option<String> {
        None
    }

    pub(crate) fn mouse_feel() -> super::pointer::PointerFeel {
        super::pointer::PointerFeel::default()
    }

    pub(crate) fn run_app() -> Result<(), String> {
        Err("The window backend is not supported on this platform".to_owned())
    }
}

#[cfg(not(target_os = "windows"))]
pub(crate) use fallback::*;
