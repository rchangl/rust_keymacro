//! YAML 编辑器模块
//!
//! 提供 YAML 配置文本的编辑功能

use crate::gui::MacroApp;
use egui;

/// 显示 YAML 配置编辑器
pub fn show(app: &mut MacroApp, ui: &mut egui::Ui) {
    ui.vertical(|ui| {
        ui.heading("宏配置编辑器");
        ui.label("编辑 YAML 配置文件，点击'应用配置'使更改生效");
        ui.add_space(10.0);
        
        // 显示配置错误
        show_config_error(app, ui);
        
        // 配置文本编辑器
        show_yaml_text_editor(app, ui);
        
        ui.add_space(10.0);
        
        // 操作按钮
        show_yaml_buttons(app, ui);
    });
}

/// 显示配置错误
fn show_config_error(app: &mut MacroApp, ui: &mut egui::Ui) {
    if app.show_config_error {
        ui.horizontal(|ui| {
            ui.colored_label(egui::Color32::RED, "⚠ 配置错误:");
        });
        egui::Frame::default()
            .fill(egui::Color32::from_rgb(255, 240, 240))
            .show(ui, |ui| {
                ui.colored_label(egui::Color32::RED, &app.config_error);
            });
        ui.add_space(10.0);
    }
}

/// 显示 YAML 文本编辑器
fn show_yaml_text_editor(app: &mut MacroApp, ui: &mut egui::Ui) {
    egui::Frame::default()
        .fill(egui::Color32::WHITE)
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    let text_edit = egui::TextEdit::multiline(&mut app.config_text)
                        .font(egui::TextStyle::Monospace)
                        .desired_rows(30)
                        .desired_width(f32::INFINITY)
                        .code_editor();
                    
                    ui.add(text_edit);
                });
        });
}

/// 显示 YAML 编辑器操作按钮
fn show_yaml_buttons(app: &mut MacroApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if ui.button("应用配置").clicked() {
            app.apply_config();
        }
        if ui.button("保存配置").clicked() {
            app.save_config();
        }
        ui.separator();
        if ui.button("格式化配置").clicked() {
            format_yaml(app);
        }
    });
}

/// 格式化 YAML 配置
fn format_yaml(app: &mut MacroApp) {
    if let Ok(config) = serde_yaml::from_str::<crate::config::Config>(&app.config_text) {
        if let Ok(formatted) = serde_yaml::to_string(&config) {
            app.config_text = formatted;
        }
    }
}
