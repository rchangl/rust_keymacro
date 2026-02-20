//! 键盘宏事件处理模块
//!
//! 负责处理键盘和手柄事件、执行热键动作和管理事件循环

use std::thread;
use std::sync::mpsc::{Receiver, Sender};
use crate::config::ActionParams;
use crate::gamepad::GamepadEvent;
use crate::macros::{get_config, get_event_sender, get_macro_phase, get_toggle_state, set_macro_phase};

/// 宏执行阶段
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MacroPhase {
    Idle,
    Executing,
}

/// 宏事件类型
#[derive(Debug, Clone)]
pub enum MacroEvent {
    HotkeyPressed { key_name: String },
    HotkeyReleased { key_name: String },
    GamepadButtonPressed { button: String },
    GamepadButtonReleased { button: String },
}

/// 启动宏处理线程
///
/// 返回一个 Sender，用于手柄事件转发
pub fn start_macro_thread() -> Sender<MacroEvent> {
    use std::sync::mpsc::{self, Sender, Receiver};

    let (sender, receiver): (Sender<MacroEvent>, Receiver<MacroEvent>) = mpsc::channel();

    // 保存发送者
    use crate::macros::MACRO_EVENT_SENDER;

    if let Ok(mut sender_guard) = MACRO_EVENT_SENDER.lock() {
        *sender_guard = Some(sender.clone());
    }

    // 启动处理线程
    thread::spawn(move || {
        while let Ok(event) = receiver.recv() {
            // 检查宏是否启用
            let should_execute = get_toggle_state();

            if should_execute {
                match event {
                    MacroEvent::HotkeyPressed { key_name } => {
                        if let Err(e) = execute_hotkey_action(&key_name) {
                            log::debug!("执行热键动作失败 ({}): {}", key_name, e);
                        }
                    }
                    MacroEvent::HotkeyReleased { key_name } => {
                        if let Err(e) = execute_hotkey_release(&key_name) {
                            log::debug!("执行热键释放失败 ({}): {}", key_name, e);
                        }
                    }
                    MacroEvent::GamepadButtonPressed { button } => {
                        let key_name = format!("GP:{}", button);
                        log::debug!("手柄按下事件: button={}, key_name={}", button, key_name);
                        if let Err(e) = execute_hotkey_action(&key_name) {
                            log::debug!("执行手柄动作失败 ({}): {}", key_name, e);
                        }
                    }
                    MacroEvent::GamepadButtonReleased { button } => {
                        let key_name = format!("GP:{}", button);
                        log::debug!("手柄释放事件: button={}, key_name={}", button, key_name);
                        if let Err(e) = execute_hotkey_release(&key_name) {
                            log::debug!("执行手柄释放失败 ({}): {}", key_name, e);
                        }
                    }
                }
            }
        }
    });

    sender
}

/// 启动手柄事件转发线程
pub fn start_gamepad_forwarder(gamepad_receiver: Receiver<GamepadEvent>, macro_sender: Sender<MacroEvent>) {
    log::info!("手柄事件转发线程已启动");
    thread::spawn(move || {
        while let Ok(event) = gamepad_receiver.recv() {
            log::debug!("转发手柄事件: {:?}", event);
            let macro_event = match event {
                GamepadEvent::ButtonPressed { button } => {
                    MacroEvent::GamepadButtonPressed { button }
                }
                GamepadEvent::ButtonReleased { button } => {
                    MacroEvent::GamepadButtonReleased { button }
                }
            };

            if let Err(e) = macro_sender.send(macro_event) {
                log::warn!("发送手柄事件失败: {}", e);
                break;
            }
        }
        log::warn!("手柄事件转发线程已退出");
    });
}

/// 执行热键动作（按下阶段）
fn execute_hotkey_action(key_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 检查并设置状态
    let can_execute = {
        let phase = get_macro_phase();
        if phase == MacroPhase::Idle {
            set_macro_phase(MacroPhase::Executing);
            true
        } else {
            false
        }
    };
    
    if !can_execute {
        return Ok(());
    }
    
    // 获取配置
    let config = get_config().ok_or("配置未加载")?;
    
    // 查找热键配置
    log::debug!("查找热键配置: {}", key_name);
    let hotkey_config = config.find_hotkey(key_name)
        .ok_or_else(|| {
            log::debug!("未找到热键配置: {}，可用热键: {:?}", key_name, 
                config.hotkeys.iter().map(|h| h.key()).collect::<Vec<_>>());
            format!("未找到热键配置: {}", key_name)
        })?;
    
    // 执行动作
    match hotkey_config.action.as_str() {
        "type_text" => {
            if let ActionParams::TypeText(params) = &hotkey_config.params {
                crate::macros::execute_type_text(params)?;
            }
        }
        "sequence" => {
            if let ActionParams::Sequence(params) = &hotkey_config.params {
                crate::macros::execute_sequence(params)?;
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
    let should_release = {
        let phase = get_macro_phase();
        if phase == MacroPhase::Executing {
            set_macro_phase(MacroPhase::Idle);
            true
        } else {
            false
        }
    };
    
    if !should_release {
        return Ok(());
    }
    
    // 这里可以添加释放按键的逻辑，如果有需要的话
    // 例如，如果某些键在按下后需要保持，在这里释放
    
    Ok(())
}

/// 键盘钩子回调
///
/// 监听低级键盘事件，当按下配置中的热键时触发宏
pub unsafe extern "system" fn keyboard_hook_proc(code: i32, wparam: windows::Win32::Foundation::WPARAM, lparam: windows::Win32::Foundation::LPARAM) -> windows::Win32::Foundation::LRESULT {
    use windows::Win32::UI::WindowsAndMessaging::*;
    use windows::Win32::Foundation::LRESULT;
    use crate::winapi::keyboard;
    
    if code >= 0 {
        let kb_struct = keyboard::get_keyboard_hook_struct(lparam);
        
        // 检查是否是模拟按键（由我们自己的 simulate_key 发送）
        // 如果是模拟按键，直接放行，避免死循环
        if kb_struct.dwExtraInfo == 0x12345678 {
            return keyboard::call_next_hook(HHOOK::default(), code, wparam, lparam);
        }
        
        // 检查宏是否启用
        if get_toggle_state() {
            // 检查是否在配置中
            if let Some(config) = get_config() {
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
                        
                        // 检查是否正在执行宏，如果是则丢弃新事件（防止堆积）
                        if get_macro_phase() != MacroPhase::Idle {
                            return LRESULT(1); // 阻止原始事件，但不发送新事件
                        }
                        
                        // 首次按下且空闲状态，发送事件
                        if let Some(sender) = get_event_sender() {
                            let _ = sender.send(MacroEvent::HotkeyPressed { key_name });
                        }
                        return LRESULT(1); // 阻止原始事件
                    }
                    // 处理松开事件
                    else if keyboard::is_key_up(wparam) {
                        // 只有当前正在执行该热键的宏时才发送释放事件
                        // 这样可以防止事件堆积，也能避免处理过期的释放事件
                        if get_macro_phase() == MacroPhase::Executing {
                            if let Some(sender) = get_event_sender() {
                                let _ = sender.send(MacroEvent::HotkeyReleased { key_name });
                            }
                        }
                        return LRESULT(1); // 阻止原始事件
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
    use windows::Win32::UI::Input::KeyboardAndMouse::*;
    
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
