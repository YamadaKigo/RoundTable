use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicIsize, Ordering};

use crate::engine::core::math::{Rect, pos2, vec2};
use crate::engine::gfx::canvas::Canvas;
use crate::engine::kit::{
    self, ColorImage, Event, ImageData, ImeEvent, Key, Modifiers, MouseWheelUnit, PointerButton,
    TextureId,
};

#[link(name = "user32")]
unsafe extern "system" {

    pub(crate) fn MessageBoxW(hwnd: isize, text: *const u16, caption: *const u16, kind: u32)
    -> i32;
    pub(crate) fn GetWindowLongW(hwnd: isize, index: i32) -> i32;
    pub(crate) fn SetWindowLongW(hwnd: isize, index: i32, value: i32) -> i32;
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
    fn SetCursor(cursor: isize) -> isize;
    fn LoadCursorW(instance: isize, cursor_name: *const u16) -> isize;
    pub(crate) fn ClientToScreen(hwnd: isize, p: *mut Point) -> i32;
    pub(crate) fn GetLocalTime(time: *mut SystemTime);
    pub(crate) fn MonitorFromWindow(hwnd: isize, flags: u32) -> isize;
    fn GetMonitorInfoW(monitor: isize, info: *mut NativeMonitorInfo) -> i32;
    fn RegisterClassW(class: *const WindowClassW) -> u16;
    fn CreateWindowExW(
        ex_style: u32,
        class_name: *const u16,
        window_name: *const u16,
        style: u32,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        parent: isize,
        menu: isize,
        instance: isize,
        param: *mut core::ffi::c_void,
    ) -> isize;
    fn ShowWindow(hwnd: isize, command: i32) -> i32;
    fn UpdateWindow(hwnd: isize) -> i32;
    fn PeekMessageW(message: *mut MessageW, hwnd: isize, min: u32, max: u32, remove: u32) -> i32;
    fn TranslateMessage(message: *const MessageW) -> i32;
    fn DispatchMessageW(message: *const MessageW) -> isize;
    fn PostQuitMessage(code: i32);
    fn DestroyWindow(hwnd: isize) -> i32;
    fn ValidateRect(hwnd: isize, rect: *const NativeRect) -> i32;
    fn GetClientRect(hwnd: isize, rect: *mut NativeRect) -> i32;
    fn ScreenToClient(hwnd: isize, point: *mut Point) -> i32;
    fn GetKeyState(key: i32) -> i16;
    fn SetTimer(hwnd: isize, id: usize, interval: u32, callback: usize) -> usize;
    fn KillTimer(hwnd: isize, id: usize) -> i32;
    fn RegisterRawInputDevices(devices: *const RawInputDevice, count: u32, size: u32) -> i32;
    fn GetRawInputData(
        input: isize,
        command: u32,
        data: *mut core::ffi::c_void,
        size: *mut u32,
        header_size: u32,
    ) -> u32;
    fn GetDC(hwnd: isize) -> isize;
    fn ReleaseDC(hwnd: isize, dc: isize) -> i32;
    fn CreateWaitableTimerExW(
        attrs: *mut core::ffi::c_void,
        name: *const u16,
        flags: u32,
        access: u32,
    ) -> isize;
    fn SetWaitableTimer(
        timer: isize,
        due_time: *const i64,
        period: i32,
        completion: isize,
        arg: *mut core::ffi::c_void,
        resume: i32,
    ) -> i32;
    fn MsgWaitForMultipleObjectsEx(
        count: u32,
        handles: *const isize,
        milliseconds: u32,
        wake_mask: u32,
        flags: u32,
    ) -> u32;
    fn EmptyClipboard() -> i32;
    fn SetClipboardData(format: u32, memory: isize) -> isize;
}

#[repr(C)]
#[derive(Default)]
pub(crate) struct Point {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

#[repr(C)]
#[derive(Default)]
struct NativeRect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
#[derive(Default)]
struct NativeBitmapHeader {
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bit_count: u16,
    compression: u32,
    size_image: u32,
    x_pels_per_meter: i32,
    y_pels_per_meter: i32,
    clr_used: u32,
    clr_important: u32,
}

#[repr(C)]
#[derive(Default)]
struct NativeBitmapInfo {
    header: NativeBitmapHeader,
    colors: [u32; 3],
}

#[repr(C)]
struct NativeMonitorInfo {
    cb_size: u32,
    monitor: NativeRect,
    work: NativeRect,
    flags: u32,
}

#[repr(C)]
struct WindowClassW {
    style: u32,
    wndproc: unsafe extern "system" fn(isize, u32, usize, isize) -> isize,
    class_extra: i32,
    window_extra: i32,
    instance: isize,
    icon: isize,
    cursor: isize,
    background: isize,
    menu_name: *const u16,
    class_name: *const u16,
}

#[repr(C)]
#[derive(Default)]
struct MessageW {
    hwnd: isize,
    message: u32,
    wparam: usize,
    lparam: isize,
    time: u32,
    point: Point,
    private: u32,
}

#[repr(C)]
struct RawInputDevice {
    usage_page: u16,
    usage: u16,
    flags: u32,
    target: isize,
}

#[repr(C)]
#[derive(Default)]
struct RawInputHeader {
    kind: u32,
    _size: u32,
    _device: isize,
    _wparam: usize,
}

#[repr(C)]
struct RawInputMouse {
    flags: u16,
    _button_flags: u16,
    _button_data: u16,
    _padding: u16,
    _raw_buttons: u32,
    last_x: i32,
    last_y: i32,
    _extra_information: u32,
}

#[repr(C)]
#[derive(Default)]
struct CompositionForm {
    style: u32,
    point: Point,
    area: NativeRect,
}

#[repr(C)]
#[derive(Default)]
struct CandidateForm {
    index: u32,
    style: u32,
    point: Point,
    area: NativeRect,
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
    pub(crate) fn GlobalAlloc(flags: u32, bytes: usize) -> isize;
    pub(crate) fn GetModuleHandleW(name: *const u16) -> isize;
    fn QueryPerformanceCounter(value: *mut i64) -> i32;
    fn QueryPerformanceFrequency(value: *mut i64) -> i32;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    fn SetDIBitsToDevice(
        dc: isize,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
        source_x: i32,
        source_y: i32,
        start_scan: u32,
        scan_lines: u32,
        bits: *const u8,
        info: *const NativeBitmapInfo,
        usage: u32,
    ) -> i32;
}

#[link(name = "imm32")]
unsafe extern "system" {
    fn ImmGetContext(hwnd: isize) -> isize;
    fn ImmReleaseContext(hwnd: isize, imc: isize) -> i32;
    fn ImmGetCompositionStringW(
        imc: isize,
        index: u32,
        buffer: *mut core::ffi::c_void,
        bytes: u32,
    ) -> i32;
    fn ImmSetCompositionWindow(imc: isize, form: *const CompositionForm) -> i32;
    fn ImmSetCandidateWindow(imc: isize, form: *const CandidateForm) -> i32;
}

thread_local! {
    static EVENT_QUEUE: RefCell<Vec<Event>> = const { RefCell::new(Vec::new()) };
    static EVENT_PENDING: Cell<bool> = const { Cell::new(false) };
    static URGENT_PENDING: Cell<bool> = const { Cell::new(false) };
    static REPAINT_ALL: Cell<bool> = const { Cell::new(true) };
    static EVENT_MODIFIERS: Cell<Modifiers> = const { Cell::new(Modifiers::NONE) };
    static CLIENT_SIZE: Cell<(i32, i32)> = const { Cell::new((1, 1)) };
    static QUIT: Cell<bool> = const { Cell::new(false) };
    static IME_CURSOR: Cell<Option<(i32, i32, i32, i32)>> = const { Cell::new(None) };
    static MINIMIZED: Cell<bool> = const { Cell::new(false) };
    static FOCUSED: Cell<bool> = const { Cell::new(true) };
    static RENDER_TIMER: Cell<isize> = const { Cell::new(0) };
    static TIME_FREQUENCY: Cell<i64> = const { Cell::new(1) };
    static CURSOR_VISIBLE: Cell<bool> = const { Cell::new(false) };
    static CURSOR_ICON: Cell<Option<kit::CursorIcon>> = const { Cell::new(None) };
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

pub(crate) fn enter_fullscreen(hwnd: isize) -> bool {
    if hwnd == 0 {
        return false;
    }

    const MONITOR_DEFAULTTONEAREST: u32 = 2;
    let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
    let mut info = NativeMonitorInfo {
        cb_size: core::mem::size_of::<NativeMonitorInfo>() as u32,
        monitor: NativeRect {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
        work: NativeRect {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        },
        flags: 0,
    };
    if monitor == 0 || unsafe { GetMonitorInfoW(monitor, &mut info) } == 0 {
        return false;
    }
    let rect = info.monitor;
    let width = rect.right - rect.left;
    let height = rect.bottom - rect.top;
    if width <= 0 || height <= 0 {
        return false;
    }

    let style = unsafe { GetWindowLongW(hwnd, GWL_STYLE) };
    let fullscreen_style = (style & !(WS_CAPTION | WS_THICKFRAME)) | WS_POPUP | WS_VISIBLE;
    if fullscreen_style != style {
        unsafe {
            SetWindowLongW(hwnd, GWL_STYLE, fullscreen_style);
        }
    }
    unsafe {
        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            rect.left,
            rect.top,
            width,
            height,
            SWP_SHOWWINDOW | SWP_FRAMECHANGED,
        ) != 0
    }
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

pub(crate) fn seconds_to_next_wall_second() -> f64 {
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
    (1.0 - f64::from(time.milliseconds) / 1000.0).clamp(0.001, 1.0)
}

pub(crate) fn high_resolution_time() -> f64 {
    let mut value = 0i64;
    if unsafe { QueryPerformanceCounter(&mut value) } == 0 {
        return 0.0;
    }
    value as f64 / TIME_FREQUENCY.with(Cell::get).max(1) as f64
}

fn init_time_source() {
    let mut frequency = 0i64;
    if unsafe { QueryPerformanceFrequency(&mut frequency) } != 0 && frequency > 0 {
        TIME_FREQUENCY.with(|value| value.set(frequency));
    }
}

pub(crate) fn client_origin(hwnd: isize) -> Option<(i32, i32)> {
    if hwnd == 0 {
        return None;
    }
    let mut p = Point { x: 0, y: 0 };
    let ok = unsafe { ClientToScreen(hwnd, &mut p) };
    (ok != 0).then_some((p.x, p.y))
}

pub(crate) fn mouse_feel() -> crate::platform::pointer::PointerFeel {
    const SPI_GETMOUSE: u32 = 0x0003;
    const SPI_GETMOUSESPEED: u32 = 0x0070;
    let mut speed = 10;
    let mut mouse = [6, 10, 1];
    unsafe {
        SystemParametersInfoW(
            SPI_GETMOUSESPEED,
            0,
            &mut speed as *mut i32 as *mut core::ffi::c_void,
            0,
        );
        SystemParametersInfoW(
            SPI_GETMOUSE,
            0,
            mouse.as_mut_ptr() as *mut core::ffi::c_void,
            0,
        );
    }
    crate::platform::pointer::PointerFeel::from_system(speed, mouse)
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

pub(crate) const MB_ICONERROR: u32 = 0x0000_0010;
pub(crate) const MB_OK: u32 = 0x0000_0000;
pub(crate) const WS_CAPTION: i32 = 0x00C0_0000;
pub(crate) const WS_THICKFRAME: i32 = 0x0004_0000;
pub(crate) const WS_POPUP: i32 = 0x8000_0000u32 as i32;
pub(crate) const WS_VISIBLE: i32 = 0x1000_0000;
pub(crate) const SWP_FRAMECHANGED: u32 = 0x0020;
pub(crate) const SWP_SHOWWINDOW: u32 = 0x0040;
pub(crate) const HWND_TOPMOST: isize = -1;

pub(crate) const WM_NCCALCSIZE: u32 = 0x0083;
const WM_CLOSE: u32 = 0x0010;
const WM_DESTROY: u32 = 0x0002;
const WM_PAINT: u32 = 0x000F;
const WM_ERASEBKGND: u32 = 0x0014;
const WM_SIZE: u32 = 0x0005;
const WM_TIMER: u32 = 0x0113;
const WM_MOUSEMOVE: u32 = 0x0200;
const WM_INPUT: u32 = 0x00FF;
const WM_LBUTTONDOWN: u32 = 0x0201;
const WM_LBUTTONUP: u32 = 0x0202;
const WM_LBUTTONDBLCLK: u32 = 0x0203;
const WM_RBUTTONDOWN: u32 = 0x0204;
const WM_RBUTTONUP: u32 = 0x0205;
const WM_RBUTTONDBLCLK: u32 = 0x0206;
const WM_MBUTTONDOWN: u32 = 0x0207;
const WM_MBUTTONUP: u32 = 0x0208;
const WM_MOUSEWHEEL: u32 = 0x020A;
const WM_KEYDOWN: u32 = 0x0100;
const WM_KEYUP: u32 = 0x0101;
const WM_SYSKEYDOWN: u32 = 0x0104;
const WM_SYSKEYUP: u32 = 0x0105;
const WM_CHAR: u32 = 0x0102;
const WM_SETFOCUS: u32 = 0x0007;
const WM_KILLFOCUS: u32 = 0x0008;
const WM_SETCURSOR: u32 = 0x0020;
const WM_IME_STARTCOMPOSITION: u32 = 0x010D;
const WM_IME_ENDCOMPOSITION: u32 = 0x010E;
const WM_IME_COMPOSITION: u32 = 0x010F;
const SIZE_MINIMIZED: u32 = 1;
const PM_REMOVE: u32 = 0x0001;
const SW_SHOW: i32 = 5;
const SW_MINIMIZE: i32 = 6;
const SW_RESTORE: i32 = 9;
const RUN_WS_POPUP: u32 = 0x8000_0000;
const RUN_WS_VISIBLE: u32 = 0x1000_0000;
const COLOR_WINDOW: i32 = 5;
const CF_UNICODETEXT_VALUE: u32 = 13;
const GMEM_MOVEABLE: u32 = 0x0002;
const GCS_COMPSTR: u32 = 0x0008;
const GCS_RESULTSTR: u32 = 0x0800;
const CFS_POINT: u32 = 0x0002;
const CFS_CANDIDATEPOS: u32 = 0x0040;
const DIB_RGB_COLORS: u32 = 0;
const IDC_ARROW: usize = 32512;
const IDC_IBEAM: usize = 32513;
const IDC_SIZENWSE: usize = 32642;
const IDC_SIZENESW: usize = 32643;
const IDC_SIZEWE: usize = 32644;
const IDC_SIZENS: usize = 32645;
const IDC_SIZEALL: usize = 32646;
const CREATE_WAITABLE_TIMER_HIGH_RESOLUTION: u32 = 0x0000_0002;
const TIMER_ALL_ACCESS: u32 = 0x001F_0003;
const QS_ALLINPUT: u32 = 0x04FF;
const WAIT_OBJECT_0: u32 = 0;
const RID_INPUT: u32 = 0x1000_0003;
const RIM_TYPE_MOUSE: u32 = 0;
const MOUSE_MOVE_ABSOLUTE: u16 = 0x0001;
const HID_USAGE_PAGE_GENERIC: u16 = 1;
const HID_USAGE_GENERIC_MOUSE: u16 = 2;

fn arm_render_timer_after(delay: std::time::Duration) {
    let timer = RENDER_TIMER.with(Cell::get);
    if timer == 0 {
        return;
    }
    let due = -(delay.as_nanos().max(1).div_ceil(100) as i64);
    unsafe {
        SetWaitableTimer(timer, &due, 0, 0, std::ptr::null_mut(), 0);
    }
}

fn disarm_render_timer() {
    let timer = RENDER_TIMER.with(Cell::get);
    if timer == 0 {
        return;
    }
    unsafe {
        SetWaitableTimer(timer, std::ptr::null(), 0, 0, std::ptr::null_mut(), 0);
    }
}

unsafe extern "system" fn rt_wndproc(hwnd: isize, msg: u32, wp: usize, lp: isize) -> isize {
    match msg {
        WM_NCCALCSIZE if wp != 0 => return 0,
        WM_ERASEBKGND => return 1,
        WM_SETCURSOR => {
            let cursor = if CURSOR_VISIBLE.with(Cell::get) {
                let icon = CURSOR_ICON.with(Cell::get).unwrap_or_default();
                unsafe { LoadCursorW(0, system_cursor_id(icon) as *const u16) }
            } else {
                0
            };
            unsafe { SetCursor(cursor) };
            return 1;
        }
        WM_PAINT => {
            unsafe { ValidateRect(hwnd, std::ptr::null()) };
            REPAINT_ALL.with(|repaint| repaint.set(true));
            EVENT_PENDING.with(|pending| pending.set(true));
            URGENT_PENDING.with(|pending| pending.set(true));
            return 0;
        }
        WM_SIZE => {
            if wp as u32 == SIZE_MINIMIZED {
                MINIMIZED.with(|value| value.set(true));
                disarm_render_timer();
                EVENT_PENDING.with(|pending| pending.set(false));
                return 0;
            }
            MINIMIZED.with(|value| value.set(false));
            let mut rect = NativeRect {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            unsafe { GetClientRect(hwnd, &mut rect) };
            CLIENT_SIZE.with(|size| size.set((rect.right - rect.left, rect.bottom - rect.top)));
            EVENT_PENDING.with(|pending| pending.set(true));
            URGENT_PENDING.with(|pending| pending.set(true));
            return 0;
        }
        WM_TIMER => {
            if wp == 2 {
                unsafe { KillTimer(hwnd, 2) };
            }
            EVENT_PENDING.with(|pending| pending.set(true));
            return 0;
        }
        WM_MOUSEMOVE => {
            push_event(Event::PointerMoved(point_from(lp)));
            return 0;
        }
        WM_INPUT => {
            push_raw_mouse(lp);
            return unsafe { DefWindowProcW(hwnd, msg, wp, lp) };
        }
        WM_LBUTTONDOWN | WM_LBUTTONDBLCLK => {
            push_button(point_from(lp), PointerButton::Primary, true);
            return 0;
        }
        WM_LBUTTONUP => {
            push_button(point_from(lp), PointerButton::Primary, false);
            return 0;
        }
        WM_RBUTTONDOWN | WM_RBUTTONDBLCLK => {
            push_button(point_from(lp), PointerButton::Secondary, true);
            return 0;
        }
        WM_RBUTTONUP => {
            push_button(point_from(lp), PointerButton::Secondary, false);
            return 0;
        }
        WM_MBUTTONDOWN => {
            push_button(point_from(lp), PointerButton::Middle, true);
            return 0;
        }
        WM_MBUTTONUP => {
            push_button(point_from(lp), PointerButton::Middle, false);
            return 0;
        }
        WM_MOUSEWHEEL => {
            let mut point = Point {
                x: low_word(lp),
                y: high_word(lp),
            };
            unsafe { ScreenToClient(hwnd, &mut point) };
            push_event(Event::PointerMoved(pos2(point.x as f32, point.y as f32)));
            let delta = ((wp >> 16) as u16 as i16) as f32 / 120.0;
            push_event(Event::MouseWheel {
                unit: MouseWheelUnit::Line,
                delta: vec2(0.0, delta),
                modifiers: modifiers(),
            });
            unsafe { SetTimer(hwnd, 2, 350, 0) };
            return 0;
        }
        WM_KEYDOWN if wp == 0x56 && modifiers().ctrl => {
            if let Some(text) = clipboard_text() {
                push_event(Event::Paste(text));
            }
            return 0;
        }
        WM_KEYDOWN | WM_SYSKEYDOWN | WM_KEYUP | WM_SYSKEYUP => {
            if let Some(key) = map_key(wp) {
                let pressed = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
                push_event(Event::Key {
                    key,
                    physical_key: Some(key),
                    pressed,
                    repeat: pressed && (lp & (1 << 30)) != 0,
                    modifiers: modifiers(),
                });
            }
            return unsafe { DefWindowProcW(hwnd, msg, wp, lp) };
        }
        WM_CHAR => {
            if let Some(ch) = char::from_u32(wp as u32)
                && !ch.is_control()
            {
                push_event(Event::Text(ch.to_string()));
            }
            return 0;
        }
        WM_IME_STARTCOMPOSITION => {
            push_event(Event::Ime(ImeEvent::Preedit {
                text: String::new(),
                active_range_chars: None,
            }));
            return 0;
        }
        WM_IME_ENDCOMPOSITION => {
            push_event(Event::Ime(ImeEvent::Preedit {
                text: String::new(),
                active_range_chars: None,
            }));
            return 0;
        }
        WM_IME_COMPOSITION => {
            if lp as u32 & GCS_RESULTSTR != 0
                && let Some(text) = composition_string(hwnd, GCS_RESULTSTR)
            {
                push_event(Event::Ime(ImeEvent::Commit(text)));
            }
            if lp as u32 & GCS_COMPSTR != 0
                && let Some(text) = composition_string(hwnd, GCS_COMPSTR)
            {
                push_event(Event::Ime(ImeEvent::Preedit {
                    text,
                    active_range_chars: None,
                }));
            }
            return 0;
        }
        WM_SETFOCUS => {
            FOCUSED.with(|value| value.set(true));
            push_event(Event::WindowFocused(true));
            EVENT_PENDING.with(|pending| pending.set(true));
            URGENT_PENDING.with(|pending| pending.set(true));
            return 0;
        }
        WM_KILLFOCUS => {
            FOCUSED.with(|value| value.set(false));
            disarm_render_timer();
            push_event(Event::WindowFocused(false));
            return 0;
        }
        WM_CLOSE => {
            unsafe { DestroyWindow(hwnd) };
            return 0;
        }
        WM_DESTROY => {
            QUIT.with(|quit| quit.set(true));
            unsafe { PostQuitMessage(0) };
            return 0;
        }
        _ => {}
    }
    unsafe { DefWindowProcW(hwnd, msg, wp, lp) }
}

fn point_from(value: isize) -> kit::Pos2 {
    pos2(low_word(value) as f32, high_word(value) as f32)
}

fn push_raw_mouse(input: isize) {
    let header_size = std::mem::size_of::<RawInputHeader>() as u32;
    let required = header_size as usize + std::mem::size_of::<RawInputMouse>();
    let mut bytes = [0u8; 64];
    if required > bytes.len() {
        return;
    }
    let mut size = bytes.len() as u32;
    let got = unsafe {
        GetRawInputData(
            input,
            RID_INPUT,
            bytes.as_mut_ptr() as *mut core::ffi::c_void,
            &mut size,
            header_size,
        )
    };
    if got < required as u32 || size as usize > bytes.len() {
        return;
    }
    let data = &bytes[..size as usize];
    let header = unsafe { std::ptr::read_unaligned(data.as_ptr() as *const RawInputHeader) };
    if header.kind != RIM_TYPE_MOUSE {
        return;
    }
    let mouse = unsafe {
        std::ptr::read_unaligned(
            data.as_ptr()
                .add(header_size as usize)
                .cast::<RawInputMouse>(),
        )
    };
    if mouse.flags & MOUSE_MOVE_ABSOLUTE != 0 || (mouse.last_x == 0 && mouse.last_y == 0) {
        return;
    }
    push_event(Event::MouseMoved(vec2(
        mouse.last_x as f32,
        mouse.last_y as f32,
    )));
}

fn low_word(value: isize) -> i32 {
    (value as u32 & 0xFFFF) as u16 as i16 as i32
}

fn high_word(value: isize) -> i32 {
    ((value as u32 >> 16) & 0xFFFF) as u16 as i16 as i32
}

fn modifiers() -> Modifiers {
    let value = Modifiers {
        alt: unsafe { GetKeyState(0x12) } < 0,
        ctrl: unsafe { GetKeyState(0x11) } < 0,
        shift: unsafe { GetKeyState(0x10) } < 0,
        mac_cmd: false,
        command: unsafe { GetKeyState(0x11) } < 0,
    };
    EVENT_MODIFIERS.with(|current| current.set(value));
    value
}

fn push_button(pos: kit::Pos2, button: PointerButton, pressed: bool) {
    push_event(Event::PointerButton {
        pos,
        button,
        pressed,
        modifiers: modifiers(),
    });
}

fn push_event(event: Event) {
    let urgent = matches!(
        event,
        Event::PointerButton { .. }
            | Event::MouseWheel { .. }
            | Event::Key { .. }
            | Event::Text(_)
            | Event::Paste(_)
            | Event::Ime(_)
            | Event::WindowFocused(_)
    );
    EVENT_QUEUE.with(|queue| queue.borrow_mut().push(event));
    EVENT_PENDING.with(|pending| pending.set(true));
    if urgent {
        URGENT_PENDING.with(|pending| pending.set(true));
    }
}

fn map_key(value: usize) -> Option<Key> {
    Some(match value {
        0x08 => Key::Backspace,
        0x09 => Key::Tab,
        0x0D => Key::Enter,
        0x1B => Key::Escape,
        0x20 => Key::Space,
        0x23 => Key::End,
        0x24 => Key::Home,
        0x25 => Key::ArrowLeft,
        0x26 => Key::ArrowUp,
        0x27 => Key::ArrowRight,
        0x28 => Key::ArrowDown,
        0x2E => Key::Delete,
        0x41 => Key::A,
        0x43 => Key::C,
        0x53 => Key::S,
        0x58 => Key::X,
        0x59 => Key::Y,
        0x5A => Key::Z,
        _ => return None,
    })
}

fn composition_string(hwnd: isize, index: u32) -> Option<String> {
    unsafe {
        let imc = ImmGetContext(hwnd);
        if imc == 0 {
            return None;
        }
        let bytes = ImmGetCompositionStringW(imc, index, std::ptr::null_mut(), 0);
        if bytes < 0 {
            ImmReleaseContext(hwnd, imc);
            return None;
        }
        let count = bytes as usize / 2;
        let mut buffer = vec![0u16; count];
        let got = ImmGetCompositionStringW(
            imc,
            index,
            buffer.as_mut_ptr() as *mut core::ffi::c_void,
            bytes as u32,
        );
        ImmReleaseContext(hwnd, imc);
        if got < 0 {
            return None;
        }
        Some(String::from_utf16_lossy(&buffer[..got as usize / 2]))
    }
}

fn set_ime_rect(hwnd: isize, io: &kit::output::IMEOutput) {
    let cursor = (
        io.cursor_rect.min.x.round() as i32,
        io.cursor_rect.min.y.round() as i32,
        io.cursor_rect.max.x.round() as i32,
        io.cursor_rect.max.y.round() as i32,
    );
    if IME_CURSOR.with(|current| current.get()) == Some(cursor) {
        return;
    }
    IME_CURSOR.with(|current| current.set(Some(cursor)));
    unsafe {
        let imc = ImmGetContext(hwnd);
        if imc == 0 {
            return;
        }
        ImmSetCompositionWindow(
            imc,
            &CompositionForm {
                style: CFS_POINT,
                point: Point {
                    x: cursor.0,
                    y: cursor.3,
                },
                area: NativeRect::default(),
            },
        );
        ImmSetCandidateWindow(
            imc,
            &CandidateForm {
                index: 0,
                style: CFS_CANDIDATEPOS,
                point: Point {
                    x: cursor.0,
                    y: cursor.3,
                },
                area: NativeRect::default(),
            },
        );
        ImmReleaseContext(hwnd, imc);
    }
}

fn copy_clipboard(hwnd: isize, text: &str) {
    let wide: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        if OpenClipboard(hwnd) == 0 {
            return;
        }
        EmptyClipboard();
        let bytes = wide.len() * 2;
        let memory = GlobalAlloc(GMEM_MOVEABLE, bytes);
        if memory != 0 {
            let target = GlobalLock(memory);
            if !target.is_null() {
                std::ptr::copy_nonoverlapping(wide.as_ptr() as *const u8, target, bytes);
                GlobalUnlock(memory);
                SetClipboardData(CF_UNICODETEXT_VALUE, memory);
            }
        }
        CloseClipboard();
    }
}

fn apply_textures(
    textures: &mut std::collections::HashMap<TextureId, ColorImage>,
    output: &kit::FullOutput,
) {
    for (id, images) in &output.textures_delta.set {
        for delta in images {
            let ImageData::Color(image) = &delta.image;
            if let Some(pos) = delta.pos {
                if let Some(target) = textures.get_mut(id) {
                    for y in 0..image.size[1] {
                        for x in 0..image.size[0] {
                            let dx = pos[0] + x;
                            let dy = pos[1] + y;
                            if dx < target.size[0] && dy < target.size[1] {
                                target[(dx, dy)] = image[(x, y)];
                            }
                        }
                    }
                }
            } else {
                textures.insert(*id, (**image).clone());
            }
        }
    }
    for id in &output.textures_delta.free {
        textures.remove(id);
    }
}

fn present(hwnd: isize, canvas: &Canvas, dirty: &[Rect]) {
    let dc = unsafe { GetDC(hwnd) };
    if dc != 0 {
        present_to_dc(dc, canvas, dirty);
        unsafe { ReleaseDC(hwnd, dc) };
    }
}

fn present_to_dc(dc: isize, canvas: &Canvas, dirty: &[Rect]) {
    if dirty.is_empty() || canvas.width() == 0 || canvas.height() == 0 {
        return;
    }
    let mut info = NativeBitmapInfo::default();
    info.header.size = std::mem::size_of::<NativeBitmapHeader>() as u32;
    info.header.width = canvas.width() as i32;
    info.header.height = -(canvas.height() as i32);
    info.header.planes = 1;
    info.header.bit_count = 32;
    info.header.compression = 0;
    for dirty in dirty {
        let x = dirty.min.x.floor().max(0.0) as i32;
        let y = dirty.min.y.floor().max(0.0) as i32;
        let right = dirty.max.x.ceil().min(canvas.width() as f32) as i32;
        let bottom = dirty.max.y.ceil().min(canvas.height() as f32) as i32;
        let width = right - x;
        let height = bottom - y;
        if width <= 0 || height <= 0 {
            continue;
        }
        unsafe {
            let bits = canvas.pixels().as_ptr().add(y as usize * canvas.width()) as *const u8;
            SetDIBitsToDevice(
                dc,
                x,
                y,
                width as u32,
                height as u32,
                x,
                0,
                0,
                height as u32,
                bits,
                &info,
                DIB_RGB_COLORS,
            );
        }
    }
}

fn apply_system_cursor(icon: kit::CursorIcon) {
    let visible = icon != kit::CursorIcon::None;
    let changed = CURSOR_ICON.with(|state| state.replace(Some(icon))) != Some(icon);
    CURSOR_VISIBLE.with(|state| state.set(visible));
    if !changed {
        return;
    }
    let cursor = if visible {
        unsafe { LoadCursorW(0, system_cursor_id(icon) as *const u16) }
    } else {
        0
    };
    unsafe {
        SetCursor(cursor);
    }
}

fn system_cursor_id(icon: kit::CursorIcon) -> usize {
    match icon {
        kit::CursorIcon::Text => IDC_IBEAM,
        kit::CursorIcon::Move => IDC_SIZEALL,
        kit::CursorIcon::ResizeVertical => IDC_SIZENS,
        kit::CursorIcon::ResizeHorizontal => IDC_SIZEWE,
        kit::CursorIcon::ResizeNwSe => IDC_SIZENWSE,
        kit::CursorIcon::ResizeNeSw => IDC_SIZENESW,
        kit::CursorIcon::Default | kit::CursorIcon::None => IDC_ARROW,
    }
}

pub(crate) fn run_app() -> Result<(), String> {
    init_time_source();
    let (_, _, width, height) =
        screen_area().ok_or_else(|| "Cannot read the screen size".to_owned())?;
    let instance = unsafe { GetModuleHandleW(std::ptr::null()) };
    let class_name = wide("RoundTableNative");
    let title = wide("RoundTable");
    let class = WindowClassW {
        style: 0,
        wndproc: rt_wndproc,
        class_extra: 0,
        window_extra: 0,
        instance,
        icon: 0,
        cursor: 0,
        background: (COLOR_WINDOW + 1) as isize,
        menu_name: std::ptr::null(),
        class_name: class_name.as_ptr(),
    };
    if unsafe { RegisterClassW(&class) } == 0 && unsafe { GetLastError() } != 1410 {
        return Err("Failed to register the window class".to_owned());
    }
    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            RUN_WS_POPUP | RUN_WS_VISIBLE,
            0,
            0,
            width,
            height,
            0,
            0,
            instance,
            std::ptr::null_mut(),
        )
    };
    if hwnd == 0 {
        return Err("Failed to create the window".to_owned());
    }
    unsafe {
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
    }
    let raw_mouse = RawInputDevice {
        usage_page: HID_USAGE_PAGE_GENERIC,
        usage: HID_USAGE_GENERIC_MOUSE,
        flags: 0,
        target: hwnd,
    };
    unsafe {
        RegisterRawInputDevices(&raw_mouse, 1, std::mem::size_of::<RawInputDevice>() as u32);
    }
    let render_timer = unsafe {
        CreateWaitableTimerExW(
            std::ptr::null_mut(),
            std::ptr::null(),
            CREATE_WAITABLE_TIMER_HIGH_RESOLUTION,
            TIMER_ALL_ACCESS,
        )
    };
    if render_timer == 0 {
        return Err("Failed to create the frame timer".to_owned());
    }
    RENDER_TIMER.with(|value| value.set(render_timer));
    let mut client = NativeRect::default();
    unsafe { GetClientRect(hwnd, &mut client) };
    CLIENT_SIZE.with(|size| size.set((client.right - client.left, client.bottom - client.top)));
    let context = kit::Context::default();
    let mut app = crate::application::app::StockApp::new(&context, hwnd);
    let mut canvas = Canvas::default();
    let mut textures = std::collections::HashMap::new();
    let mut first = true;
    let mut dirty_rects = Vec::new();
    loop {
        let mut message = MessageW::default();
        while unsafe { PeekMessageW(&mut message, 0, 0, 0, PM_REMOVE) } > 0 {
            if message.message == 0x0012 {
                QUIT.with(|quit| quit.set(true));
                break;
            }
            unsafe {
                TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
        if QUIT.with(Cell::get) {
            break;
        }
        let active = !MINIMIZED.with(Cell::get) && FOCUSED.with(Cell::get);
        let pending = EVENT_PENDING.with(|value| value.replace(false));
        let urgent = URGENT_PENDING.with(|value| value.replace(false));
        if active && (first || pending || urgent) {
            let now = high_resolution_time();
            let due = app.next_input_repaint_at(now);
            if !first && now + 0.0005 < due {
                EVENT_PENDING.with(|value| value.set(true));
                arm_render_timer_after(std::time::Duration::from_secs_f64(due - now));
            } else {
                first = false;
                let events = EVENT_QUEUE.with(|queue| std::mem::take(&mut *queue.borrow_mut()));
                let (width, height) = CLIENT_SIZE.with(Cell::get);
                canvas.resize(width.max(1) as usize, height.max(1) as usize);
                let screen = Rect::from_min_size(
                    pos2(0.0, 0.0),
                    vec2(width.max(1) as f32, height.max(1) as f32),
                );
                let repaint_all = REPAINT_ALL.with(|repaint| repaint.replace(false));
                let mut raw = kit::RawInput {
                    screen_rect: Some(screen),
                    time: Some(now),
                    events,
                    modifiers: EVENT_MODIFIERS.with(Cell::get),
                };
                if repaint_all {
                    canvas.invalidate();
                }
                app.lock_pointer_events(&mut raw);
                let output = context.run_ui(raw, |ui| app.ui(ui));
                let mut recycled_events = context.take_input_events();
                recycled_events.clear();
                EVENT_QUEUE.with(|queue| *queue.borrow_mut() = recycled_events);
                let mut close = false;
                for command in &output.platform_output.commands {
                    match command {
                        kit::ViewportCommand::Close => {
                            close = true;
                            unsafe { DestroyWindow(hwnd) };
                        }
                        kit::ViewportCommand::Minimized(minimized) => {
                            unsafe {
                                ShowWindow(hwnd, if *minimized { SW_MINIMIZE } else { SW_RESTORE })
                            };
                        }
                    }
                }
                if close {
                    QUIT.with(|quit| quit.set(true));
                    break;
                }
                apply_system_cursor(output.platform_output.cursor_icon);
                apply_textures(&mut textures, &output);
                if output
                    .textures_delta
                    .set
                    .values()
                    .flatten()
                    .any(|delta| delta.pos.is_none())
                {
                    canvas.invalidate();
                }
                canvas.draw(&output.shapes, &textures);
                if let Some(text) = &output.platform_output.copied_text {
                    copy_clipboard(hwnd, text);
                }
                if let Some(ime) = &output.platform_output.ime {
                    set_ime_rect(hwnd, ime);
                }
                canvas.take_dirty_into(&mut dirty_rects);
                if !dirty_rects.is_empty() {
                    present(hwnd, &canvas, &dirty_rects);
                }
                let finished = high_resolution_time();
                let next_repaint_at = app.next_repaint_at(finished);
                arm_render_timer_after(std::time::Duration::from_secs_f64(
                    (next_repaint_at - finished).max(0.001),
                ));
            }
        } else if !active {
            disarm_render_timer();
        }
        let wait =
            unsafe { MsgWaitForMultipleObjectsEx(1, &render_timer, u32::MAX, QS_ALLINPUT, 0) };
        if wait == WAIT_OBJECT_0 && !MINIMIZED.with(Cell::get) && FOCUSED.with(Cell::get) {
            EVENT_PENDING.with(|value| value.set(true));
        }
    }
    unsafe {
        KillTimer(hwnd, 1);
        KillTimer(hwnd, 2);
        CloseHandle(render_timer);
    }
    RENDER_TIMER.with(|value| value.set(0));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_mouse_structures_match_the_win32_layout() {
        assert_eq!(std::mem::size_of::<RawInputMouse>(), 24);
        assert_eq!(
            std::mem::offset_of!(RawInputMouse, last_x),
            std::mem::size_of::<u32>() * 3
        );
        #[cfg(target_pointer_width = "64")]
        {
            assert_eq!(std::mem::size_of::<RawInputHeader>(), 24);
            assert_eq!(std::mem::size_of::<RawInputDevice>(), 16);
        }
        #[cfg(target_pointer_width = "32")]
        {
            assert_eq!(std::mem::size_of::<RawInputHeader>(), 16);
            assert_eq!(std::mem::size_of::<RawInputDevice>(), 12);
        }
    }

    #[test]
    fn dirty_rectangles_blit_from_the_full_canvas_without_a_subset_copy() {
        let mut canvas = Canvas::default();
        canvas.resize(64, 64);
        let all = Rect::from_min_size(pos2(0.0, 0.0), vec2(64.0, 64.0));
        let color = kit::Color32::from_rgb(0x12, 0x34, 0x56);
        canvas.fill_rect(
            Rect::from_min_size(pos2(40.0, 57.0), vec2(2.0, 2.0)),
            color,
            all,
        );
        for y in 32..64 {
            canvas.fill_rect(
                Rect::from_min_size(pos2(32.0, y as f32), vec2(1.0, 1.0)),
                kit::Color32::from_rgb(y as u8, (y * 3) as u8, 255 - y as u8),
                all,
            );
        }
        let dirty = canvas.take_dirty();

        let mut target_info = crate::engine::font::BitMapInfo::default();
        target_info.header.size =
            std::mem::size_of::<crate::engine::font::BitMapInfoHeader>() as u32;
        target_info.header.width = 64;
        target_info.header.height = -64;
        target_info.header.planes = 1;
        target_info.header.bit_count = 32;
        let mut bits: *mut u8 = std::ptr::null_mut();
        unsafe {
            let dc = crate::engine::font::CreateCompatibleDC(0);
            assert_ne!(dc, 0);
            let bitmap = crate::engine::font::CreateDIBSection(
                dc,
                &target_info,
                DIB_RGB_COLORS,
                &mut bits,
                0,
                0,
            );
            assert_ne!(bitmap, 0);
            let previous = crate::engine::font::SelectObject(dc, bitmap);
            present_to_dc(dc, &canvas, &dirty);
            let pixels = std::slice::from_raw_parts(bits.cast::<u32>(), 64 * 64);
            for y in 32..64 {
                assert_eq!(
                    pixels[y * 64 + 32],
                    canvas.pixels()[y * 64 + 32],
                    "dirty blit shifted source scan line {y}"
                );
            }
            assert_eq!(pixels[57 * 64 + 40], color.to_u32());
            assert_eq!(pixels[56 * 64 + 40], 0);
            crate::engine::font::SelectObject(dc, previous);
            crate::engine::font::DeleteObject(bitmap);
            crate::engine::font::DeleteDC(dc);
        }
    }

    #[test]
    #[ignore]
    fn bench_present_dirty_1080p() {
        let width = 1920usize;
        let height = 1080usize;
        let mut canvas = Canvas::default();
        canvas.resize(width, height);
        let all = Rect::from_min_size(pos2(0.0, 0.0), vec2(width as f32, height as f32));
        canvas.fill_rect(all, kit::Color32::from_rgb(0xEE, 0xEE, 0xEE), all);
        let _ = canvas.take_dirty();

        let mut target_info = crate::engine::font::BitMapInfo::default();
        target_info.header.size =
            std::mem::size_of::<crate::engine::font::BitMapInfoHeader>() as u32;
        target_info.header.width = width as i32;
        target_info.header.height = -(height as i32);
        target_info.header.planes = 1;
        target_info.header.bit_count = 32;
        let mut bits: *mut u8 = std::ptr::null_mut();
        unsafe {
            let dc = crate::engine::font::CreateCompatibleDC(0);
            let bitmap = crate::engine::font::CreateDIBSection(
                dc,
                &target_info,
                DIB_RGB_COLORS,
                &mut bits,
                0,
                0,
            );
            let previous = crate::engine::font::SelectObject(dc, bitmap);
            let n = 200;
            let started = std::time::Instant::now();
            for frame in 0..n {
                let shade = if frame % 2 == 0 { 0x11 } else { 0x22 };
                let color = kit::Color32::from_rgb(shade, shade, shade);
                canvas.fill_rect(
                    Rect::from_min_size(pos2(0.0, 1024.0), vec2(1920.0, 16.0)),
                    color,
                    all,
                );
                canvas.fill_rect(
                    Rect::from_min_size(pos2(800.0, 200.0), vec2(1.0, 600.0)),
                    color,
                    all,
                );
                canvas.fill_rect(
                    Rect::from_min_size(pos2(300.0, 500.0), vec2(1200.0, 1.0)),
                    color,
                    all,
                );
                let dirty = canvas.take_dirty();
                present_to_dc(dc, &canvas, &dirty);
            }
            let elapsed = started.elapsed().as_secs_f64() * 1000.0 / n as f64;
            eprintln!("BENCH present 1080p dirty regions: {elapsed:.3} ms/frame");
            crate::engine::font::SelectObject(dc, previous);
            crate::engine::font::DeleteObject(bitmap);
            crate::engine::font::DeleteDC(dc);
        }
    }
}
