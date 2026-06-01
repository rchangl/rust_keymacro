//! 按键选择器模块
//!
//! 提供键盘和手柄按键的选择器

use egui::ComboBox;

/// 显示键盘按键选择器
pub fn show_keyboard_selector(ui: &mut egui::Ui, key: &mut String) {
    let keys = vec![
        "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M",
        "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z",
        "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
        "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12",
        "Space", "Enter", "Tab", "Backspace", "Escape",
        "Shift", "Ctrl", "Alt",
        "`", "'",
    ];
    
    ComboBox::from_label("选择按键")
        .selected_text(key.clone())
        .show_ui(ui, |ui| {
            for k in keys {
                ui.selectable_value(key, k.to_string(), k);
            }
        });
}

/// 显示手柄按键选择器
pub fn show_gamepad_selector(ui: &mut egui::Ui, key: &mut String) {
    let keys = vec![
        "A", "B", "X", "Y",
        "LB", "RB", "LT", "RT",
        "Start", "Back", "Guide",
        "LS", "RS",
        "DUp", "DDown", "DLeft", "DRight",
    ];
    
    ComboBox::from_label("选择手柄按键")
        .selected_text(key.clone())
        .show_ui(ui, |ui| {
            for k in keys {
                ui.selectable_value(key, k.to_string(), k);
            }
        });
}
