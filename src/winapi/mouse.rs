//! 鼠标控制模块
//!
//! 提供鼠标移动、点击、滚轮等操作的Windows API封装

use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_ABSOLUTE, MOUSEEVENTF_LEFTDOWN,
    MOUSEEVENTF_LEFTUP, MOUSEEVENTF_MIDDLEDOWN, MOUSEEVENTF_MIDDLEUP, MOUSEEVENTF_MOVE,
    MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEEVENTF_WHEEL, MOUSEINPUT,
};

/// 鼠标按钮类型
#[derive(Debug, Clone)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// 移动鼠标到绝对坐标
///
/// # 参数
///
/// * `x` - X坐标（屏幕像素坐标）
/// * `y` - Y坐标（屏幕像素坐标）
pub fn move_mouse_to(x: i32, y: i32) -> Result<(), windows::core::Error> {
    unsafe {
        // 将像素坐标转换为标准化坐标（0-65535）
        let screen_width = windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
            windows::Win32::UI::WindowsAndMessaging::SM_CXSCREEN,
        );
        let screen_height = windows::Win32::UI::WindowsAndMessaging::GetSystemMetrics(
            windows::Win32::UI::WindowsAndMessaging::SM_CYSCREEN,
        );

        let normalized_x = ((x as f64 / screen_width as f64) * 65535.0) as i32;
        let normalized_y = ((y as f64 / screen_height as f64) * 65535.0) as i32;

        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: normalized_x,
                    dy: normalized_y,
                    mouseData: 0,
                    dwFlags: MOUSEEVENTF_MOVE | MOUSEEVENTF_ABSOLUTE,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        let result = SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        if result == 0 {
            Err(windows::core::Error::from_win32())
        } else {
            Ok(())
        }
    }
}

/// 相对移动鼠标
///
/// # 参数
///
/// * `dx` - X轴相对移动量（像素）
/// * `dy` - Y轴相对移动量（像素）
pub fn move_mouse_relative(dx: i32, dy: i32) -> Result<(), windows::core::Error> {
    unsafe {
        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx,
                    dy,
                    mouseData: 0,
                    dwFlags: MOUSEEVENTF_MOVE,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        let result = SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        if result == 0 {
            Err(windows::core::Error::from_win32())
        } else {
            Ok(())
        }
    }
}

/// 鼠标点击
///
/// # 参数
///
/// * `button` - 鼠标按钮类型
pub fn mouse_click(button: MouseButton) -> Result<(), windows::core::Error> {
    mouse_down(&button)?;
    std::thread::sleep(std::time::Duration::from_millis(10));
    mouse_up(&button)?;
    Ok(())
}

/// 鼠标按下
///
/// # 参数
///
/// * `button` - 鼠标按钮类型
pub fn mouse_down(button: &MouseButton) -> Result<(), windows::core::Error> {
    unsafe {
        let flag = match button {
            MouseButton::Left => MOUSEEVENTF_LEFTDOWN,
            MouseButton::Right => MOUSEEVENTF_RIGHTDOWN,
            MouseButton::Middle => MOUSEEVENTF_MIDDLEDOWN,
        };

        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: 0,
                    dwFlags: flag,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        let result = SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        if result == 0 {
            Err(windows::core::Error::from_win32())
        } else {
            Ok(())
        }
    }
}

/// 鼠标释放
///
/// # 参数
///
/// * `button` - 鼠标按钮类型
pub fn mouse_up(button: &MouseButton) -> Result<(), windows::core::Error> {
    unsafe {
        let flag = match button {
            MouseButton::Left => MOUSEEVENTF_LEFTUP,
            MouseButton::Right => MOUSEEVENTF_RIGHTUP,
            MouseButton::Middle => MOUSEEVENTF_MIDDLEUP,
        };

        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: 0,
                    dwFlags: flag,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        let result = SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        if result == 0 {
            Err(windows::core::Error::from_win32())
        } else {
            Ok(())
        }
    }
}

/// 鼠标滚轮
///
/// # 参数
///
/// * `delta` - 滚轮滚动量（正数向上，负数向下，通常为120的倍数）
pub fn mouse_wheel(delta: i32) -> Result<(), windows::core::Error> {
    unsafe {
        let input = INPUT {
            r#type: INPUT_MOUSE,
            Anonymous: INPUT_0 {
                mi: MOUSEINPUT {
                    dx: 0,
                    dy: 0,
                    mouseData: delta as u32,
                    dwFlags: MOUSEEVENTF_WHEEL,
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };

        let result = SendInput(&[input], std::mem::size_of::<INPUT>() as i32);
        if result == 0 {
            Err(windows::core::Error::from_win32())
        } else {
            Ok(())
        }
    }
}

/// 获取当前鼠标位置
///
/// # 返回
///
/// 返回 (x, y) 坐标
pub fn get_mouse_position() -> Result<(i32, i32), windows::core::Error> {
    use windows::Win32::Foundation::POINT;
    use windows::Win32::UI::WindowsAndMessaging::GetCursorPos;

    unsafe {
        let mut point = POINT::default();
        GetCursorPos(&mut point)?;
        Ok((point.x, point.y))
    }
}
