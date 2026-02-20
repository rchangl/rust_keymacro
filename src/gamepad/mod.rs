//! 游戏手柄输入处理模块
//!
//! 使用 Windows XInput API 支持 Xbox 协议手柄

use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::UI::Input::XboxController::*;

/// 手柄事件类型
#[derive(Debug, Clone)]
pub enum GamepadEvent {
    ButtonPressed { button: String },
    ButtonReleased { button: String },
}

/// 启动手柄监听线程
///
/// 返回一个 Receiver，用于接收手柄事件
pub fn start_gamepad_thread() -> Receiver<GamepadEvent> {
    let (sender, receiver) = mpsc::channel::<GamepadEvent>();

    thread::spawn(move || {
        log::info!("手柄监听线程启动 (XInput)");

        // 检查 XInput 是否可用
        let mut found_controller = false;
        for i in 0..4u32 {
            let mut state = XINPUT_STATE::default();
            let result = unsafe { XInputGetState(i, &mut state) };
            if result == ERROR_SUCCESS.0 {
                log::info!("检测到手柄 [{}] 已连接", i);
                found_controller = true;
            }
        }

        if !found_controller {
            log::warn!("未检测到手柄，等待手柄连接...");
        }

        // 跟踪每个手柄的按钮状态
        let mut prev_states: [u16; 4] = [0; 4];
        let mut controller_connected: [bool; 4] = [false; 4];

        loop {
            for i in 0..4usize {
                let mut state = XINPUT_STATE::default();
                let result = unsafe { XInputGetState(i as u32, &mut state) };

                if result == ERROR_SUCCESS.0 {
                    // 手柄已连接
                    if !controller_connected[i] {
                        log::info!("手柄 [{}] 已连接", i);
                        controller_connected[i] = true;
                    }

                    let current_buttons = state.Gamepad.wButtons.0;
                    let changed = current_buttons ^ prev_states[i];

                    if changed != 0 {
                        // 有按钮状态变化
                        check_button_changes(
                            i as u32,
                            prev_states[i],
                            current_buttons,
                            changed,
                            &sender,
                        );
                        prev_states[i] = current_buttons;
                    }
                } else {
                    // 手柄未连接或断开
                    if controller_connected[i] {
                        log::info!("手柄 [{}] 已断开", i);
                        controller_connected[i] = false;
                        prev_states[i] = 0;
                    }
                }
            }

            // 降低 CPU 占用 (约 60Hz 轮询)
            thread::sleep(Duration::from_millis(16));
        }
    });

    receiver
}

/// 检查按钮变化并发送事件
fn check_button_changes(
    controller_id: u32,
    _prev: u16,
    current: u16,
    changed: u16,
    sender: &mpsc::Sender<GamepadEvent>,
) {
    // 定义按钮映射
    let buttons: [(u16, &str); 14] = [
        (XINPUT_GAMEPAD_DPAD_UP.0, "DUp"),
        (XINPUT_GAMEPAD_DPAD_DOWN.0, "DDown"),
        (XINPUT_GAMEPAD_DPAD_LEFT.0, "DLeft"),
        (XINPUT_GAMEPAD_DPAD_RIGHT.0, "DRight"),
        (XINPUT_GAMEPAD_START.0, "Start"),
        (XINPUT_GAMEPAD_BACK.0, "Back"),
        (XINPUT_GAMEPAD_LEFT_THUMB.0, "LS"),
        (XINPUT_GAMEPAD_RIGHT_THUMB.0, "RS"),
        (XINPUT_GAMEPAD_LEFT_SHOULDER.0, "LB"),
        (XINPUT_GAMEPAD_RIGHT_SHOULDER.0, "RB"),
        (XINPUT_GAMEPAD_A.0, "A"),
        (XINPUT_GAMEPAD_B.0, "B"),
        (XINPUT_GAMEPAD_X.0, "X"),
        (XINPUT_GAMEPAD_Y.0, "Y"),
    ];

    for (mask, name) in &buttons {
        if changed & mask != 0 {
            if current & mask != 0 {
                // 按钮按下
                log::info!("手柄 [{}] 按钮按下: {}", controller_id, name);
                if let Err(e) = sender.send(GamepadEvent::ButtonPressed {
                    button: name.to_string(),
                }) {
                    log::error!("发送按钮按下事件失败: {}", e);
                }
            } else {
                // 按钮释放
                log::info!("手柄 [{}] 按钮释放: {}", controller_id, name);
                if let Err(e) = sender.send(GamepadEvent::ButtonReleased {
                    button: name.to_string(),
                }) {
                    log::error!("发送按钮释放事件失败: {}", e);
                }
            }
        }
    }
}

/// 将 gilrs Button 映射为配置键名（保留此函数以兼容现有代码）
pub fn button_to_key_name(button: &str) -> String {
    // Xbox 标准按键映射
    match button {
        "South" => "A".to_string(),
        "East" => "B".to_string(),
        "West" => "X".to_string(),
        "North" => "Y".to_string(),
        "LeftTrigger" => "LT".to_string(),
        "RightTrigger" => "RT".to_string(),
        "LeftTrigger2" => "LB".to_string(),
        "RightTrigger2" => "RB".to_string(),
        "Select" => "Back".to_string(),
        "Start" => "Start".to_string(),
        "Mode" => "Guide".to_string(),
        "LeftThumb" => "LS".to_string(),
        "RightThumb" => "RS".to_string(),
        "DPadUp" => "DUp".to_string(),
        "DPadDown" => "DDown".to_string(),
        "DPadLeft" => "DLeft".to_string(),
        "DPadRight" => "DRight".to_string(),
        _ => button.to_string(),
    }
}
