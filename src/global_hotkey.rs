//! 应用级全局快捷键模块
//!
//! 使用 global-hotkey（Windows RegisterHotKey）注册全局快捷键，
//! 使程序在后台运行时（窗口不在前台）也能通过快捷键控制宏系统。
//!
//! # 设计说明
//!
//! `GlobalHotKeyManager` 不是 `Send`/`Sync`，且必须保持存活（drop 后已注册的
//! 热键会失效）。因此这里用 `Box::leak` 将管理器泄漏为 `'static` 引用，并借助
//! 一个 `Send + Sync` 的裸指针包装在全局静态中安全共享。所有对管理器的访问
//! 都在 `Mutex` 内进行，且注册完成后只做只读的 register/unregister 调用。

use crate::config::GlobalHotkeyConfig;
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use once_cell::sync::Lazy;
use std::sync::Mutex;

/// 从配置构建 HotKey
pub fn hotkey_from_config(cfg: &GlobalHotkeyConfig) -> HotKey {
    let mut mods = Modifiers::empty();
    for m in cfg.modifiers.split('+').map(|s| s.trim().to_lowercase()) {
        match m.as_str() {
            "ctrl" | "control" => mods |= Modifiers::CONTROL,
            "alt" => mods |= Modifiers::ALT,
            "shift" => mods |= Modifiers::SHIFT,
            "win" | "meta" | "super" => mods |= Modifiers::META,
            _ => {}
        }
    }
    let code = code_from_name(&cfg.key);
    HotKey::new(if mods.is_empty() { None } else { Some(mods) }, code)
}

/// 将键名（如 "q"、"f12"）解析为 Code
pub fn code_from_name(name: &str) -> Code {
    let n = name.trim().to_lowercase();
    use Code::*;
    // 字母键
    if let Some(c) = n.chars().next() {
        if c.is_ascii_alphabetic() && n.len() == 1 {
            let code = match c {
                'a' => KeyA,
                'b' => KeyB,
                'c' => KeyC,
                'd' => KeyD,
                'e' => KeyE,
                'f' => KeyF,
                'g' => KeyG,
                'h' => KeyH,
                'i' => KeyI,
                'j' => KeyJ,
                'k' => KeyK,
                'l' => KeyL,
                'm' => KeyM,
                'n' => KeyN,
                'o' => KeyO,
                'p' => KeyP,
                'q' => KeyQ,
                'r' => KeyR,
                's' => KeyS,
                't' => KeyT,
                'u' => KeyU,
                'v' => KeyV,
                'w' => KeyW,
                'x' => KeyX,
                'y' => KeyY,
                'z' => KeyZ,
                _ => unreachable!(),
            };
            return code;
        }
    }
    // 数字键
    if let Ok(digit) = n.parse::<u8>() {
        if digit <= 9 {
            return match digit {
                0 => Digit0,
                1 => Digit1,
                2 => Digit2,
                3 => Digit3,
                4 => Digit4,
                5 => Digit5,
                6 => Digit6,
                7 => Digit7,
                8 => Digit8,
                _ => Digit9,
            };
        }
    }
    // 功能键 F1-F24
    if let Some(rest) = n.strip_prefix('f') {
        if let Ok(num) = rest.parse::<u8>() {
            if (1..=24).contains(&num) {
                return match num {
                    1 => F1, 2 => F2, 3 => F3, 4 => F4, 5 => F5, 6 => F6,
                    7 => F7, 8 => F8, 9 => F9, 10 => F10, 11 => F11, 12 => F12,
                    13 => F13, 14 => F14, 15 => F15, 16 => F16, 17 => F17,
                    18 => F18, 19 => F19, 20 => F20, 21 => F21, 22 => F22,
                    23 => F23, _ => F24,
                };
            }
        }
    }
    // 常见特殊键
    match n.as_str() {
        "space" | "spacebar" => Space,
        "enter" | "return" => Enter,
        "esc" | "escape" => Escape,
        "tab" => Tab,
        "backspace" => Backspace,
        "delete" | "del" => Delete,
        "insert" | "ins" => Insert,
        "home" => Home,
        "end" => End,
        "pageup" => PageUp,
        "pagedown" => PageDown,
        "up" => ArrowUp,
        "down" => ArrowDown,
        "left" => ArrowLeft,
        "right" => ArrowRight,
        _ => KeyQ, // 未知键，回退到 Q
    }
}

/// 将修饰键组合格式化为可读文本（用于界面显示）
fn modifiers_text(mods: Modifiers) -> String {
    let mut parts: Vec<&str> = Vec::new();
    if mods.contains(Modifiers::CONTROL) {
        parts.push("Ctrl");
    }
    if mods.contains(Modifiers::ALT) {
        parts.push("Alt");
    }
    if mods.contains(Modifiers::SHIFT) {
        parts.push("Shift");
    }
    if mods.contains(Modifiers::META) {
        parts.push("Win");
    }
    parts.join("+")
}

/// 将 `HotKey` 格式化为可读文本，如 "Ctrl+Alt+Q"
pub fn hotkey_text(hotkey: &HotKey) -> String {
    let code_text = format!("{:?}", hotkey.key).trim_start_matches("Key").to_string();
    let mods_text = modifiers_text(hotkey.mods);
    if mods_text.is_empty() {
        code_text
    } else {
        format!("{}+{}", mods_text, code_text)
    }
}

/// 全局热键管理器内部状态
///
/// 通过 `Mutex<*mut GlobalHotKeyManager>` 实现 `Send + Sync`（裸指针是 `Send`）。
/// 访问管理器总在锁内进行，泄漏的内存永不释放，因此裸指针始终有效。
struct GlobalHotkeyState {
    manager: *mut GlobalHotKeyManager,
    current: Option<HotKey>,
}

// 确保该状态可在线程间共享（管理器访问均为只读的 register/unregister，安全）
unsafe impl Send for GlobalHotkeyState {}
unsafe impl Sync for GlobalHotkeyState {}

/// 全局快捷键状态单例
static STATE: Lazy<Mutex<GlobalHotkeyState>> = Lazy::new(|| {
    // 泄漏管理器，保证其永久存活
    let manager = Box::leak(Box::new(
        GlobalHotKeyManager::new().expect("创建全局快捷键管理器失败"),
    ));
    Mutex::new(GlobalHotkeyState {
        manager: manager as *mut GlobalHotKeyManager,
        current: None,
    })
});

/// 当前的全局快捷键（默认 Ctrl+Alt+Q）
pub fn default_hotkey() -> HotKey {
    HotKey::new(
        Some(Modifiers::CONTROL | Modifiers::ALT),
        Code::KeyQ,
    )
}

/// 获取当前生效的全局快捷键
pub fn current_hotkey() -> HotKey {
    STATE
        .lock()
        .ok()
        .and_then(|s| s.current.clone())
        .unwrap_or_else(default_hotkey)
}

/// 启动全局快捷键监听
///
/// 注册配置指定的快捷键（未配置则使用默认 `Ctrl+Alt+Q`）并启动事件监听线程。
///
/// # 参数
///
/// * `config` - 配置中的全局快捷键（可选）
///
/// # 返回值
///
/// 成功返回 `Ok(快捷键描述)`；注册失败（例如被其他程序占用）返回 `Err`。
pub fn start_global_hotkeys(config: Option<&GlobalHotkeyConfig>) -> Result<String, String> {
    let hotkey = match config {
        Some(cfg) => hotkey_from_config(cfg),
        None => default_hotkey(),
    };
    register_hotkey(hotkey)?;
    start_listener();
    Ok(hotkey_text(&current_hotkey()))
}

/// 注册/替换当前全局快捷键
///
/// 如果已有快捷键生效，会先注销旧的，再注册新的。
///
/// # 参数
///
/// * `hotkey` - 要注册的快捷键
///
/// # 返回值
///
/// 成功返回 `Ok(快捷键描述)`；被占用或注册失败返回 `Err(原因)`。
pub fn set_hotkey(hotkey: HotKey) -> Result<String, String> {
    let mut state = STATE
        .lock()
        .map_err(|_| "全局快捷键状态锁不可用".to_string())?;

    // 若与当前相同则无需处理
    if state.current.as_ref() == Some(&hotkey) {
        return Ok(hotkey_text(&hotkey));
    }

    // 先尝试注册新的；若失败（冲突）则不改变当前热键，避免快捷键丢失
    unsafe {
        (*state.manager)
            .register(hotkey.clone())
            .map_err(|e| format!("快捷键 {} 注册失败：可能已被其他程序占用（{}）", hotkey_text(&hotkey), e))?;
    }

    // 注册成功后再注销旧的（忽略错误，旧的可能已失效）
    if let Some(old) = state.current.take() {
        unsafe {
            let _ = (*state.manager).unregister(old);
        }
    }

    state.current = Some(hotkey);
    Ok(hotkey_text(&hotkey))
}

/// 注册快捷键（不替换现有）
fn register_hotkey(hotkey: HotKey) -> Result<(), String> {
    let mut state = STATE
        .lock()
        .map_err(|_| "全局快捷键状态锁不可用".to_string())?;

    unsafe {
        (*state.manager)
            .register(hotkey.clone())
            .map_err(|e| format!("快捷键 {} 注册失败：可能已被其他程序占用（{}）", hotkey_text(&hotkey), e))?;
    }
    state.current = Some(hotkey);
    Ok(())
}

/// 启动事件监听线程（进程生命周期内只启动一次）
fn start_listener() {
    static LISTENER_STARTED: std::sync::atomic::AtomicBool =
        std::sync::atomic::AtomicBool::new(false);
    if LISTENER_STARTED.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return; // 已启动
    }

    std::thread::spawn(|| {
        let receiver = GlobalHotKeyEvent::receiver();
        loop {
            if let Ok(event) = receiver.recv() {
                if event.state() == HotKeyState::Pressed {
                    log::info!("全局快捷键触发：切换宏总开关");
                    crate::macros::toggle_macro_state();
                }
            }
        }
    });
}
