//! 键盘宏功能模块
//!
//! 从配置文件加载热键映射，支持多种操作类型

use windows::Win32::{
    Foundation::{LPARAM, LRESULT, WPARAM},
    UI::{
        WindowsAndMessaging::*,
        Input::KeyboardAndMouse::*,
    },
};
use std::{
    sync::{Mutex, mpsc::{self, Sender, Receiver}},
    thread,
    time::Duration,
};
use once_cell::sync::Lazy;
use crate::winapi::keyboard;
use crate::config::{Config, ActionParams, Step};

/// 宏执行阶段
#[derive(Debug, Clone, Copy, PartialEq)]
enum MacroPhase {
    Idle,
    Executing,
}

/// 宏事件类型
#[derive(Debug, Clone)]
enum MacroEvent {
    HotkeyPressed { key_name: String },
    HotkeyReleased { key_name: String },
}

// 全局变量
static TOGGLE_STATE: Lazy<Mutex<bool>> = Lazy::new(|| Mutex::new(true));
static MACRO_PHASE: Lazy<Mutex<MacroPhase>> = Lazy::new(|| Mutex::new(MacroPhase::Idle));
static MACRO_EVENT_SENDER: Lazy<Mutex<Option<Sender<MacroEvent>>>> = Lazy::new(|| Mutex::new(None));
static CONFIG: Lazy<Mutex<Option<Config>>> = Lazy::new(|| Mutex::new(None));

/// 初始化键盘宏系统
///
/// # 参数
///
/// * `config` - 配置对象
///
/// # 返回
///
/// 成功返回钩子句柄，失败返回 None
///
/// # 说明
///
/// 设置低级键盘钩子监听全局键盘事件，启动宏处理线程
pub fn init_keyboard_macro_system(config: Config) -> Option<HHOOK> {
    // 保存配置
    if let Ok(mut config_guard) = CONFIG.lock() {
        *config_guard = Some(config);
    }
    
    start_macro_thread();
    
    match keyboard::set_keyboard_hook(Some(keyboard_hook_proc), 0) {
        Ok(hook) => Some(hook),
        Err(e) => {
            eprintln!("[WARN] 设置键盘钩子失败: {}", e);
            None
        }
    }
}

/// 设置配置（用于运行时重载）
#[allow(dead_code)]
pub fn set_config(config: Config) {
    if let Ok(mut config_guard) = CONFIG.lock() {
        *config_guard = Some(config);
    }
}

/// 启动宏处理线程
fn start_macro_thread() {
    let (sender, receiver): (Sender<MacroEvent>, Receiver<MacroEvent>) = mpsc::channel();
    
    // 保存发送者
    if let Ok(mut sender_guard) = MACRO_EVENT_SENDER.lock() {
        *sender_guard = Some(sender);
    }
    
    // 启动处理线程
    thread::spawn(move || {
        while let Ok(event) = receiver.recv() {
            // 检查宏是否启用
            let should_execute = if let Ok(state) = TOGGLE_STATE.lock() {
                *state
            } else {
                false
            };
            
            if should_execute {
                match event {
                    MacroEvent::HotkeyPressed { key_name } => {
                        if let Err(e) = execute_hotkey_action(&key_name) {
                            eprintln!("[DEBUG] 执行热键动作失败 ({}): {}", key_name, e);
                        }
                    }
                    MacroEvent::HotkeyReleased { key_name } => {
                        if let Err(e) = execute_hotkey_release(&key_name) {
                            eprintln!("[DEBUG] 执行热键释放失败 ({}): {}", key_name, e);
                        }
                    }
                }
            }
        }
    });
}

/// 执行热键动作（按下阶段）
fn execute_hotkey_action(key_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 检查并设置状态
    let can_execute = if let Ok(mut phase) = MACRO_PHASE.lock() {
        if *phase == MacroPhase::Idle {
            *phase = MacroPhase::Executing;
            true
        } else {
            false
        }
    } else {
        false
    };
    
    if !can_execute {
        return Ok(());
    }
    
    // 获取配置
    let config = if let Ok(config_guard) = CONFIG.lock() {
        if let Some(cfg) = config_guard.as_ref() {
            cfg.clone()
        } else {
            return Err("配置未加载".into());
        }
    } else {
        return Err("无法获取配置锁".into());
    };
    
    // 查找热键配置
    let hotkey_config = if let Some(hk) = config.find_hotkey(key_name) {
        hk
    } else {
        return Err(format!("未找到热键配置: {}", key_name).into());
    };
    
    // 执行动作
    match hotkey_config.action.as_str() {
        "type_text" => {
            if let ActionParams::TypeText(params) = &hotkey_config.params {
                execute_type_text(params)?;
            }
        }
        "sequence" => {
            if let ActionParams::Sequence(params) = &hotkey_config.params {
                execute_sequence(params)?;
            }
        }
        _ => {
            return Err(format!("未知的动作类型: {}", hotkey_config.action).into());
        }
    }
    
    Ok(())
}

/// 执行热键释放（清理阶段）
fn execute_hotkey_release(_key_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let should_release = if let Ok(mut phase) = MACRO_PHASE.lock() {
        if *phase == MacroPhase::Executing {
            *phase = MacroPhase::Idle;
            true
        } else {
            false
        }
    } else {
        false
    };
    
    if !should_release {
        return Ok(());
    }
    
    // 这里可以添加释放按键的逻辑，如果有需要的话
    // 例如，如果某些键在按下后需要保持，在这里释放
    
    Ok(())
}

/// 执行输入文本操作
fn execute_type_text(params: &crate::config::TypeTextParams) -> Result<(), Box<dyn std::error::Error>> {
    // 根据速度设置延迟
    let char_delay = match params.speed.as_deref() {
        Some("fastest") => Duration::from_millis(5),
        Some("fast") => Duration::from_millis(10),
        Some("normal") => Duration::from_millis(20),
        Some("slow") => Duration::from_millis(50),
        _ => Duration::from_millis(10),
    };
    
    // 输入每个字符
    for ch in params.text.chars() {
        if let Some(vk) = char_to_vk(ch) {
            keyboard::simulate_key_press(vk)?;
            thread::sleep(char_delay);
            keyboard::simulate_key_release(vk)?;
            thread::sleep(char_delay);
        } else {
            // 尝试发送 Unicode 字符
            simulate_unicode_char(ch)?;
        }
    }
    
    Ok(())
}

/// 执行序列操作
fn execute_sequence(params: &crate::config::SequenceParams) -> Result<(), Box<dyn std::error::Error>> {
    for step in &params.steps {
        match step {
            Step::Key { value, delay, action } => {
                if let Some(vk) = parse_key_string(value) {
                    let key_action = action.as_ref().unwrap_or(&crate::config::KeyAction::Complete);
                    
                    match key_action {
                        crate::config::KeyAction::Press => {
                            keyboard::simulate_key_press(vk)?;
                            if let Some(d) = delay {
                                thread::sleep(Duration::from_millis(*d));
                            }
                        }
                        crate::config::KeyAction::Release => {
                            keyboard::simulate_key_release(vk)?;
                            if let Some(d) = delay {
                                thread::sleep(Duration::from_millis(*d));
                            }
                        }
                        crate::config::KeyAction::Complete => {
                            keyboard::simulate_key_press(vk)?;
                            if let Some(d) = delay {
                                thread::sleep(Duration::from_millis(*d));
                            }
                            keyboard::simulate_key_release(vk)?;
                        }
                    }
                }
            }
            Step::Wait { value } => {
                thread::sleep(Duration::from_millis(*value));
            }
            Step::Text { value, delay } => {
                for ch in value.chars() {
                    if let Some(vk) = char_to_vk(ch) {
                        keyboard::simulate_key_press(vk)?;
                        if let Some(d) = delay {
                            thread::sleep(Duration::from_millis(*d));
                        }
                        keyboard::simulate_key_release(vk)?;
                    } else {
                        simulate_unicode_char(ch)?;
                    }
                }
            }
        }
    }
    
    Ok(())
}

/// 设置宏启用状态
///
/// # 参数
///
/// * `enabled` - true 启用，false 禁用
pub fn set_macro_enabled(enabled: bool) {
    if let Ok(mut state) = TOGGLE_STATE.lock() {
        *state = enabled;
    }
}

/// 清理键盘钩子
///
/// # 参数
///
/// * `hook` - 要卸载的钩子句柄
pub fn cleanup_keyboard_hook(hook: HHOOK) {
    if let Err(e) = keyboard::unhook_keyboard_hook(hook) {
        eprintln!("[DEBUG] 卸载键盘钩子失败: {}", e);
    }
}

/// 键盘钩子回调
///
/// 监听低级键盘事件，当按下配置中的热键时触发宏
unsafe extern "system" fn keyboard_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code >= 0 {
        let kb_struct = keyboard::get_keyboard_hook_struct(lparam);
        
        // 检查是否是模拟按键（由我们自己的 simulate_key 发送）
        // 如果是模拟按键，直接放行，避免死循环
        if kb_struct.dwExtraInfo == 0x12345678 {
            return keyboard::call_next_hook(HHOOK::default(), code, wparam, lparam);
        }
        
        // 检查宏是否启用
        let macro_enabled = if let Ok(state) = TOGGLE_STATE.lock() {
            *state
        } else {
            false
        };
        
        if macro_enabled {
            // 检查是否在配置中
            if let Ok(config_guard) = CONFIG.lock() {
                if let Some(config) = config_guard.as_ref() {
                    // 构建当前按键字符串（简单实现，支持单键）
                    let key_name = vk_to_key_name(kb_struct.vkCode);
                    
                    if config.find_hotkey(&key_name).is_some() {
                        // 处理按下事件
                        if keyboard::is_key_down(wparam) {
                            // 检查是否是重复事件（长按自动重复）
                            if keyboard::is_key_repeat(lparam) {
                                // 是重复事件，忽略，不发送事件，不阻止原始事件
                                return keyboard::call_next_hook(HHOOK::default(), code, wparam, lparam);
                            }
                            
                            // 首次按下，发送事件
                            if let Ok(sender_guard) = MACRO_EVENT_SENDER.lock() {
                                if let Some(sender) = sender_guard.as_ref() {
                                    let _ = sender.send(MacroEvent::HotkeyPressed { key_name });
                                }
                            }
                            return LRESULT(1); // 阻止原始事件
                        }
                        // 处理松开事件
                        else if keyboard::is_key_up(wparam) {
                            if let Ok(sender_guard) = MACRO_EVENT_SENDER.lock() {
                                if let Some(sender) = sender_guard.as_ref() {
                                    let _ = sender.send(MacroEvent::HotkeyReleased { key_name });
                                }
                            }
                            return LRESULT(1); // 阻止原始事件
                        }
                    }
                }
            }
        }
    }
    
    // 调用下一个钩子
    keyboard::call_next_hook(HHOOK::default(), code, wparam, lparam)
}

/// 将虚拟键码转换为键名字符串（简单实现）
fn vk_to_key_name(vk: u32) -> String {
    match vk {
        0x41 => "A".to_string(),
        0x42 => "B".to_string(),
        0x43 => "C".to_string(),
        0x44 => "D".to_string(),
        0x45 => "E".to_string(),
        0x46 => "F".to_string(),
        0x47 => "G".to_string(),
        0x48 => "H".to_string(),
        0x49 => "I".to_string(),
        0x4A => "J".to_string(),
        0x4B => "K".to_string(),
        0x4C => "L".to_string(),
        0x4D => "M".to_string(),
        0x4E => "N".to_string(),
        0x4F => "O".to_string(),
        0x50 => "P".to_string(),
        0x51 => "Q".to_string(),
        0x52 => "R".to_string(),
        0x53 => "S".to_string(),
        0x54 => "T".to_string(),
        0x55 => "U".to_string(),
        0x56 => "V".to_string(),
        0x57 => "W".to_string(),
        0x58 => "X".to_string(),
        0x59 => "Y".to_string(),
        0x5A => "Z".to_string(),
        0x30..=0x39 => format!("{}", vk - 0x30),
        0x60..=0x69 => format!("Numpad{}", vk - 0x60),
        0x70..=0x87 => format!("F{}", vk - 0x6F),
        x if x == VK_OEM_3.0 as u32 => "`".to_string(),
        x if x == VK_OEM_7.0 as u32 => "'".to_string(),
        x if x == VK_SPACE.0 as u32 => "Space".to_string(),
        x if x == VK_RETURN.0 as u32 => "Enter".to_string(),
        x if x == VK_TAB.0 as u32 => "Tab".to_string(),
        x if x == VK_BACK.0 as u32 => "Backspace".to_string(),
        x if x == VK_ESCAPE.0 as u32 => "Escape".to_string(),
        x if x == VK_SHIFT.0 as u32 => "Shift".to_string(),
        x if x == VK_CONTROL.0 as u32 => "Ctrl".to_string(),
        x if x == VK_MENU.0 as u32 => "Alt".to_string(),
        _ => format!("VK_{:X}", vk),
    }
}

/// 将键名字符串解析为虚拟键码
fn parse_key_string(key: &str) -> Option<u16> {
    match key.to_uppercase().as_str() {
        "A" => Some(0x41),
        "B" => Some(0x42),
        "C" => Some(0x43),
        "D" => Some(0x44),
        "E" => Some(0x45),
        "F" => Some(0x46),
        "G" => Some(0x47),
        "H" => Some(0x48),
        "I" => Some(0x49),
        "J" => Some(0x4A),
        "K" => Some(0x4B),
        "L" => Some(0x4C),
        "M" => Some(0x4D),
        "N" => Some(0x4E),
        "O" => Some(0x4F),
        "P" => Some(0x50),
        "Q" => Some(0x51),
        "R" => Some(0x52),
        "S" => Some(0x53),
        "T" => Some(0x54),
        "U" => Some(0x55),
        "V" => Some(0x56),
        "W" => Some(0x57),
        "X" => Some(0x58),
        "Y" => Some(0x59),
        "Z" => Some(0x5A),
        s if s.len() == 1 && s.chars().next().unwrap().is_ascii_digit() => {
            s.chars().next().map(|c| c as u16 - '0' as u16 + 0x30)
        }
        "SPACE" | "Space" => Some(VK_SPACE.0),
        "ENTER" | "Enter" => Some(VK_RETURN.0),
        "TAB" | "Tab" => Some(VK_TAB.0),
        "BACKSPACE" | "Backspace" => Some(VK_BACK.0),
        "ESC" | "Escape" => Some(VK_ESCAPE.0),
        "SHIFT" | "Shift" => Some(VK_SHIFT.0),
        "CTRL" | "Ctrl" => Some(VK_CONTROL.0),
        "ALT" | "Alt" => Some(VK_MENU.0),
        _ => None,
    }
}

/// 将字符转换为虚拟键码
fn char_to_vk(ch: char) -> Option<u16> {
    match ch {
        'a'..='z' => Some(ch as u16 - 'a' as u16 + 0x41),
        'A'..='Z' => Some(ch as u16 - 'A' as u16 + 0x41),
        '0'..='9' => Some(ch as u16 - '0' as u16 + 0x30),
        ' ' => Some(VK_SPACE.0),
        '\r' | '\n' => Some(VK_RETURN.0),
        '\t' => Some(VK_TAB.0),
        _ => None,
    }
}

/// 模拟 Unicode 字符输入（备用方案）
fn simulate_unicode_char(_ch: char) -> Result<(), Box<dyn std::error::Error>> {
    // 这里可以实现 Unicode 字符输入，使用 SendInput 的 Unicode 模式
    // 为简化实现，这里暂时返回错误
    Err("Unicode 字符不支持".into())
}
