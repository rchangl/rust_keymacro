//! 主入口模块
//!
//! 应用程序入口点，负责初始化日志、加载配置并启动 GUI 应用。
//! Release 模式下隐藏控制台窗口，Debug 模式下保留以便查看日志
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use rust_keymacro::{bootstrap, logger, gui::MacroApp};

/// 应用程序主入口
fn main() -> eframe::Result {
    // 初始化日志系统
    logger::init_logger();

    // 加载配置文件
    let config = match bootstrap::load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            log::error!("加载配置文件失败: {}", e);
            bootstrap::show_error_dialog(&e);
            std::process::exit(1);
        }
    };

    // 原生选项
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 700.0])
            .with_min_inner_size([600.0, 500.0])
            .with_title("键盘宏控制台 - KeyMacro"),
        ..Default::default()
    };

    // 运行 eframe 应用
    eframe::run_native(
        "键盘宏控制台 - KeyMacro",
        native_options,
        Box::new(|cc| {
            Ok(Box::new(MacroApp::new(cc, config)))
        }),
    )
}
