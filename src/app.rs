//! 应用程序核心模块
//!
//! 管理系统托盘应用的主逻辑、事件处理和生命周期

use crate::macros::{init_keyboard_macro_system, set_macro_enabled, cleanup_keyboard_hook};
use crate::config::Config;
use tray_icon::{
    menu::{Menu, MenuItem, MenuId},
    TrayIcon, TrayIconBuilder,
};
use winit::{
    application::ApplicationHandler,
    event_loop::{ActiveEventLoop, ControlFlow},
};
use global_hotkey::{GlobalHotKeyEvent, HotKeyState};
use windows::Win32::UI::WindowsAndMessaging::HHOOK;

/// 托盘应用程序主结构体
///
/// 负责处理系统托盘图标、菜单、热键和键盘宏
pub struct TrayApp {
    /// 退出菜单项 ID
    quit_item_id: MenuId,
    
    /// 菜单事件接收器
    menu_event_receiver: tray_icon::menu::MenuEventReceiver,
    
    /// 托盘图标事件接收器
    tray_event_receiver: tray_icon::TrayIconEventReceiver,
    
    /// 热键管理器（保持活动以监听热键）
    _hotkey_manager: global_hotkey::GlobalHotKeyManager,
    
    /// 当前状态（0 或 1）
    toggle_state: bool,
    
    /// 托盘图标
    tray_icon: Option<tray_icon::TrayIcon>,
    
    /// 状态 0 的图标（红色）
    icon_state_0: tray_icon::Icon,
    
    /// 状态 1 的图标（绿色）
    icon_state_1: tray_icon::Icon,
    
    /// 键盘钩子句柄（程序退出时清理）
    keyboard_hook: Option<HHOOK>,
    
    /// 键盘宏配置
    config: Config,
}

impl TrayApp {
    /// 创建新的托盘应用实例
    ///
    /// # 参数
    ///
    /// * `quit_item_id` - 退出菜单项 ID
    /// * `menu_event_receiver` - 菜单事件接收器
    /// * `tray_event_receiver` - 托盘事件接收器
    /// * `hotkey_manager` - 热键管理器
    /// * `tray_icon` - 托盘图标
    /// * `icon_state_0` - 状态 0 图标
    /// * `icon_state_1` - 状态 1 图标
    /// * `config` - 键盘宏配置
    pub fn new(
        quit_item_id: MenuId,
        menu_event_receiver: tray_icon::menu::MenuEventReceiver,
        tray_event_receiver: tray_icon::TrayIconEventReceiver,
        hotkey_manager: global_hotkey::GlobalHotKeyManager,
        tray_icon: tray_icon::TrayIcon,
        icon_state_0: tray_icon::Icon,
        icon_state_1: tray_icon::Icon,
        config: Config,
    ) -> Self {
        Self {
            quit_item_id,
            menu_event_receiver,
            tray_event_receiver,
            _hotkey_manager: hotkey_manager,
            toggle_state: true, // 默认开启
            tray_icon: Some(tray_icon),
            icon_state_0,
            icon_state_1,
            keyboard_hook: None,
            config,
        }
    }
}

impl ApplicationHandler for TrayApp {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        // 初始化键盘宏系统（传递配置）
        if self.keyboard_hook.is_none() {
            self.keyboard_hook = init_keyboard_macro_system(self.config.clone());
        }
    }

    fn window_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        _event: winit::event::WindowEvent,
    ) {
        // 本应用无窗口，忽略窗口事件
    }

    fn user_event(&mut self, _event_loop: &ActiveEventLoop, _event: ()) {
        // 无自定义用户事件
    }

    fn new_events(&mut self, event_loop: &ActiveEventLoop, _cause: winit::event::StartCause) {
        // 等待模式，减少 CPU 占用
        event_loop.set_control_flow(ControlFlow::Wait);

        // 处理菜单事件（退出）
        if let Ok(menu_event) = self.menu_event_receiver.try_recv() {
            if menu_event.id == self.quit_item_id {
                // 清理钩子并退出
                if let Some(hook) = self.keyboard_hook.take() {
                    cleanup_keyboard_hook(hook);
                }
                event_loop.exit();
            }
        }

        // 忽略托盘图标事件（右键自动显示菜单）
        let _ = self.tray_event_receiver.try_recv();

        // 处理热键事件（Ctrl+`）
        while let Ok(hotkey_event) = GlobalHotKeyEvent::receiver().try_recv() {
            if hotkey_event.state() == HotKeyState::Pressed {
                // 切换状态
                self.toggle_state = !self.toggle_state;
                
                // 更新宏状态和托盘
                set_macro_enabled(self.toggle_state);
                
                let state_text = if self.toggle_state { "1" } else { "0" };
                if let Some(tray_icon) = &self.tray_icon {
                    let _ = tray_icon.set_tooltip(Some(&format!("状态: {}", state_text)));
                    let new_icon = if self.toggle_state { &self.icon_state_1 } else { &self.icon_state_0 };
                    let _ = tray_icon.set_icon(Some(new_icon.clone()));
                }
                
                // 显示屏幕提示
                crate::overlay::show_overlay(state_text);
                break;
            }
        }
    }
}

/// 初始化并创建托盘图标
/// 
/// # 返回
/// 
/// 返回一个元组，包含：
/// - 托盘图标对象（需要保持活动状态）
/// - 退出菜单项的ID（用于后续事件处理）
/// - 状态0的图标（红色）
/// - 状态1的图标（绿色）
/// 
/// # 注意
/// 
/// 托盘图标对象必须保持活动状态，否则托盘图标会消失
pub fn init_tray_icon() -> (TrayIcon, MenuId, tray_icon::Icon, tray_icon::Icon) {
    // 创建托盘右键菜单和"退出"菜单项
    let tray_menu = Menu::new();
    let quit_item = MenuItem::new("退出", true, None);
    let quit_item_id = quit_item.id().clone();
    
    // 将菜单项添加到菜单中
    tray_menu.append(&quit_item).unwrap();

    // 创建两种状态的图标
    let icon_state_0 = load_icon(false); // 状态0 - 红色
    let icon_state_1 = load_icon(true);  // 状态1 - 绿色

    // 创建托盘图标（菜单所有权已转移，无需返回）
    // 默认使用状态1的图标（绿色）
    let tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip("状态: 1") // 默认状态为开 (1)
        .with_icon(icon_state_1.clone())
        .build()
        .expect("Failed to create tray icon");

    (tray_icon, quit_item_id, icon_state_0, icon_state_1)
}

/// 加载并创建托盘图标
/// 
/// # 参数
/// 
/// * `is_state_1` - 是否为状态1（true=绿色，false=红色）
/// 
/// # 返回
/// 
/// 返回一个托盘图标对象，用于在系统托盘中显示
/// 
/// # 说明
/// 
/// 创建一个 16x16 像素的方块图标。在实际应用中，
/// 可以加载自定义的图标文件（如 .ico 格式）。
/// 每个像素包含 4 个字节（R, G, B, A）。
fn load_icon(is_state_1: bool) -> tray_icon::Icon {
    const SIZE: usize = 16;
    
    // 根据状态选择颜色：状态1为绿色，状态0为红色
    let pixel_color = if is_state_1 {
        [0, 255, 0, 255] // 绿色像素: R=0, G=255, B=0, A=255
    } else {
        [255, 0, 0, 255] // 红色像素: R=255, G=0, B=0, A=255
    };
    
    // 创建 16x16 的图标数据（256 个像素）
    let rgba: Vec<u8> = std::iter::repeat(&pixel_color)
        .take(SIZE * SIZE)
        .flatten()
        .copied()
        .collect();
    
    tray_icon::Icon::from_rgba(rgba, SIZE as u32, SIZE as u32)
        .expect("Failed to create icon from RGBA data")
}
