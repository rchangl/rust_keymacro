//! 屏幕置顶提示模块
//!
//! 在屏幕中央显示临时状态提示

use std::{
    time::Duration,
    thread,
    sync::{Mutex, Arc, Condvar},
};
use once_cell::sync::Lazy;
use windows::Win32::{
    Foundation::{HWND, WPARAM, LPARAM, LRESULT, COLORREF, RECT},
    UI::WindowsAndMessaging::*,
    Graphics::Gdi::*,
};
use crate::winapi::window;

// 窗口配置
const WINDOW_WIDTH: i32 = 200;
const WINDOW_HEIGHT: i32 = 150;
const DISPLAY_DURATION_MS: u64 = 500;
const WINDOW_ALPHA: u8 = 192;
const FONT_NAME: &str = "Arial";
const FONT_SIZE: i32 = 150;
const FONT_WEIGHT: i32 = 700;
const COLOR_RED: COLORREF = COLORREF(0x000000FF);
const COLOR_GREEN: COLORREF = COLORREF(0x0000FF00);
const CLASS_NAME: &str = "OverlayClass_001";

// 窗口状态
struct WindowState {
    handle: Option<isize>,
    is_closing: bool,
}

static CURRENT_WINDOW: Lazy<Arc<(Mutex<WindowState>, Condvar)>> = Lazy::new(|| {
    Arc::new((
        Mutex::new(WindowState { handle: None, is_closing: false }),
        Condvar::new(),
    ))
});

static WINDOW_CLASS_INIT: std::sync::Once = std::sync::Once::new();

/// 在屏幕中央显示状态提示
///
/// # 参数
///
/// * `text` - 显示的文本（"0" 或 "1"）
///
/// # 说明
///
/// - 显示 0.5 秒后自动消失
/// - 0 显示为红色，1 显示为绿色
/// - 如果已有窗口，会关闭旧窗口后创建新窗口
pub fn show_overlay(text: &str) {
    close_existing_window_async();
    
    let text = text.to_string();
    thread::spawn(move || {
        // 注册窗口类（仅一次）
        WINDOW_CLASS_INIT.call_once(|| {
            let info = window::WindowClassInfo {
                class_name: CLASS_NAME.to_string(),
                window_proc: Some(window_proc),
                ..Default::default()
            };
            
            if let Err(e) = window::register_window_class(&info) {
                eprintln!("[WARN] 注册窗口类失败: {}", e);
            }
        });
        
        // 准备窗口文本和创建参数
        let status_text_vec: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
        let window_text = format!("状态: {}", text);
        let create_param = status_text_vec.as_ptr() as *const core::ffi::c_void;
        
        let create_info = window::WindowCreateInfo {
            class_name: CLASS_NAME.to_string(),
            window_name: window_text,
            style: WS_POPUP,
            ex_style: WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED | WS_EX_NOACTIVATE,
            x: 0,
            y: 0,
            width: WINDOW_WIDTH,
            height: WINDOW_HEIGHT,
            create_param: Some(create_param),
        };
        
        let hwnd = match window::create_window(&create_info) {
            Ok(hwnd) => hwnd,
            Err(e) => {
                eprintln!("[WARN] 创建窗口失败: {}", e);
                return;
            }
        };
        
        // 保存窗口句柄
        {
            let (mutex, _) = &**CURRENT_WINDOW;
            let mut window_state = mutex.lock().unwrap();
            window_state.handle = Some(hwnd.0 as isize);
            window_state.is_closing = false;
        }
        
        // 设置窗口位置（屏幕中央）和透明度
        let screen_width = window::get_system_metrics(SM_CXSCREEN);
        let screen_height = window::get_system_metrics(SM_CYSCREEN);
        
        let _ = window::set_window_position(
            hwnd,
            (screen_width - WINDOW_WIDTH) / 2,
            (screen_height - WINDOW_HEIGHT) / 2,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            SWP_SHOWWINDOW,
        );
        
        let _ = window::set_window_alpha(hwnd, WINDOW_ALPHA);
        
        // 显示窗口
        let _ = window::show_window(hwnd, SW_SHOW);
        let _ = window::set_foreground_window(hwnd);
        let _ = window::bring_window_to_top(hwnd);
        
        // 消息循环，确保窗口绘制
        let mut msg = MSG::default();
        let start_time = std::time::Instant::now();
        
        while start_time.elapsed() < Duration::from_millis(DISPLAY_DURATION_MS) {
            unsafe {
                while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                    let _ = TranslateMessage(&msg);
                    let _ = DispatchMessageW(&msg);
                }
            }
            thread::sleep(Duration::from_millis(10));
        }
        
        // 销毁窗口并清除句柄
        let _ = window::destroy_window(hwnd);
        
        {
            let (mutex, cvar) = &**CURRENT_WINDOW;
            let mut window_state = mutex.lock().unwrap();
            window_state.handle = None;
            window_state.is_closing = false;
            cvar.notify_all();
        }
    });
}

/// 窗口过程（处理窗口消息）
unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CREATE => {
            // 保存状态文本指针
            let create_struct = lparam.0 as *const CREATESTRUCTW;
            if !create_struct.is_null() {
                let text_ptr = (*create_struct).lpCreateParams;
                if !text_ptr.is_null() {
                    let _ = window::set_window_long_ptr(hwnd, GWLP_USERDATA, text_ptr as isize);
                }
            }
            LRESULT(0)
        }
        WM_CLOSE => {
            // 标记窗口正在关闭
            {
                let (mutex, cvar) = &**CURRENT_WINDOW;
                let mut window_state = mutex.lock().unwrap();
                if let Some(hwnd_value) = window_state.handle {
                    if hwnd.0 as isize == hwnd_value {
                        window_state.is_closing = true;
                    }
                }
                cvar.notify_all();
            }
            let _ = window::destroy_window(hwnd);
            LRESULT(0)
        }
        WM_PAINT => {
            let mut ps = PAINTSTRUCT::default();
            
            if let Ok(hdc) = window::begin_paint(hwnd, &mut ps) {
                // 创建字体
                let font_info = window::FontInfo {
                    name: FONT_NAME.to_string(),
                    size: FONT_SIZE,
                    weight: FONT_WEIGHT,
                };
                
                if let Ok(hfont) = window::create_font(&font_info) {
                    if let Ok(old_font) = window::select_object(hdc, HGDIOBJ(hfont.0)) {
                        // 绘制文本
                        let rect = RECT {
                            left: 0, top: 0, right: WINDOW_WIDTH, bottom: WINDOW_HEIGHT
                        };
                        
                        let _ = window::set_bk_mode(hdc, TRANSPARENT);
                        
                        // 获取状态文本并设置颜色
                        let text_ptr = window::get_window_long_ptr(hwnd, GWLP_USERDATA) as *const u16;
                        if !text_ptr.is_null() {
                            let text_slice = std::slice::from_raw_parts(text_ptr, 256);
                            let mut text_vec = Vec::new();
                            let mut is_one = false;
                            
                            // 复制字符串直到遇到空字符
                            for &ch in text_slice.iter() {
                                if ch == 0 {
                                    break;
                                }
                                if ch == 49 && text_vec.is_empty() { // 49 = '1'
                                    is_one = true;
                                }
                                text_vec.push(ch);
                            }
                            
                            // 根据状态设置颜色
                            let text_color = if is_one { COLOR_GREEN } else { COLOR_RED };
                            let _ = window::set_text_color(hdc, text_color);
                            
                            let mut draw_info = window::DrawTextInfo {
                                text: text_vec,
                                rect,
                                format: DT_CENTER | DT_VCENTER | DT_SINGLELINE,
                            };
                            
                            let _ = window::draw_text(hdc, &mut draw_info);
                        }
                        
                        let _ = window::select_object(hdc, old_font);
                        let _ = window::delete_object(HGDIOBJ(hfont.0));
                    }
                }
                
                let _ = window::end_paint(hwnd, &ps);
            }
            
            LRESULT(0)
        }
        WM_DESTROY => {
            window::post_quit_message(0);
            LRESULT(0)
        }
        _ => window::default_window_proc(hwnd, msg, wparam, lparam),
    }
}

/// 关闭已存在的窗口（异步）
fn close_existing_window_async() {
    let (mutex, _cvar) = &**CURRENT_WINDOW;
    let mut window_state = mutex.lock().unwrap();
    
    if let Some(hwnd_value) = window_state.handle {
        if window_state.is_closing {
            return;
        }
        
        window_state.is_closing = true;
        
        // 发送关闭消息
        if let Err(e) = window::post_close_message(hwnd_value) {
            eprintln!("[WARN] 发送关闭消息失败: {}", e);
            let _ = window::destroy_window(HWND(hwnd_value as *mut core::ffi::c_void));
        }
    }
}
