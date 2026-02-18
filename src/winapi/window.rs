//! Windows 窗口 API 安全封装
//!
//! 提供窗口创建、消息处理、窗口管理等功能

use windows::Win32::{
    Foundation::{HWND, WPARAM, LPARAM, LRESULT, COLORREF, HINSTANCE, RECT},
    UI::WindowsAndMessaging::*,
    Graphics::Gdi::*,
    System::LibraryLoader::GetModuleHandleW,
};
use windows::core::PCWSTR;
use std::ptr;

/// 窗口类注册信息
pub struct WindowClassInfo {
    pub class_name: String,
    pub window_proc: WNDPROC,
    pub background_brush: HBRUSH,
    pub cursor: HCURSOR,
}

impl Default for WindowClassInfo {
    fn default() -> Self {
        Self {
            class_name: String::new(),
            window_proc: None,
            background_brush: unsafe { HBRUSH(GetStockObject(BLACK_BRUSH).0) },
            cursor: unsafe { LoadCursorW(None, IDC_ARROW).unwrap_or_default() },
        }
    }
}

/// 窗口创建参数
pub struct WindowCreateInfo {
    pub class_name: String,
    pub window_name: String,
    pub style: WINDOW_STYLE,
    pub ex_style: WINDOW_EX_STYLE,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub create_param: Option<*const core::ffi::c_void>,
}

/// 字体创建信息
pub struct FontInfo {
    pub name: String,
    pub size: i32,
    pub weight: i32,
}

impl Default for FontInfo {
    fn default() -> Self {
        Self {
            name: "Arial".to_string(),
            size: 16,
            weight: 400,
        }
    }
}

/// 绘制文本信息
pub struct DrawTextInfo {
    pub text: Vec<u16>,
    pub rect: RECT,
    pub format: DRAW_TEXT_FORMAT,
}

/// 注册窗口类
///
/// # 参数
///
/// * `info` - 窗口类信息
pub fn register_window_class(info: &WindowClassInfo) -> Result<u16, windows::core::Error> {
    unsafe {
        let class_name_vec: Vec<u16> = info.class_name.encode_utf16().chain(Some(0)).collect();
        let class_name_ptr = PCWSTR::from_raw(class_name_vec.as_ptr());
        
        let hinstance = GetModuleHandleW(None).unwrap_or_default();
        
        let wc = WNDCLASSW {
            hbrBackground: info.background_brush,
            hCursor: info.cursor,
            hInstance: HINSTANCE(hinstance.0),
            lpszClassName: class_name_ptr,
            lpfnWndProc: info.window_proc,
            ..Default::default()
        };
        
        let class_atom = RegisterClassW(&wc);
        if class_atom == 0 {
            Err(windows::core::Error::from_win32())
        } else {
            Ok(class_atom)
        }
    }
}

/// 创建窗口
///
/// # 参数
///
/// * `info` - 窗口创建参数
pub fn create_window(info: &WindowCreateInfo) -> Result<HWND, windows::core::Error> {
    unsafe {
        let class_name_vec: Vec<u16> = info.class_name.encode_utf16().chain(Some(0)).collect();
        let class_name_ptr = PCWSTR::from_raw(class_name_vec.as_ptr());
        
        let window_name_vec: Vec<u16> = info.window_name.encode_utf16().chain(Some(0)).collect();
        let window_name_ptr = PCWSTR::from_raw(window_name_vec.as_ptr());
        
        let hinstance = GetModuleHandleW(None).unwrap_or_default();
        
        CreateWindowExW(
            info.ex_style,
            class_name_ptr,
            window_name_ptr,
            info.style,
            info.x,
            info.y,
            info.width,
            info.height,
            None,
            None,
            hinstance,
            info.create_param,
        )
    }
}

/// 发送关闭窗口消息（异步）
///
/// # 参数
///
/// * `hwnd_value` - 窗口句柄值
pub fn post_close_message(hwnd_value: isize) -> Result<(), windows::core::Error> {
    unsafe {
        let hwnd = HWND(hwnd_value as *mut core::ffi::c_void);
        PostMessageW(hwnd, WM_CLOSE, WPARAM(0), LPARAM(0))?;
        Ok(())
    }
}

/// 销毁窗口
///
/// # 参数
///
/// * `hwnd` - 窗口句柄
pub fn destroy_window(hwnd: HWND) -> Result<(), windows::core::Error> {
    unsafe {
        DestroyWindow(hwnd)?;
        Ok(())
    }
}

/// 设置窗口位置和大小
///
/// # 参数
///
/// * `hwnd` - 窗口句柄
/// * `x`, `y` - 位置
/// * `width`, `height` - 大小
/// * `flags` - 设置标志
pub fn set_window_position(
    hwnd: HWND,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    flags: SET_WINDOW_POS_FLAGS,
) -> Result<(), windows::core::Error> {
    unsafe {
        SetWindowPos(hwnd, HWND_TOPMOST, x, y, width, height, flags)?;
        Ok(())
    }
}

/// 设置窗口透明度
///
/// # 参数
///
/// * `hwnd` - 窗口句柄
/// * `alpha` - 透明度（0-255）
pub fn set_window_alpha(hwnd: HWND, alpha: u8) -> Result<(), windows::core::Error> {
    unsafe {
        SetLayeredWindowAttributes(hwnd, COLORREF(0), alpha, LWA_ALPHA)?;
        Ok(())
    }
}

/// 显示窗口
///
/// # 参数
///
/// * `hwnd` - 窗口句柄
/// * `show_cmd` - 显示命令
pub fn show_window(hwnd: HWND, show_cmd: SHOW_WINDOW_CMD) -> Result<(), windows::core::Error> {
    unsafe {
        let _ = ShowWindow(hwnd, show_cmd);
        Ok(())
    }
}

/// 设置窗口为前台窗口
///
/// # 参数
///
/// * `hwnd` - 窗口句柄
pub fn set_foreground_window(hwnd: HWND) -> bool {
    unsafe {
        SetForegroundWindow(hwnd).as_bool()
    }
}

/// 将窗口带到顶层
///
/// # 参数
///
/// * `hwnd` - 窗口句柄
pub fn bring_window_to_top(hwnd: HWND) -> Result<(), windows::core::Error> {
    unsafe {
        BringWindowToTop(hwnd)?;
        Ok(())
    }
}

/// 获取系统度量值
///
/// # 参数
///
/// * `index` - 度量索引
pub fn get_system_metrics(index: SYSTEM_METRICS_INDEX) -> i32 {
    unsafe {
        GetSystemMetrics(index)
    }
}

/// 创建字体
///
/// # 参数
///
/// * `info` - 字体信息
pub fn create_font(info: &FontInfo) -> Result<HFONT, windows::core::Error> {
    unsafe {
        let name_vec: Vec<u16> = info.name.encode_utf16().chain(Some(0)).collect();
        let hfont = CreateFontW(
            info.size, 0, 0, 0,
            info.weight,
            0, 0, 0,
            1, // DEFAULT_CHARSET
            0, // OUT_DEFAULT_PRECIS
            0, // CLIP_DEFAULT_PRECIS
            0, // DEFAULT_QUALITY
            0, // DEFAULT_PITCH
            PCWSTR::from_raw(name_vec.as_ptr()),
        );
        
        if hfont.0 == ptr::null_mut() {
            Err(windows::core::Error::from_win32())
        } else {
            Ok(hfont)
        }
    }
}

/// 选择对象到设备上下文
///
/// # 参数
///
/// * `hdc` - 设备上下文
/// * `obj` - 要选择的对象
pub fn select_object(hdc: HDC, obj: HGDIOBJ) -> Result<HGDIOBJ, windows::core::Error> {
    unsafe {
        let old_obj = SelectObject(hdc, obj);
        if old_obj.0 == ptr::null_mut() {
            Err(windows::core::Error::from_win32())
        } else {
            Ok(old_obj)
        }
    }
}

/// 删除 GDI 对象
///
/// # 参数
///
/// * `obj` - 要删除的对象
pub fn delete_object(obj: HGDIOBJ) -> Result<(), windows::core::Error> {
    unsafe {
        let result = DeleteObject(obj);
        if result.as_bool() {
            Ok(())
        } else {
            Err(windows::core::Error::from_win32())
        }
    }
}

/// 设置文本颜色
///
/// # 参数
///
/// * `hdc` - 设备上下文
/// * `color` - 颜色值
pub fn set_text_color(hdc: HDC, color: COLORREF) -> COLORREF {
    unsafe {
        SetTextColor(hdc, color)
    }
}

/// 设置背景模式
///
/// # 参数
///
/// * `hdc` - 设备上下文
/// * `mode` - 背景模式
pub fn set_bk_mode(hdc: HDC, mode: BACKGROUND_MODE) -> i32 {
    unsafe {
        SetBkMode(hdc, mode)
    }
}

/// 绘制文本
///
/// # 参数
///
/// * `hdc` - 设备上下文
/// * `info` - 绘制文本信息
pub fn draw_text(hdc: HDC, info: &mut DrawTextInfo) -> i32 {
    unsafe {
        DrawTextW(hdc, &mut info.text, &mut info.rect, info.format)
    }
}

/// 开始绘制
///
/// # 参数
///
/// * `hwnd` - 窗口句柄
/// * `ps` - 绘制结构
pub fn begin_paint(hwnd: HWND, ps: &mut PAINTSTRUCT) -> Result<HDC, windows::core::Error> {
    unsafe {
        let hdc = BeginPaint(hwnd, ps);
        if hdc.0 == ptr::null_mut() {
            Err(windows::core::Error::from_win32())
        } else {
            Ok(hdc)
        }
    }
}

/// 结束绘制
///
/// # 参数
///
/// * `hwnd` - 窗口句柄
/// * `ps` - 绘制结构
pub fn end_paint(hwnd: HWND, ps: &PAINTSTRUCT) -> Result<(), windows::core::Error> {
    unsafe {
        let result = EndPaint(hwnd, ps);
        if result.as_bool() {
            Ok(())
        } else {
            Err(windows::core::Error::from_win32())
        }
    }
}

/// 设置窗口长指针值
///
/// # 参数
///
/// * `hwnd` - 窗口句柄
/// * `index` - 索引
/// * `value` - 要设置的值
pub fn set_window_long_ptr(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX, value: isize) -> Result<isize, windows::core::Error> {
    unsafe {
        let result = SetWindowLongPtrW(hwnd, index, value);
        Ok(result)
    }
}

/// 获取窗口长指针值
///
/// # 参数
///
/// * `hwnd` - 窗口句柄
/// * `index` - 索引
pub fn get_window_long_ptr(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX) -> isize {
    unsafe {
        GetWindowLongPtrW(hwnd, index)
    }
}

/// 投递退出消息
pub fn post_quit_message(exit_code: i32) {
    unsafe {
        PostQuitMessage(exit_code);
    }
}

/// 默认窗口过程
///
/// # 参数
///
/// * `hwnd` - 窗口句柄
/// * `msg` - 消息
/// * `wparam` - WPARAM
/// * `lparam` - LPARAM
pub fn default_window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}
