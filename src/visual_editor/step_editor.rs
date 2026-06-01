//! 步骤编辑模块
//!
//! 提供按键序列的步骤编辑和详情编辑功能

use crate::config::*;
use crate::visual_editor::VisualEditor;
use egui::Id;

/// 键盘按键列表
fn keyboard_key_list() -> Vec<&'static str> {
    vec![
        "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M",
        "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z",
        "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
        "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12",
        "Space", "Enter", "Tab", "Backspace", "Escape",
        "Shift", "Ctrl", "Alt",
        "`", "'",
    ]
}

/// 编辑输入文本操作
pub fn edit_type_text_action(
    editor: &mut VisualEditor,
    ui: &mut egui::Ui,
    idx: usize,
    status_message: &mut String,
    log_messages: &mut Vec<String>,
) {
    ui.label("输入文本配置:");
    ui.separator();
    
    if let ActionParams::TypeText(params) = &editor.config.hotkeys[idx].params {
        let mut text = params.text.clone();
        let mut delay_ms = match &params.delay {
            Some(DelayConfig::Fixed(ms)) => *ms,
            _ => 10,
        };
        let mut min_delay = 5;
        let mut max_delay = 15;
        let mut use_range = false;
        
        if let Some(DelayConfig::Range { min, max }) = &params.delay {
            min_delay = *min;
            max_delay = *max;
            use_range = true;
        }
        
        ui.label("要输入的文本:");
        ui.add(egui::TextEdit::multiline(&mut text)
            .desired_rows(3)
            .desired_width(f32::INFINITY));
        
        ui.label("字符间延迟 (ms):");
        
        ui.horizontal(|ui| {
            ui.radio_value(&mut use_range, false, "固定延迟");
            ui.radio_value(&mut use_range, true, "随机范围");
        });
        
        if use_range {
            ui.horizontal(|ui| {
                ui.label("最小:");
                ui.add(egui::DragValue::new(&mut min_delay).range(0..=10000));
                ui.label("最大:");
                ui.add(egui::DragValue::new(&mut max_delay).range(0..=10000));
            });
        } else {
            ui.add(egui::DragValue::new(&mut delay_ms).range(0..=10000));
        }
        
        if ui.button("✓ 应用文本配置").clicked() {
            let delay = if use_range {
                Some(DelayConfig::Range { min: min_delay, max: max_delay })
            } else {
                Some(DelayConfig::Fixed(delay_ms))
            };
            
            if let Some(hotkey) = editor.config.hotkeys.get_mut(idx) {
                hotkey.params = ActionParams::TypeText(TypeTextParams {
                    text,
                    delay,
                });
                editor.config_changed = true;
                *status_message = "文本配置已更新".to_string();
                log_messages.push("[INFO] 更新文本配置".to_string());
            }
        }
    }
}

/// 编辑按键序列操作
pub fn edit_sequence_action(
    editor: &mut VisualEditor,
    ui: &mut egui::Ui,
    idx: usize,
    status_message: &mut String,
    log_messages: &mut Vec<String>,
) {
    ui.label("按键序列编辑:");
    ui.separator();
    
    // 显示步骤数量
    if let ActionParams::Sequence(params) = &editor.config.hotkeys[idx].params {
        ui.label(format!("📊 步骤数量: {}", params.steps.len()));
    }
    
    // 步骤列表
    show_step_list(editor, ui, idx);
    
    ui.add_space(10.0);
    
    // 添加步骤按钮和删除按钮
    let mut added_idx: Option<usize> = None;
    ui.horizontal(|ui| {
        ui.menu_button("➕ 添加步骤", |ui| {
            if ui.button("⌨ 按键").clicked() {
                added_idx = add_key_step(editor, idx, status_message, log_messages);
                ui.close_menu();
            }
            if ui.button("⏱ 等待").clicked() {
                added_idx = add_wait_step(editor, idx, status_message, log_messages);
                ui.close_menu();
            }
            if ui.button("✍ 文本").clicked() {
                added_idx = add_text_step(editor, idx, status_message, log_messages);
                ui.close_menu();
            }
            ui.menu_button("🖱 鼠标点击", |ui| {
                if ui.button("左键").clicked() {
                    added_idx = add_mouse_click_step(editor, idx, MouseButtonType::Left, status_message, log_messages);
                    ui.close_menu();
                }
                if ui.button("右键").clicked() {
                    added_idx = add_mouse_click_step(editor, idx, MouseButtonType::Right, status_message, log_messages);
                    ui.close_menu();
                }
                if ui.button("中键").clicked() {
                    added_idx = add_mouse_click_step(editor, idx, MouseButtonType::Middle, status_message, log_messages);
                    ui.close_menu();
                }
            });
        });
        
        // 删除步骤
        if let Some(step_idx) = ui.data(|data| data.get_temp::<usize>(Id::new("selected_step"))) {
            let mut should_remove = false;
            if ui.button("❌ 删除步骤").clicked() {
                should_remove = true;
            }
            if should_remove {
                remove_step(editor, idx, step_idx, status_message, log_messages);
            }
        }
    });
    
    // 自动选中新添加的步骤
    if let Some(step_idx) = added_idx {
        ui.data_mut(|data| {
            data.insert_temp(Id::new("selected_step"), Some(step_idx));
        });
    }
    
    // 编辑选中的步骤
    if let Some(step_idx) = ui.data(|data| data.get_temp::<usize>(Id::new("selected_step"))) {
        ui.separator();
        ui.label("✏ 编辑步骤详情:");
        edit_step_detail(editor, ui, idx, step_idx, status_message, log_messages);
    }
}

/// 显示步骤列表
fn show_step_list(
    editor: &mut VisualEditor,
    ui: &mut egui::Ui,
    idx: usize,
) {
    let mut delete_idx: Option<usize> = None;
    
    egui::ScrollArea::vertical()
        .id_salt("step_list")
        .auto_shrink([false, false])
        .min_scrolled_height(100.0)
        .max_height(ui.available_height() - 150.0)
        .show(ui, |ui| {
            if let ActionParams::Sequence(params) = &mut editor.config.hotkeys[idx].params {
                for (i, step) in params.steps.iter_mut().enumerate() {
                    let is_selected = ui.data(|data| data.get_temp::<usize>(Id::new("selected_step")) == Some(i));
                    
                    ui.horizontal(|ui| {
                        match step {
                            Step::Key { value, .. } => {
                                // Key 步骤：⌨ 按键: 作为可选择标签，后面直接跟 ComboBox 选择键值
                                let selected = ui.selectable_label(is_selected, "⌨ 按键:");
                                if selected.clicked() {
                                    ui.data_mut(|data| {
                                        data.insert_temp(Id::new("selected_step"), if is_selected { None } else { Some(i) });
                                    });
                                }
                                let keys = keyboard_key_list();
                                let mut key_value = value.clone();
                                egui::ComboBox::from_id_salt(format!("step_key_cb_{}", i))
                                    .selected_text(key_value.clone())
                                    .width(60.0)
                                    .show_ui(ui, |ui| {
                                        for k in keys {
                                            ui.selectable_value(&mut key_value, k.to_string(), k);
                                        }
                                    });
                                if key_value != *value {
                                    *value = key_value;
                                    editor.config_changed = true;
                                }
                            }
                            _ => {
                                let label = match step {
                                    Step::Wait { value, random } => {
                                        if *random == Some(true) {
                                            format!("⏱ 随机等待: 0-{}ms", value)
                                        } else {
                                            format!("⏱ 等待: {}ms", value)
                                        }
                                    }
                                    Step::Text { value, .. } => format!("✍ 文本: \"{}\"", value),
                                    Step::MouseClick { button, .. } => format!("🖱 鼠标{}键点击", 
                                        match button {
                                            MouseButtonType::Left => "左",
                                            MouseButtonType::Right => "右",
                                            MouseButtonType::Middle => "中",
                                        }),
                                    Step::MouseAction { button, action, .. } => format!("🖱 鼠标{}键 {:?}", 
                                        match button {
                                            MouseButtonType::Left => "左",
                                            MouseButtonType::Right => "右",
                                            MouseButtonType::Middle => "中",
                                        }, action),
                                    Step::MouseMove { x, y, relative, .. } => {
                                        if *relative == Some(true) {
                                            format!("🖱 相对移动: ({}, {})", x, y)
                                        } else {
                                            format!("🖱 移动到: ({}, {})", x, y)
                                        }
                                    }
                                    Step::MouseWheel { delta, .. } => {
                                        if *delta > 0 {
                                            format!("🖱 滚轮向上: {}", delta)
                                        } else {
                                            format!("🖱 滚轮向下: {}", delta.abs())
                                        }
                                    }
                                    _ => String::new(),
                                };
                                
                                if ui.selectable_label(is_selected, label).clicked() {
                                    ui.data_mut(|data| {
                                        data.insert_temp(Id::new("selected_step"), if data.get_temp::<usize>(Id::new("selected_step")) == Some(i) {
                                            None
                                        } else {
                                            Some(i)
                                        });
                                    });
                                }
                            }
                        }
                        
                        // 每个步骤行上的删除按钮
                        if ui.small_button("🗑").clicked() {
                            delete_idx = Some(i);
                        }
                    });
                }
            }
        });
    
    // 处理步骤删除（在循环外执行可变操作）
    if let Some(step_idx) = delete_idx {
        remove_step(editor, idx, step_idx, &mut String::new(), &mut Vec::new());
        // 清除选中状态
        ui.data_mut(|data| {
            data.remove::<usize>(Id::new("selected_step"));
        });
    }
}

/// 添加按键步骤
fn add_key_step(
    editor: &mut VisualEditor,
    idx: usize,
    status_message: &mut String,
    log_messages: &mut Vec<String>,
) -> Option<usize> {
    if let Some(hotkey) = editor.config.hotkeys.get_mut(idx) {
        if let ActionParams::Sequence(params) = &mut hotkey.params {
            params.steps.push(Step::Key {
                value: "A".to_string(),
                delay: Some(DelayConfig::Fixed(50)),
                action: Some(KeyAction::Complete),
            });
            let new_idx = params.steps.len() - 1;
            *status_message = "已添加按键步骤".to_string();
            log_messages.push("[INFO] 添加按键步骤".to_string());
            return Some(new_idx);
        }
    }
    None
}

/// 添加等待步骤
fn add_wait_step(
    editor: &mut VisualEditor,
    idx: usize,
    status_message: &mut String,
    log_messages: &mut Vec<String>,
) -> Option<usize> {
    if let Some(hotkey) = editor.config.hotkeys.get_mut(idx) {
        if let ActionParams::Sequence(params) = &mut hotkey.params {
            params.steps.push(Step::Wait {
                value: 100,
                random: None,
            });
            let new_idx = params.steps.len() - 1;
            *status_message = "已添加等待步骤".to_string();
            log_messages.push("[INFO] 添加等待步骤".to_string());
            return Some(new_idx);
        }
    }
    None
}

/// 添加文本步骤
fn add_text_step(
    editor: &mut VisualEditor,
    idx: usize,
    status_message: &mut String,
    log_messages: &mut Vec<String>,
) -> Option<usize> {
    if let Some(hotkey) = editor.config.hotkeys.get_mut(idx) {
        if let ActionParams::Sequence(params) = &mut hotkey.params {
            params.steps.push(Step::Text {
                value: "hello".to_string(),
                delay: Some(DelayConfig::Fixed(10)),
            });
            let new_idx = params.steps.len() - 1;
            *status_message = "已添加文本步骤".to_string();
            log_messages.push("[INFO] 添加文本步骤".to_string());
            return Some(new_idx);
        }
    }
    None
}

/// 添加鼠标点击步骤
fn add_mouse_click_step(
    editor: &mut VisualEditor,
    idx: usize,
    button: MouseButtonType,
    status_message: &mut String,
    log_messages: &mut Vec<String>,
) -> Option<usize> {
    if let Some(hotkey) = editor.config.hotkeys.get_mut(idx) {
        if let ActionParams::Sequence(params) = &mut hotkey.params {
            let btn_label = match button {
                MouseButtonType::Left => "左键",
                MouseButtonType::Right => "右键",
                MouseButtonType::Middle => "中键",
            };
            params.steps.push(Step::MouseClick {
                button,
                delay: Some(DelayConfig::Fixed(50)),
            });
            let new_idx = params.steps.len() - 1;
            *status_message = format!("已添加鼠标{}点击步骤", btn_label);
            log_messages.push(format!("[INFO] 添加鼠标{}点击步骤", btn_label));
            return Some(new_idx);
        }
    }
    None
}

/// 删除步骤
fn remove_step(
    editor: &mut VisualEditor,
    macro_idx: usize,
    step_idx: usize,
    status_message: &mut String,
    log_messages: &mut Vec<String>,
) {
    if let Some(hotkey) = editor.config.hotkeys.get_mut(macro_idx) {
        if let ActionParams::Sequence(params) = &mut hotkey.params {
            if step_idx < params.steps.len() {
                params.steps.remove(step_idx);
                *status_message = "已删除步骤".to_string();
                log_messages.push("[INFO] 删除步骤".to_string());
            }
        }
    }
}

/// 编辑步骤详情
fn edit_step_detail(
    editor: &mut VisualEditor,
    ui: &mut egui::Ui,
    macro_idx: usize,
    step_idx: usize,
    status_message: &mut String,
    log_messages: &mut Vec<String>,
) {
    // 使用索引访问而不是直接借用 step，避免借用冲突
    let step_type = if let ActionParams::Sequence(params) = &editor.config.hotkeys[macro_idx].params {
        if step_idx < params.steps.len() {
            match &params.steps[step_idx] {
                Step::Key { .. } => 0,
                Step::Wait { .. } => 1,
                Step::Text { .. } => 2,
                Step::MouseClick { .. } => 3,
                Step::MouseAction { .. } => 4,
                Step::MouseMove { .. } => 5,
                Step::MouseWheel { .. } => 6,
            }
        } else {
            return;
        }
    } else {
        return;
    };
    
    match step_type {
        0 => edit_key_step(editor, ui, macro_idx, step_idx, status_message, log_messages),
        1 => edit_wait_step(editor, ui, macro_idx, step_idx, status_message, log_messages),
        2 => edit_text_step(editor, ui, macro_idx, step_idx, status_message, log_messages),
        3 => edit_mouse_click_step(editor, ui, macro_idx, step_idx, status_message, log_messages),
        4 => edit_mouse_action_step(editor, ui, macro_idx, step_idx, status_message, log_messages),
        5 => edit_mouse_move_step(editor, ui, macro_idx, step_idx, status_message, log_messages),
        6 => edit_mouse_wheel_step(editor, ui, macro_idx, step_idx, status_message, log_messages),
        _ => {}
    }
}

/// 编辑按键步骤
fn edit_key_step(
    editor: &mut VisualEditor,
    ui: &mut egui::Ui,
    macro_idx: usize,
    step_idx: usize,
    status_message: &mut String,
    log_messages: &mut Vec<String>,
) {
    if let ActionParams::Sequence(params) = &mut editor.config.hotkeys[macro_idx].params {
        if let Step::Key { delay, action, .. } = &mut params.steps[step_idx] {
            ui.label("⌨ 按键配置:");
            
            // 按键已在内联列表中直接可改，详情只显示动作和延迟
            
            let mut action_str = match action {
                Some(KeyAction::Press) => "press".to_string(),
                Some(KeyAction::Release) => "release".to_string(),
                _ => "complete".to_string(),
            };
            let old_action = action_str.clone();
            ui.horizontal(|ui| {
                ui.radio_value(&mut action_str, "press".to_string(), "按下");
                ui.radio_value(&mut action_str, "release".to_string(), "释放");
                ui.radio_value(&mut action_str, "complete".to_string(), "完成(按下+释放)");
            });
            if action_str != old_action {
                *action = Some(match action_str.as_str() {
                    "press" => KeyAction::Press,
                    "release" => KeyAction::Release,
                    _ => KeyAction::Complete,
                });
                editor.config_changed = true;
                *status_message = "按键动作已更新".to_string();
                log_messages.push("[INFO] 更新按键动作".to_string());
            }
            
            ui.label("延迟 (ms):");
            edit_delay_config(ui, delay);
        }
    }
}

/// 编辑等待步骤
fn edit_wait_step(
    editor: &mut VisualEditor,
    ui: &mut egui::Ui,
    macro_idx: usize,
    step_idx: usize,
    status_message: &mut String,
    log_messages: &mut Vec<String>,
) {
    if let ActionParams::Sequence(params) = &mut editor.config.hotkeys[macro_idx].params {
        if let Step::Wait { value, random } = &mut params.steps[step_idx] {
            ui.label("⏱ 等待配置:");
            
            ui.label("等待时间 (ms):");
            let mut wait_val = value.to_string();
            ui.text_edit_singleline(&mut wait_val);
            
            ui.label("随机等待:");
            let mut is_random = random.unwrap_or(false);
            ui.checkbox(&mut is_random, "启用随机等待");
            
            if is_random {
                ui.horizontal(|ui| {
                    ui.label("随机范围: 0 ~");
                    ui.label(wait_val.as_str());
                });
            }
            
            if ui.button("✓ 应用等待配置").clicked() {
                if let Ok(wait_ms) = wait_val.parse::<u64>() {
                    *value = wait_ms;
                    *random = if is_random { Some(true) } else { None };
                    editor.config_changed = true;
                    *status_message = "等待配置已更新".to_string();
                    log_messages.push("[INFO] 更新等待配置".to_string());
                }
            }
        }
    }
}

/// 编辑文本步骤
fn edit_text_step(
    editor: &mut VisualEditor,
    ui: &mut egui::Ui,
    macro_idx: usize,
    step_idx: usize,
    status_message: &mut String,
    log_messages: &mut Vec<String>,
) {
    if let ActionParams::Sequence(params) = &mut editor.config.hotkeys[macro_idx].params {
        if let Step::Text { value, delay: _ } = &mut params.steps[step_idx] {
            ui.label("✍ 文本配置:");
            
            ui.label("输入文本:");
            ui.add(egui::TextEdit::multiline(value)
                .desired_rows(2)
                .desired_width(f32::INFINITY));
            
            if ui.button("✓ 应用文本配置").clicked() {
                editor.config_changed = true;
                *status_message = "文本配置已更新".to_string();
                log_messages.push("[INFO] 更新文本步骤".to_string());
            }
        }
    }
}

/// 编辑鼠标点击步骤
fn edit_mouse_click_step(
    editor: &mut VisualEditor,
    ui: &mut egui::Ui,
    macro_idx: usize,
    step_idx: usize,
    status_message: &mut String,
    log_messages: &mut Vec<String>,
) {
    if let ActionParams::Sequence(params) = &mut editor.config.hotkeys[macro_idx].params {
        if let Step::MouseClick { button, delay } = &mut params.steps[step_idx] {
            ui.label("🖱 鼠标点击配置:");
            
            ui.label("按钮:");
            let mut button_str = match button {
                MouseButtonType::Left => "left",
                MouseButtonType::Right => "right",
                MouseButtonType::Middle => "middle",
            }.to_string();
            
            ui.horizontal(|ui| {
                ui.radio_value(&mut button_str, "left".to_string(), "左键");
                ui.radio_value(&mut button_str, "right".to_string(), "右键");
                ui.radio_value(&mut button_str, "middle".to_string(), "中键");
            });
            
            *button = match button_str.as_str() {
                "left" => MouseButtonType::Left,
                "right" => MouseButtonType::Right,
                "middle" => MouseButtonType::Middle,
                _ => MouseButtonType::Left,
            };
            
            ui.label("延迟 (ms):");
            edit_delay_config(ui, delay);
            
            if ui.button("✓ 应用鼠标点击配置").clicked() {
                editor.config_changed = true;
                *status_message = "鼠标点击配置已更新".to_string();
                log_messages.push("[INFO] 更新鼠标点击配置".to_string());
            }
        }
    }
}

/// 编辑鼠标动作步骤
fn edit_mouse_action_step(
    editor: &mut VisualEditor,
    ui: &mut egui::Ui,
    macro_idx: usize,
    step_idx: usize,
    status_message: &mut String,
    log_messages: &mut Vec<String>,
) {
    if let ActionParams::Sequence(params) = &mut editor.config.hotkeys[macro_idx].params {
        if let Step::MouseAction { button, action, delay } = &mut params.steps[step_idx] {
            ui.label("🖱 鼠标动作配置:");
            
            ui.label("按钮:");
            let mut button_str = match button {
                MouseButtonType::Left => "left",
                MouseButtonType::Right => "right",
                MouseButtonType::Middle => "middle",
            }.to_string();
            
            ui.horizontal(|ui| {
                ui.radio_value(&mut button_str, "left".to_string(), "左键");
                ui.radio_value(&mut button_str, "right".to_string(), "右键");
                ui.radio_value(&mut button_str, "middle".to_string(), "中键");
            });
            
            *button = match button_str.as_str() {
                "left" => MouseButtonType::Left,
                "right" => MouseButtonType::Right,
                "middle" => MouseButtonType::Middle,
                _ => MouseButtonType::Left,
            };
            
            ui.label("动作:");
            let mut action_str = match action {
                MouseAction::Click => "click",
                MouseAction::Down => "down",
                MouseAction::Up => "up",
            }.to_string();
            
            ui.horizontal(|ui| {
                ui.radio_value(&mut action_str, "click".to_string(), "点击");
                ui.radio_value(&mut action_str, "down".to_string(), "按下");
                ui.radio_value(&mut action_str, "up".to_string(), "释放");
            });
            
            *action = match action_str.as_str() {
                "click" => MouseAction::Click,
                "down" => MouseAction::Down,
                "up" => MouseAction::Up,
                _ => MouseAction::Click,
            };
            
            ui.label("延迟 (ms):");
            edit_delay_config(ui, delay);
            
            if ui.button("✓ 应用鼠标动作配置").clicked() {
                editor.config_changed = true;
                *status_message = "鼠标动作配置已更新".to_string();
                log_messages.push("[INFO] 更新鼠标动作配置".to_string());
            }
        }
    }
}

/// 编辑鼠标移动步骤
fn edit_mouse_move_step(
    editor: &mut VisualEditor,
    ui: &mut egui::Ui,
    macro_idx: usize,
    step_idx: usize,
    status_message: &mut String,
    log_messages: &mut Vec<String>,
) {
    if let ActionParams::Sequence(params) = &mut editor.config.hotkeys[macro_idx].params {
        if let Step::MouseMove { x, y, relative, delay } = &mut params.steps[step_idx] {
            ui.label("🖱 鼠标移动配置:");
            
            ui.label("移动类型:");
            let mut is_relative = relative.unwrap_or(false);
            ui.horizontal(|ui| {
                ui.radio_value(&mut is_relative, false, "绝对位置");
                ui.radio_value(&mut is_relative, true, "相对移动");
            });
            *relative = Some(is_relative);
            
            ui.label(if is_relative { "X偏移:" } else { "X坐标:" });
            let mut x_str = x.to_string();
            ui.text_edit_singleline(&mut x_str);
            if let Ok(new_x) = x_str.parse::<i32>() {
                *x = new_x;
            }
            
            ui.label(if is_relative { "Y偏移:" } else { "Y坐标:" });
            let mut y_str = y.to_string();
            ui.text_edit_singleline(&mut y_str);
            if let Ok(new_y) = y_str.parse::<i32>() {
                *y = new_y;
            }
            
            ui.label("延迟 (ms):");
            edit_delay_config(ui, delay);
            
            if ui.button("✓ 应用鼠标移动配置").clicked() {
                editor.config_changed = true;
                *status_message = "鼠标移动配置已更新".to_string();
                log_messages.push("[INFO] 更新鼠标移动配置".to_string());
            }
        }
    }
}

/// 编辑鼠标滚轮步骤
fn edit_mouse_wheel_step(
    editor: &mut VisualEditor,
    ui: &mut egui::Ui,
    macro_idx: usize,
    step_idx: usize,
    status_message: &mut String,
    log_messages: &mut Vec<String>,
) {
    if let ActionParams::Sequence(params) = &mut editor.config.hotkeys[macro_idx].params {
        if let Step::MouseWheel { delta, delay } = &mut params.steps[step_idx] {
            ui.label("🖱 鼠标滚轮配置:");
            
            ui.label("滚轮:");
            let mut delta_str = delta.to_string();
            ui.horizontal(|ui| {
                ui.label("数值 (正=向上, 负=向下):");
                ui.text_edit_singleline(&mut delta_str);
            });
            
            if let Ok(new_delta) = delta_str.parse::<i32>() {
                *delta = new_delta;
            }
            
            ui.label("延迟 (ms):");
            edit_delay_config(ui, delay);
            
            if ui.button("✓ 应用鼠标滚轮配置").clicked() {
                editor.config_changed = true;
                *status_message = "鼠标滚轮配置已更新".to_string();
                log_messages.push("[INFO] 更新鼠标滚轮配置".to_string());
            }
        }
    }
}

/// 编辑延迟配置（通用函数）
fn edit_delay_config(ui: &mut egui::Ui, delay: &mut Option<DelayConfig>) {
    ui.label("延迟 (ms):");
    let mut delay_type = match delay {
        Some(DelayConfig::Fixed(ms)) => ("固定".to_string(), ms.to_string(), "".to_string()),
        Some(DelayConfig::Range { min, max }) => ("范围".to_string(), min.to_string(), max.to_string()),
        None => ("固定".to_string(), "50".to_string(), "".to_string()),
    };
    
    ui.horizontal(|ui| {
        ui.radio_value(&mut delay_type.0, "固定".to_string(), "固定延迟");
        ui.radio_value(&mut delay_type.0, "范围".to_string(), "随机范围");
    });
    
    if delay_type.0 == "固定" {
        ui.text_edit_singleline(&mut delay_type.1);
    } else {
        ui.horizontal(|ui| {
            ui.label("最小:");
            ui.text_edit_singleline(&mut delay_type.1);
            ui.label("最大:");
            ui.text_edit_singleline(&mut delay_type.2);
        });
    }
    
    // 更新 delay 值
    *delay = if delay_type.0 == "固定" {
        delay_type.1.parse::<u64>()
            .ok()
            .map(DelayConfig::Fixed)
    } else {
        if let (Ok(min), Ok(max)) = (delay_type.1.parse(), delay_type.2.parse()) {
            Some(DelayConfig::Range { min, max })
        } else {
            None
        }
    };
}
