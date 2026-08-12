//! 按键选择器模块
//!
//! 提供键盘和手柄按键的可视化选择器（二维键盘图）

/// 绘制键盘按键选择界面（在 Window 内部调用）
pub fn show_keyboard_content(ui: &mut egui::Ui, current_key: &mut String) -> bool {
    let mut selected = false;

    ui.set_min_width(620.0);

    // F 键行
    ui.label(egui::RichText::new("功能键").strong());
    key_row(ui, current_key, &[
        ("Esc", "Escape"), ("F1", "F1"), ("F2", "F2"), ("F3", "F3"), ("F4", "F4"),
        ("F5", "F5"), ("F6", "F6"), ("F7", "F7"), ("F8", "F8"),
        ("F9", "F9"), ("F10", "F10"), ("F11", "F11"), ("F12", "F12"),
    ], &mut selected);

    ui.add_space(4.0);
    ui.separator();
    ui.add_space(4.0);

    // 数字行
    ui.label(egui::RichText::new("数字/符号").strong());
    key_row(ui, current_key, &[
        ("`", "`"), ("1", "1"), ("2", "2"), ("3", "3"), ("4", "4"),
        ("5", "5"), ("6", "6"), ("7", "7"), ("8", "8"), ("9", "9"),
        ("0", "0"), ("-", "-"), ("=", "="), ("Backsp", "Backspace"),
    ], &mut selected);

    ui.add_space(4.0);

    // QWERTY 行
    ui.label(egui::RichText::new("QWERTY").strong());
    key_row(ui, current_key, &[
        ("Tab", "Tab"),
        ("Q", "Q"), ("W", "W"), ("E", "E"), ("R", "R"), ("T", "T"),
        ("Y", "Y"), ("U", "U"), ("I", "I"), ("O", "O"), ("P", "P"),
        ("[", "["), ("]", "]"), ("\\", "\\"),
    ], &mut selected);

    ui.add_space(4.0);

    // A 行
    key_row(ui, current_key, &[
        ("Caps", "CapsLock"),
        ("A", "A"), ("S", "S"), ("D", "D"), ("F", "F"), ("G", "G"),
        ("H", "H"), ("J", "J"), ("K", "K"), ("L", "L"),
        (";", ";"), ("'", "'"), ("Enter", "Enter"),
    ], &mut selected);

    ui.add_space(4.0);

    // ZXCV 行
    key_row(ui, current_key, &[
        ("Shift", "Shift"),
        ("Z", "Z"), ("X", "X"), ("C", "C"), ("V", "V"), ("B", "B"),
        ("N", "N"), ("M", "M"), (",", ","), (".", "."), ("/", "/"),
        ("Shift", "Shift"),
    ], &mut selected);

    ui.add_space(4.0);
    ui.separator();
    ui.add_space(4.0);

    // 底行
    ui.label(egui::RichText::new("修饰键").strong());
    key_row(ui, current_key, &[
        ("Ctrl", "Ctrl"), ("Win", "Win"), ("Alt", "Alt"),
        ("Space", "Space"),
        ("Alt", "Alt"), ("Menu", "Menu"), ("Ctrl", "Ctrl"),
    ], &mut selected);

    ui.add_space(8.0);
    ui.separator();
    ui.add_space(4.0);

    // 方向键
    ui.label(egui::RichText::new("方向键").strong());
    key_row(ui, current_key, &[
        ("↑", "Up"), ("↓", "Down"), ("←", "Left"), ("→", "Right"),
    ], &mut selected);

    ui.add_space(4.0);
    ui.separator();
    ui.add_space(4.0);

    // 导航键
    ui.label(egui::RichText::new("导航键").strong());
    key_row(ui, current_key, &[
        ("Insert", "Insert"), ("Home", "Home"), ("PageUp", "PageUp"),
        ("Delete", "Delete"), ("End", "End"), ("PageDown", "PageDown"),
    ], &mut selected);

    selected
}

/// 绘制手柄按键选择界面（在 Window 内部调用）
pub fn show_gamepad_content(ui: &mut egui::Ui, current_key: &mut String) -> bool {
    let mut selected = false;

    ui.set_min_width(400.0);

    ui.label(egui::RichText::new("动作键").strong());
    key_row(ui, current_key, &[
        ("A", "A"), ("B", "B"), ("X", "X"), ("Y", "Y"),
    ], &mut selected);

    ui.add_space(4.0);

    ui.label(egui::RichText::new("肩键").strong());
    key_row(ui, current_key, &[
        ("LB", "LB"), ("RB", "RB"), ("LT", "LT"), ("RT", "RT"),
    ], &mut selected);

    ui.add_space(4.0);

    ui.label(egui::RichText::new("功能键").strong());
    key_row(ui, current_key, &[
        ("Start", "Start"), ("Back", "Back"), ("Guide", "Guide"),
    ], &mut selected);

    ui.add_space(4.0);

    ui.label(egui::RichText::new("摇杆").strong());
    key_row(ui, current_key, &[
        ("LS", "LS"), ("RS", "RS"),
    ], &mut selected);

    ui.add_space(4.0);

    ui.label(egui::RichText::new("十字键").strong());
    key_row(ui, current_key, &[
        ("↑", "DUp"), ("↓", "DDown"), ("←", "DLeft"), ("→", "DRight"),
    ], &mut selected);

    selected
}

/// 绘制一行按键
fn key_row(
    ui: &mut egui::Ui,
    current_key: &mut String,
    keys: &[(&str, &str)], // (显示标签, 实际键值)
    selected: &mut bool,
) {
    ui.horizontal_wrapped(|ui| {
        for (label, value) in keys {
            let is_current = *current_key == *value;
            let btn = egui::Button::new(*label)
                .min_size(egui::vec2(36.0, 32.0))
                .fill(if is_current {
                    ui.style().visuals.selection.bg_fill
                } else {
                    ui.style().visuals.widgets.inactive.bg_fill
                })
                .stroke(if is_current {
                    egui::Stroke::new(2.0, ui.style().visuals.selection.stroke.color)
                } else {
                    egui::Stroke::NONE
                });
            if ui.add(btn).clicked() {
                *current_key = value.to_string();
                *selected = true;
            }
        }
    });
}
