//! 主入口模块
//!
//! 负责初始化和启动托盘应用，显示系统托盘图标，
//! 响应 Ctrl+` 快捷键切换状态，并在屏幕中央显示提示。

#![windows_subsystem = "windows"] // 隐藏控制台窗口

mod app;
mod overlay;
mod macros;
mod winapi;
mod config;
mod bootstrap;

/// 应用程序主入口
///
/// 初始化并启动托盘应用，所有具体逻辑由模块化函数处理
fn main() {
    // 加载配置文件
    let config = match bootstrap::load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            bootstrap::show_error_dialog(&e);
            std::process::exit(1);
        }
    };

    // 初始化并运行应用
    if let Err(e) = bootstrap::run_application(config) {
        bootstrap::show_error_dialog(&e);
        std::process::exit(1);
    }
}

