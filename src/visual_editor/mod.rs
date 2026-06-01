//! 可视化配置编辑器模块
//!
//! 提供表单式的GUI界面来编辑宏配置，替代YAML文本编辑

pub mod macro_list;
pub mod macro_editor;
pub mod step_editor;
pub mod key_selector;

use crate::config::*;

/// 可视化编辑器状态
pub struct VisualEditor {
    /// 当前配置
    pub config: Config,
    /// 当前选中的宏索引
    selected_macro: Option<usize>,
    /// 编辑状态：触发源类型 (true=键盘, false=手柄)
    edit_is_keyboard: bool,
    /// 编辑状态：触发键
    edit_key: String,
    /// 编辑状态：操作类型 (true=序列, false=文本)
    edit_is_sequence: bool,
    /// 配置是否已更改（需要重新初始化系统）
    pub config_changed: bool,
}

impl VisualEditor {
    /// 创建新的编辑器实例
    pub fn new(config: Config) -> Self {
        Self {
            config,
            selected_macro: None,
            edit_is_keyboard: true,
            edit_key: "F1".to_string(),
            edit_is_sequence: true,
            config_changed: false,
        }
    }
    
    /// 当选择不同的宏时，从配置加载编辑状态
    fn load_edit_state(&mut self, idx: usize) {
        if idx < self.config.hotkeys.len() {
            let hotkey = &self.config.hotkeys[idx];
            self.edit_is_keyboard = matches!(&hotkey.trigger, TriggerSource::Keyboard { .. });
            self.edit_key = match &hotkey.trigger {
                TriggerSource::Keyboard { key } => key.clone(),
                TriggerSource::Gamepad { key } => key.clone(),
            };
            self.edit_is_sequence = hotkey.action == "sequence";
        }
    }
    
    /// 显示编辑器UI
    pub fn show(&mut self, ui: &mut egui::Ui, status_message: &mut String, log_messages: &mut Vec<String>) {
        ui.vertical(|ui| {
            ui.heading("🎯 宏配置可视化编辑器");
            ui.add_space(10.0);
            
            // 使用columns创建左右分栏
            ui.columns(2, |columns| {
                // 左侧：宏列表
                columns[0].vertical(|ui| {
                    self.show_macro_list(ui, status_message, log_messages);
                });
                
                // 右侧：宏详情编辑
                columns[1].vertical(|ui| {
                    self.show_macro_detail(ui, status_message, log_messages);
                });
            });
        });
    }
    
    /// 显示宏列表（左侧面板）
    fn show_macro_list(&mut self, ui: &mut egui::Ui, status_message: &mut String, log_messages: &mut Vec<String>) {
        use macro_list::show_macro_list;
        show_macro_list(self, ui, status_message, log_messages);
    }
    
    /// 显示宏详情编辑（右侧面板）
    fn show_macro_detail(&mut self, ui: &mut egui::Ui, status_message: &mut String, log_messages: &mut Vec<String>) {
        use macro_editor::show_macro_detail;
        show_macro_detail(self, ui, status_message, log_messages);
    }
}
