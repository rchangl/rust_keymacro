//! Windows API 安全封装模块
//!
//! 本模块封装了所有与 Windows API 交互的 unsafe 代码，为上层模块提供安全接口。
//! 所有 unsafe 操作都限制在此模块内部，便于代码审查和维护。

pub mod window;
pub mod keyboard;

// 可以根据需要添加更多 Windows API 封装模块
// pub mod process;
// pub mod registry;
