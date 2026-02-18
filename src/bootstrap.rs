//! 应用程序启动模块
//!
//! 负责加载配置、初始化应用和错误处理

use crate::app::TrayApp;
use crate::config::Config;
use winit::{
    event_loop::EventLoop,
    platform::windows::EventLoopBuilderExtWindows,
};
use global_hotkey::GlobalHotKeyManager;

/// 加载配置文件
///
/// 优先从当前工作目录加载，找不到则从可执行文件所在目录加载
///
/// # 返回值
///
/// 成功返回配置对象，失败返回错误信息
pub fn load_config() -> Result<Config, String> {
    // 获取当前工作目录
    let current_dir = std::env::current_dir()
        .map_err(|_| "获取当前工作目录失败".to_string())?;
    
    let current_dir_config = current_dir.join("config.yaml");
    
    // 首先尝试从工作目录加载
    if current_dir_config.exists() {
        return Config::from_file(current_dir_config.to_str().unwrap())
            .map_err(|e| format!(
                "加载配置文件失败: {}\n\n配置文件路径: {}\n\n当前工作目录: {}",
                e,
                current_dir_config.display(),
                current_dir.display()
            ));
    }
    
    // 如果工作目录没有，则从exe所在目录加载
    let exe_path = std::env::current_exe()
        .map_err(|_| "获取可执行文件路径失败".to_string())?;
    
    let exe_dir = exe_path.parent()
        .ok_or("获取可执行文件目录失败".to_string())?;
    
    let exe_dir_config = exe_dir.join("config.yaml");
    
    Config::from_file(exe_dir_config.to_str().unwrap())
        .map_err(|e| format!(
            "加载配置文件失败: {}\n\n请确保 config.yaml 文件存在于以下任一目录:\n1. 工作目录: {}\n2. 程序目录: {}\n\n当前工作目录: {}",
            e,
            current_dir_config.display(),
            exe_dir_config.display(),
            current_dir.display()
        ))
}

/// 运行应用程序
///
/// 初始化并启动托盘应用的主循环
///
/// # 参数
///
/// * `config` - 键盘宏配置
///
/// # 返回值
///
/// 运行成功返回 Ok，失败返回错误信息
pub fn run_application(config: Config) -> Result<(), String> {
    // 创建事件循环
    let event_loop = EventLoop::builder()
        .with_any_thread(true)
        .build()
        .map_err(|_| "创建事件循环失败".to_string())?;

    // 初始化托盘图标
    let (tray_icon, quit_item_id, icon_state_0, icon_state_1) = crate::app::init_tray_icon();

    // 注册全局热键
    let hotkey_manager = GlobalHotKeyManager::new()
        .map_err(|_| "创建热键管理器失败".to_string())?;
    
    let hotkey = global_hotkey::hotkey::HotKey::new(
        Some(global_hotkey::hotkey::Modifiers::CONTROL),
        global_hotkey::hotkey::Code::Backquote
    );
    
    hotkey_manager.register(hotkey)
        .map_err(|_| "注册热键失败".to_string())?;

    // 创建应用实例并运行
    let mut app = TrayApp::new(
        quit_item_id,
        tray_icon::menu::MenuEvent::receiver().clone(),
        tray_icon::TrayIconEvent::receiver().clone(),
        hotkey_manager,
        tray_icon,
        icon_state_0,
        icon_state_1,
        config,
    );

    event_loop.run_app(&mut app)
        .map_err(|_| "运行事件循环失败".to_string())?;
    
    Ok(())
}

/// 显示错误对话框
///
/// 使用 Windows MessageBox 显示错误信息
///
/// # 参数
///
/// * `message` - 错误消息
pub fn show_error_dialog(message: &str) {
    use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};
    use windows::Win32::Foundation::HWND;

    let message_wide: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
    let title_wide: Vec<u16> = "错误".encode_utf16().chain(std::iter::once(0)).collect();

    unsafe {
        MessageBoxW(
            HWND(std::ptr::null_mut()),
            windows::core::PCWSTR(message_wide.as_ptr()),
            windows::core::PCWSTR(title_wide.as_ptr()),
            MB_ICONERROR | MB_OK
        );
    }
}
