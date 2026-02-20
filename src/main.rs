//! 主入口模块
//!
//! 应用程序入口点，负责初始化日志、加载配置并启动应用。

#![windows_subsystem = "windows"] // 隐藏控制台窗口

use rust_keymacro::{bootstrap, logger};

/// 应用程序主入口
fn main() {
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

    // 初始化并运行应用
    if let Err(e) = bootstrap::run_application(config) {
        log::error!("应用运行失败: {}", e);
        bootstrap::show_error_dialog(&e);
        std::process::exit(1);
    }
}
