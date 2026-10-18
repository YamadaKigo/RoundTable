use raw_window_handle::RawWindowHandle;
use std::sync::atomic::{AtomicIsize, Ordering};

#[link(name = "user32")]
unsafe extern "system" {

    pub(crate) fn MessageBoxW(hwnd: isize, text: *const u16, caption: *const u16, kind: u32)
    -> i32;
    pub(crate) fn GetWindowLongW(hwnd: isize, index: i32) -> i32;
    pub(crate) fn SetWindowLongW(hwnd: isize, index: i32, value: i32) -> i32;
    pub(crate) fn SetWindowLongPtrW(hwnd: isize, index: i32, value: isize) -> isize;
    pub(crate) fn SetWindowPos(
        hwnd: isize,
        after: isize,
        x: i32,
        y: i32,
        cx: i32,
        cy: i32,
        flags: u32,
    ) -> i32;
    pub(crate) fn DefWindowProcW(hwnd: isize, msg: u32, wp: usize, lp: isize) -> isize;
    pub(crate) fn CallWindowProcW(
        prev: isize,
        hwnd: isize,
        msg: u32,
        wp: usize,
        lp: isize,
    ) -> isize;
    pub(crate) fn OpenClipboard(hwnd: isize) -> i32;
    pub(crate) fn CloseClipboard() -> i32;
    pub(crate) fn IsClipboardFormatAvailable(format: u32) -> i32;
    pub(crate) fn GetClipboardData(format: u32) -> isize;
    pub(crate) fn SystemParametersInfoW(
        action: u32,
        param: u32,
        data: *mut core::ffi::c_void,
        win_ini: u32,
    ) -> i32;
    pub(crate) fn GetSystemMetrics(index: i32) -> i32;
    pub(crate) fn GetCursorPos(p: *mut Point) -> i32;
    pub(crate) fn SetCursorPos(x: i32, y: i32) -> i32;
    pub(crate) fn ClientToScreen(hwnd: isize, p: *mut Point) -> i32;
    pub(crate) fn GetLocalTime(time: *mut SystemTime);
}

#[repr(C)]
pub(crate) struct Point {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

#[repr(C)]
pub(crate) struct SystemTime {
    year: u16,
    month: u16,
    day_of_week: u16,
    day: u16,
    hour: u16,
    minute: u16,
    second: u16,
    milliseconds: u16,
}

#[link(name = "kernel32")]
unsafe extern "system" {
    pub(crate) fn CreateMutexW(
        attrs: *mut core::ffi::c_void,
        owner: i32,
        name: *const u16,
    ) -> isize;
    pub(crate) fn CloseHandle(handle: isize) -> i32;
    pub(crate) fn GetLastError() -> u32;
    pub(crate) fn SetLastError(error: u32);
    pub(crate) fn GlobalLock(handle: isize) -> *mut u8;
    pub(crate) fn GlobalUnlock(handle: isize) -> i32;
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

pub(crate) fn message_box(title: &str, body: &str) {
    let title = wide(title);
    let body = wide(body);
    unsafe {
        MessageBoxW(0, body.as_ptr(), title.as_ptr(), MB_OK | MB_ICONERROR);
    }
}

static SINGLE_INSTANCE_HANDLE: AtomicIsize = AtomicIsize::new(0);

pub(crate) fn acquire_single_instance() -> bool {
    if SINGLE_INSTANCE_HANDLE.load(Ordering::Acquire) != 0 {
        return true;
    }
    let name = wide("Local\\RoundTable.SingleInstance");
    unsafe {
        SetLastError(0);
        let handle = CreateMutexW(core::ptr::null_mut(), 0, name.as_ptr());
        if handle == 0 {
            return false;
        }
        if GetLastError() == 183 {
            CloseHandle(handle);
            return false;
        }
        SINGLE_INSTANCE_HANDLE.store(handle, Ordering::Release);
    }
    true
}

pub(crate) const CF_UNICODETEXT: u32 = 13;

pub(crate) const SM_CXSCREEN: i32 = 0;
pub(crate) const SM_CYSCREEN: i32 = 1;

pub(crate) fn screen_area() -> Option<(i32, i32, i32, i32)> {
    let (w, h) = unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) };
    if w <= 0 || h <= 0 {
        return None;
    }
    Some((0, 0, w, h))
}

pub(crate) fn cursor_pos() -> Option<(i32, i32)> {
    let mut p = Point { x: 0, y: 0 };
    let ok = unsafe { GetCursorPos(&mut p) };
    (ok != 0).then_some((p.x, p.y))
}

pub(crate) fn set_cursor_pos(x: i32, y: i32) -> bool {
    unsafe { SetCursorPos(x, y) != 0 }
}

pub(crate) fn current_time_text() -> String {
    let mut time = SystemTime {
        year: 0,
        month: 0,
        day_of_week: 0,
        day: 0,
        hour: 0,
        minute: 0,
        second: 0,
        milliseconds: 0,
    };
    unsafe { GetLocalTime(&mut time) };
    format!("{:02}:{:02}:{:02}", time.hour, time.minute, time.second)
}

pub(crate) fn client_origin(hwnd: isize) -> Option<(i32, i32)> {
    if hwnd == 0 {
        return None;
    }
    let mut p = Point { x: 0, y: 0 };
    let ok = unsafe { ClientToScreen(hwnd, &mut p) };
    (ok != 0).then_some((p.x, p.y))
}

pub(crate) fn mouse_feel() -> crate::pointer::PointerFeel {
    crate::pointer::PointerFeel::default()
}

pub(crate) const SPI_GETWORKAREA: u32 = 0x0030;

pub(crate) fn work_area() -> Option<(i32, i32, i32, i32)> {
    #[repr(C)]
    struct Rect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }
    let mut r = Rect {
        left: 0,
        top: 0,
        right: 0,
        bottom: 0,
    };
    let ok = unsafe {
        SystemParametersInfoW(
            SPI_GETWORKAREA,
            0,
            &mut r as *mut Rect as *mut core::ffi::c_void,
            0,
        )
    };
    if ok == 0 || r.right <= r.left || r.bottom <= r.top {
        return None;
    }
    Some((r.left, r.top, r.right - r.left, r.bottom - r.top))
}

pub(crate) fn clipboard_text() -> Option<String> {
    unsafe {
        if IsClipboardFormatAvailable(CF_UNICODETEXT) == 0 {
            return None;
        }

        let mut opened = false;
        for _ in 0..8 {
            if OpenClipboard(0) != 0 {
                opened = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        if !opened {
            return None;
        }
        let mut out = None;
        let handle = GetClipboardData(CF_UNICODETEXT);
        if handle != 0 {
            let ptr = GlobalLock(handle);
            if !ptr.is_null() {
                let mut len = 0usize;
                while len < (1 << 22) && *(ptr as *const u16).add(len) != 0 {
                    len += 1;
                }
                let wide = std::slice::from_raw_parts(ptr as *const u16, len);
                out = Some(String::from_utf16_lossy(wide));
                GlobalUnlock(handle);
            }
        }
        CloseClipboard();
        out
    }
}

pub(crate) const GWL_STYLE: i32 = -16;
pub(crate) const GWL_EXSTYLE: i32 = -20;
pub(crate) const GWLP_WNDPROC: i32 = -4;

pub(crate) const MB_ICONERROR: u32 = 0x0000_0010;
pub(crate) const MB_OK: u32 = 0x0000_0000;
pub(crate) const WS_CAPTION: i32 = 0x00C0_0000;
pub(crate) const WS_THICKFRAME: i32 = 0x0004_0000;
pub(crate) const WS_EX_WINDOWEDGE: i32 = 0x0000_0100;
pub(crate) const WS_EX_CLIENTEDGE: i32 = 0x0000_0200;
pub(crate) const WS_EX_STATICEDGE: i32 = 0x0000_2000;
pub(crate) const SWP_NOSIZE: u32 = 0x0001;
pub(crate) const SWP_NOMOVE: u32 = 0x0002;
pub(crate) const SWP_NOZORDER: u32 = 0x0004;
pub(crate) const SWP_NOACTIVATE: u32 = 0x0010;
pub(crate) const SWP_FRAMECHANGED: u32 = 0x0020;

pub(crate) const WM_NCCALCSIZE: u32 = 0x0083;

pub(crate) static PREV_WNDPROC: AtomicIsize = AtomicIsize::new(0);

pub(crate) fn win32_hwnd(raw: RawWindowHandle) -> isize {
    match raw {
        RawWindowHandle::Win32(h) => h.hwnd.get(),
        _ => 0,
    }
}

pub(crate) fn clear_window_frame(hwnd: isize) {
    if hwnd == 0 {
        return;
    }
    let style = unsafe { GetWindowLongW(hwnd, GWL_STYLE) };
    let ex_style = unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) };
    let style_dirty = style & (WS_CAPTION | WS_THICKFRAME) != 0;
    let ex_dirty = ex_style & (WS_EX_WINDOWEDGE | WS_EX_CLIENTEDGE | WS_EX_STATICEDGE) != 0;
    if !style_dirty && !ex_dirty {
        return;
    }
    unsafe {
        if style_dirty {
            SetWindowLongW(hwnd, GWL_STYLE, style & !(WS_CAPTION | WS_THICKFRAME));
        }
        if ex_dirty {
            SetWindowLongW(
                hwnd,
                GWL_EXSTYLE,
                ex_style & !(WS_EX_WINDOWEDGE | WS_EX_CLIENTEDGE | WS_EX_STATICEDGE),
            );
        }
        SetWindowPos(
            hwnd,
            0,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );
    }
}

unsafe extern "system" fn rt_wndproc(hwnd: isize, msg: u32, wp: usize, lp: isize) -> isize {
    if msg == WM_NCCALCSIZE && wp != 0 {
        return 0;
    }

    let prev = PREV_WNDPROC.load(Ordering::Relaxed);
    unsafe {
        if prev == 0 {
            DefWindowProcW(hwnd, msg, wp, lp)
        } else {
            CallWindowProcW(prev, hwnd, msg, wp, lp)
        }
    }
}

pub(crate) fn install_client_cover(hwnd: isize) -> bool {
    if hwnd == 0 {
        return false;
    }
    if PREV_WNDPROC.load(Ordering::Relaxed) != 0 {
        return true;
    }
    let prev = unsafe { SetWindowLongPtrW(hwnd, GWLP_WNDPROC, rt_wndproc as *const () as isize) };
    if prev == 0 {
        return false;
    }
    PREV_WNDPROC.store(prev, Ordering::Relaxed);
    unsafe {
        SetWindowPos(
            hwnd,
            0,
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED,
        );
    }
    true
}
