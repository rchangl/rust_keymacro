//! 键盘宏执行模块
//!
//! 负责执行各种宏操作，包括输入文本和按键序列

use std::thread;
use std::time::Duration;
use crate::config::{TypeTextParams, SequenceParams, Step, KeyAction};
use crate::winapi::keyboard;

/// 执行输入文本操作
pub fn execute_type_text(params: &TypeTextParams) -> Result<(), Box<dyn std::error::Error>> {
    // 使用配置的延迟，默认为 10ms
    let char_delay = Duration::from_millis(params.delay.unwrap_or(10));
    
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
pub fn execute_sequence(params: &SequenceParams) -> Result<(), Box<dyn std::error::Error>> {
    for step in &params.steps {
        match step {
            Step::Key { value, delay, action } => {
                if let Some(vk) = parse_key_string(value) {
                    let key_action = action.as_ref().unwrap_or(&KeyAction::Complete);
                    
                    match key_action {
                        KeyAction::Press => {
                            keyboard::simulate_key_press(vk)?;
                            if let Some(d) = delay {
                                thread::sleep(Duration::from_millis(*d));
                            }
                        }
                        KeyAction::Release => {
                            keyboard::simulate_key_release(vk)?;
                            if let Some(d) = delay {
                                thread::sleep(Duration::from_millis(*d));
                            }
                        }
                        KeyAction::Complete => {
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

/// 将字符转换为虚拟键码
fn char_to_vk(ch: char) -> Option<u16> {
    match ch {
        'a'..='z' => Some(ch as u16 - 'a' as u16 + 0x41),
        'A'..='Z' => Some(ch as u16 - 'A' as u16 + 0x41),
        '0'..='9' => Some(ch as u16 - '0' as u16 + 0x30),
        ' ' => Some(windows::Win32::UI::Input::KeyboardAndMouse::VK_SPACE.0),
        '\r' | '\n' => Some(windows::Win32::UI::Input::KeyboardAndMouse::VK_RETURN.0),
        '\t' => Some(windows::Win32::UI::Input::KeyboardAndMouse::VK_TAB.0),
        _ => None,
    }
}

/// 模拟 Unicode 字符输入（备用方案）
fn simulate_unicode_char(_ch: char) -> Result<(), Box<dyn std::error::Error>> {
    // 这里可以实现 Unicode 字符输入，使用 SendInput 的 Unicode 模式
    // 为简化实现，这里暂时返回错误
    Err("Unicode 字符不支持".into())
}

/// 将键名字符串解析为虚拟键码
fn parse_key_string(key: &str) -> Option<u16> {
    use windows::Win32::UI::Input::KeyboardAndMouse::*;
    
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
