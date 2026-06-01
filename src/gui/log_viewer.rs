//! 日志查看器模块
//!
//! 提供运行日志的显示和清空功能

use crate::gui::MacroApp;
use egui;

/// 显示日志查看器
pub fn show(app: &mut MacroApp, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        ui.heading("运行日志");
        ui.add_space(10.0);
        
        // 清空日志按钮
        ui.horizontal(|ui| {
            if ui.button("清空日志").clicked() {
                app.log_messages.clear();
                app.log_messages.push("[INFO] 日志已清空".to_string());
            }
        });
        
        ui.add_space(10.0);
        
        // 日志滚动区域
        show_log_messages(app, ui);
    });
}

/// 显示日志消息
fn show_log_messages(app: &mut MacroApp, ui: &mut egui::Ui) {
    egui::Frame::default()
        .fill(egui::Color32::WHITE)
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for msg in &app.log_messages {
                        let color = if msg.contains("[ERROR]") {
                            egui::Color32::RED
                        } else if msg.contains("[WARN]") {
                            egui::Color32::YELLOW
                        } else {
                            egui::Color32::DARK_GRAY
                        };
                        
                        ui.colored_label(color, msg);
                    }
                });
        });
}
