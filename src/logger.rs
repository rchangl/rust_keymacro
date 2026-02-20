//! 日志系统初始化模块
//!
//! 负责根据编译模式初始化日志系统

use simplelog::{ConfigBuilder, LevelFilter, WriteLogger};

#[cfg(debug_assertions)]
use std::fs::File;

/// 初始化日志系统，将日志写入文件
///
/// Debug 模式下输出所有日志，Release 模式下不输出任何日志
pub fn init_logger() {
    #[cfg(debug_assertions)]
    {
        let log_path = std::env::current_exe()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .parent()
            .unwrap_or(std::path::Path::new("."))
            .join("app.log");

        let mut config_builder = ConfigBuilder::new();
        config_builder.set_time_offset_to_local().ok();

        let _ = WriteLogger::init(
            LevelFilter::Debug,
            config_builder.build(),
            File::create(&log_path).unwrap_or_else(|_| {
                File::create("app.log").expect("无法创建日志文件")
            }),
        );

        log::info!("日志系统初始化完成 (Debug 模式)，日志文件: {:?}", log_path);
    }

    #[cfg(not(debug_assertions))]
    {
        // Release 模式：不输出任何日志
        // 初始化一个空的日志系统，避免 log::xxx 调用 panic
        let _ = WriteLogger::init(
            LevelFilter::Off,
            ConfigBuilder::new().build(),
            std::io::sink(),  // 输出到空设备，不创建日志文件
        );
    }
}
