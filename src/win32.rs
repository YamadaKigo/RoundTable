// ===========================================================================
//  模块 win32：Windows 专用窗口杂项
// ===========================================================================
//  去掉系统画在窗口最外圈的那层装饰（1px 边框 / 投影），被系统重新套回去时再清掉。
//  本文件 = 拆分前那份单文件里的「十三、去掉系统画在窗口外圈的东西」。
use raw_window_handle::RawWindowHandle;
use std::sync::atomic::{AtomicIsize, Ordering};

// ===========================================================================
//  十三、去掉系统画在窗口外圈的东西（Windows 专用）
// ===========================================================================
//  窗口最外圈原本有两层系统装饰，两层都得自己清：
//    a) 窗口最上面那 1px - 用户看到的那条「奇怪颜色的 1px 边框」就是它。
//       winit 为了让无边框窗口带上投影，在 WM_NCCALCSIZE 里把客户区整体下移 1px
//       （winit 源码注释原话：leads to a small black 1px border on the top）。
//       于是窗口最上面那一行永远不在客户区里：我们自己画不到，露出来的是 DWM 画的
//       1px 窗口边框（颜色取自 Windows 的窗口边框色，本机渲染成一条深灰渐变，
//       两个上角更深）。egui / winit 都没有对应开关，只能在窗口过程里拦 WM_NCCALCSIZE：
//       把非客户区压成 0、客户区矩形 = 整个窗口矩形，窗口 100% 变成我们的画布，
//       最外圈于是只剩我们自己画的那 1px 黑边框（见 install_client_cover）。
//    b) DWM 投影（窗口外面那一圈阴影）。DWM 给「带标题栏样式 WS_CAPTION 的窗口」
//       画投影，而 winit 为了 Aero Snap 一直留着这个样式位；把样式位清掉即可，
//       winit 在最小化/还原时会把样式重新套回去，所以每帧查一次（见 clear_window_frame）。
//  这些是 Windows 专属的窗口细节，用不到 windows-sys，直接声明要用的几个函数。
//  窗口样式是 32 位值，用 GetWindowLongW / SetWindowLongW（32/64 位通吃）；
//  窗口过程是 64 位指针，只能走 SetWindowLongPtrW。
#[link(name = "user32")]
unsafe extern "system" {
    pub(crate) fn GetWindowLongW(hwnd: isize, index: i32) -> i32;
    pub(crate) fn SetWindowLongW(hwnd: isize, index: i32, value: i32) -> i32;
    pub(crate) fn SetWindowLongPtrW(hwnd: isize, index: i32, value: isize) -> isize;
    pub(crate) fn SetWindowPos(hwnd: isize, after: isize, x: i32, y: i32, cx: i32, cy: i32, flags: u32) -> i32;
    pub(crate) fn DefWindowProcW(hwnd: isize, msg: u32, wp: usize, lp: isize) -> isize;
    pub(crate) fn CallWindowProcW(prev: isize, hwnd: isize, msg: u32, wp: usize, lp: isize) -> isize;
    pub(crate) fn OpenClipboard(hwnd: isize) -> i32;
    pub(crate) fn CloseClipboard() -> i32;
    pub(crate) fn IsClipboardFormatAvailable(format: u32) -> i32;
    pub(crate) fn GetClipboardData(format: u32) -> isize;
    pub(crate) fn SystemParametersInfoW(action: u32, param: u32, data: *mut core::ffi::c_void, win_ini: u32) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    pub(crate) fn GlobalLock(handle: isize) -> *mut u8;
    pub(crate) fn GlobalUnlock(handle: isize) -> i32;
}

/// CF_UNICODETEXT：系统剪贴板里的宽字符文本格式。
pub(crate) const CF_UNICODETEXT: u32 = 13;

/// SPI_GETWORKAREA：主显示器上「没被任务栏挡住」的那块工作区。
pub(crate) const SPI_GETWORKAREA: u32 = 0x0030;

/// 主显示器的工作区（左上角 + 宽高，物理像素）：任务栏占掉的那一条不算在内。
/// 开机按它给窗口定位定尺寸，窗口就正好铺满工作区、不会压到任务栏上。
/// 读不到（理论上不会）就返回 None，调用方退回默认尺寸。
pub(crate) fn work_area() -> Option<(i32, i32, i32, i32)> {
    #[repr(C)]
    struct Rect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }
    let mut r = Rect { left: 0, top: 0, right: 0, bottom: 0 };
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

/// 读系统剪贴板里的文本。自绘编辑器 / 输入框的「粘贴」按钮用它
/// （Ctrl+V 不用它 - winit 会直接把内容作为 Event::Paste 送进来）。
/// 读不到（剪贴板被别的进程占着、里面也不是文本）就返回 None，调用方一个字都不动。
pub(crate) fn clipboard_text() -> Option<String> {
    unsafe {
        if IsClipboardFormatAvailable(CF_UNICODETEXT) == 0 {
            return None;
        }
        // 打开剪贴板最多试几次：它同一时刻只允许一个进程拿着。
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
                // 宽字符串：数到第一个 0 为止（另加一个上限兜底，防止对面没给结束符）。
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

/// 被我们挤掉的那个系统窗口过程（其实是 winit 的），其余消息原样转给它。
pub(crate) static PREV_WNDPROC: AtomicIsize = AtomicIsize::new(0);

/// 从 raw-window-handle 的句柄里取出 Win32 的 HWND（拿不到就返回 0）。
pub(crate) fn win32_hwnd(raw: RawWindowHandle) -> isize {
    match raw {
        RawWindowHandle::Win32(h) => h.hwnd.get(),
        _ => 0,
    }
}

/// 清掉会让系统画出投影 / 3D 边 / 边框的那几个样式位，再让窗口重算一次框架。
///
/// 为什么要清：
///   投影 - DWM 会给「带标题栏样式（WS_CAPTION）的窗口」画一圈投影；winit 为了
///   保住 Aero Snap 之类的功能，无边框窗口也一直留着 WS_CAPTION，而 egui-winit
///   在 decorations(false) 时还会主动请求这个阴影（它那个开关只影响客户区顶部
///   1px 的处理，治不了 DWM 本身）。
///   3D 边 - WS_EX_WINDOWEDGE / CLIENTEDGE / STATICEDGE 会让系统沿窗口外圈画一条
///   立体边（左上亮、右下暗）。
/// winit 在最小化/还原这类标志位变化时会把样式重新套回去，所以这里每帧查一次。
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

/// 我们自己的窗口过程：只拦 WM_NCCALCSIZE，把非客户区定成 0。
/// 进来时 lParam 里的 rgrc[0] 就是「这一次要用的窗口矩形」，原样不动 + 返回 0
/// 就等于宣告「客户区 = 整个窗口」。
/// 注意不要去调 GetWindowRect 拿窗口矩形：最小化/还原过程中那个值还是旧位置
/// （最小化时是任务栏图标的位置），拿它算出来的客户区会把整个画面挪到窗口外。
unsafe extern "system" fn rt_wndproc(hwnd: isize, msg: u32, wp: usize, lp: isize) -> isize {
    if msg == WM_NCCALCSIZE && wp != 0 {
        return 0;
    }
    // 其余消息原样转给 winit，别把它的逻辑打断。
    let prev = PREV_WNDPROC.load(Ordering::Relaxed);
    unsafe {
        if prev == 0 {
            DefWindowProcW(hwnd, msg, wp, lp)
        } else {
            CallWindowProcW(prev, hwnd, msg, wp, lp)
        }
    }
}

/// 装上窗口过程钩子，并让系统立刻重算一次非客户区（此后客户区 = 整个窗口）。
pub(crate) fn install_client_cover(hwnd: isize) -> bool {
    if hwnd == 0 {
        return false;
    }
    if PREV_WNDPROC.load(Ordering::Relaxed) != 0 {
        // 已经装过就别再装：重复装会把我们自己的过程当成「上一个」，形成自套自。
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

