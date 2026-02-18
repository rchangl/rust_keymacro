//! Windows 键盘 API 安全封装
//!
//! 提供键盘钩子、按键模拟等功能的安全接口

use windows::Win32::{
    Foundation::{HINSTANCE, LPARAM, LRESULT, WPARAM},
    UI::{
        WindowsAndMessaging::*,
        Input::KeyboardAndMouse::*,
    },
};
use windows::Win32::UI::Input::KeyboardAndMouse::MAPVK_VK_TO_VSC;

/// 按键事件类型
#[derive(Debug, Clone, Copy)]
pub enum KeyEventType {
    Press,
    Release,
}

/// 设置低级键盘钩子
///
/// # 参数
///
/// * `hook_proc` - 钩子回调
/// * `thread_id` - 线程 ID（0 表示所有线程）
pub fn set_keyboard_hook(hook_proc: HOOKPROC, thread_id: u32) -> Result<HHOOK, windows::core::Error> {
    unsafe {
        SetWindowsHookExW(WH_KEYBOARD_LL, hook_proc, HINSTANCE::default(), thread_id)
    }
}

/// 卸载键盘钩子
///
/// # 参数
///
/// * `hook` - 要卸载的钩子句柄
pub fn unhook_keyboard_hook(hook: HHOOK) -> Result<(), windows::core::Error> {
    unsafe {
        UnhookWindowsHookEx(hook)?;
        Ok(())
    }
}

/// 调用下一个钩子
///
/// # 参数
///
/// * `hook` - 当前钩子句柄
/// * `code` - 钩子代码
/// * `wparam` - WPARAM
/// * `lparam` - LPARAM
pub fn call_next_hook(hook: HHOOK, code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        CallNextHookEx(hook, code, wparam, lparam)
    }
}

/// 模拟按键
///
/// # 参数
///
/// * `vk` - 虚拟键码
/// * `event_type` - 事件类型（按下或释放）
pub fn simulate_key(vk: u16, event_type: KeyEventType) -> Result<(), windows::core::Error> {
    unsafe {
        let scan_code = MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC);
        
        let flags = match event_type {
            KeyEventType::Press => {
                if scan_code != 0 {
                    KEYEVENTF_SCANCODE
                } else {
                    KEYBD_EVENT_FLAGS::default()
                }
            }
            KeyEventType::Release => {
                if scan_code != 0 {
                    KEYEVENTF_KEYUP | KEYEVENTF_SCANCODE
                } else {
                    KEYEVENTF_KEYUP
                }
            }
        };
        
        let mut input = INPUT::default();
        input.r#type = INPUT_KEYBOARD;
        input.Anonymous.ki.wVk = VIRTUAL_KEY(vk);
        input.Anonymous.ki.wScan = scan_code as u16;
        input.Anonymous.ki.dwFlags = flags;
        input.Anonymous.ki.time = 0;
        // 使用特殊标记标识这是模拟按键，避免钩子死循环
        input.Anonymous.ki.dwExtraInfo = 0x12345678;
        
        let result = SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        if result == 0 {
            Err(windows::core::Error::from_win32())
        } else {
            Ok(())
        }
    }
}

/// 模拟按键按下
///
/// # 参数
///
/// * `vk` - 虚拟键码
pub fn simulate_key_press(vk: u16) -> Result<(), windows::core::Error> {
    simulate_key(vk, KeyEventType::Press)
}

/// 模拟按键释放
///
/// # 参数
///
/// * `vk` - 虚拟键码
pub fn simulate_key_release(vk: u16) -> Result<(), windows::core::Error> {
    simulate_key(vk, KeyEventType::Release)
}

/// 模拟完整按键（按下+释放）
#[allow(dead_code)]
pub fn simulate_key_complete(vk: u16) -> Result<(), windows::core::Error> {
    simulate_key_press(vk)?;
    simulate_key_release(vk)?;
    Ok(())
}

/// 从 LPARAM 获取键盘钩子结构
///
/// # 安全
///
/// 需要信任 LPARAM 包含有效的 KBDLLHOOKSTRUCT 指针
pub unsafe fn get_keyboard_hook_struct(lparam: LPARAM) -> &'static KBDLLHOOKSTRUCT {
    &*(lparam.0 as *const KBDLLHOOKSTRUCT)
}

/// 检查是否是键按下消息
pub fn is_key_down(wparam: WPARAM) -> bool {
    wparam.0 as u32 == WM_KEYDOWN
}

/// 检查是否是键释放消息
#[allow(dead_code)]
pub fn is_key_up(wparam: WPARAM) -> bool {
    wparam.0 as u32 == WM_KEYUP
}

/// 检查按键是否是重复事件（长按自动重复）
/// 
/// # 参数
/// 
/// * `lparam` - LPARAM，包含 KBDLLHOOKSTRUCT
/// 
/// # 返回
/// 
/// true 表示是重复事件，false 表示是首次按下
pub fn is_key_repeat(lparam: LPARAM) -> bool {
    const LLKHF_REPEAT: u32 = 0x0001;
    
    let kb_struct = unsafe { get_keyboard_hook_struct(lparam) };
    let flags: u32 = kb_struct.flags.0;
    (flags & LLKHF_REPEAT) != 0
}
