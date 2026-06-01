//! 宏详情编辑模块
//!
//! 提供宏的触发源配置和操作类型选择

use crate::config::*;
use crate::visual_editor::VisualEditor;
use crate::visual_editor::key_selector;

/// 显示宏详情编辑（右侧面板）
pub fn show_macro_detail(
    editor: &mut VisualEditor,
    ui: &mut egui::Ui,
    status_message: &mut String,
    log_messages: &mut Vec<String>,
) {
    egui::Frame::group(ui.style()).show(ui, |ui| {
        ui.heading("宏详情编辑");
        ui.add_space(5.0);
        
        if let Some(idx) = editor.selected_macro {
            if idx < editor.config.hotkeys.len() {
                edit_macro_detail(editor, ui, idx, status_message, log_messages);
            }
        } else {
            ui.label("💡 提示：");
            ui.label("• 点击左侧宏列表项选择要编辑的宏");
            ui.label("• 在右侧编辑宏的详细配置");
        }
    });
}

/// 编辑宏详情
fn edit_macro_detail(
    editor: &mut VisualEditor,
    ui: &mut egui::Ui,
    idx: usize,
    status_message: &mut String,
    log_messages: &mut Vec<String>,
) {
    ui.vertical(|ui| {
        ui.label(format!("编辑宏 #{}", idx + 1));
        ui.separator();
        
        // 触发源类型
        ui.label("触发源类型:");
        ui.horizontal(|ui| {
            ui.radio_value(&mut editor.edit_is_keyboard, true, "⌨ 键盘");
            ui.radio_value(&mut editor.edit_is_keyboard, false, "🎮 手柄");
        });
        
        ui.add_space(5.0);
        
        // 触发键 - 使用局部变量避免借用冲突
        ui.label("触发键:");
        let mut temp_key = editor.edit_key.clone();
        
        if editor.edit_is_keyboard {
            key_selector::show_keyboard_selector(ui, &mut temp_key);
        } else {
            key_selector::show_gamepad_selector(ui, &mut temp_key);
        }
        editor.edit_key = temp_key;
        
        ui.add_space(5.0);
        
        // 操作类型
        ui.label("操作类型:");
        ui.horizontal(|ui| {
            ui.radio_value(&mut editor.edit_is_sequence, true, "📝 按键序列");
            ui.radio_value(&mut editor.edit_is_sequence, false, "✍ 输入文本");
        });
        
        // 自动应用触发配置
        apply_trigger_config(editor, idx);
            
        ui.add_space(10.0);
        ui.separator();
        
        // 根据操作类型显示不同的编辑器
        if editor.edit_is_sequence {
            super::step_editor::edit_sequence_action(editor, ui, idx, status_message, log_messages);
        } else {
            super::step_editor::edit_type_text_action(editor, ui, idx, status_message, log_messages);
        }
    });
}

/// 自动应用触发配置（仅在值真正变化时触发更新）
fn apply_trigger_config(editor: &mut VisualEditor, idx: usize) {
    if let Some(hotkey) = editor.config.hotkeys.get_mut(idx) {
        // 计算新的值
        let new_trigger = if editor.edit_is_keyboard {
            TriggerSource::Keyboard { key: editor.edit_key.clone() }
        } else {
            TriggerSource::Gamepad { key: editor.edit_key.clone() }
        };
        let new_action = if editor.edit_is_sequence { "sequence" } else { "type_text" }.to_string();
        
        // 检查触发源和操作类型是否变化（TriggerSource 未实现 PartialEq，手动比较）
        let trigger_changed = match (&hotkey.trigger, &new_trigger) {
            (TriggerSource::Keyboard { key: a }, TriggerSource::Keyboard { key: b }) => a != b,
            (TriggerSource::Gamepad { key: a }, TriggerSource::Gamepad { key: b }) => a != b,
            _ => true, // 类型不同，肯定变了
        };
        let action_changed = hotkey.action != new_action;
        
        if trigger_changed || action_changed {
            hotkey.trigger = new_trigger;
            hotkey.action = new_action;
            
            // 如果切换到type_text但没有params，初始化
            if !editor.edit_is_sequence {
                if !matches!(hotkey.params, ActionParams::TypeText(_)) {
                    hotkey.params = ActionParams::TypeText(TypeTextParams {
                        text: "Hello World".to_string(),
                        delay: Some(DelayConfig::Fixed(10)),
                    });
                }
            }
            
            editor.config_changed = true;
        }
    }
}
