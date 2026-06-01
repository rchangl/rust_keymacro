//! GUI 应用模块
//!
//! 使用 egui + eframe 构建的主窗口界面

pub mod yaml_editor;
pub mod log_viewer;

use eframe::egui;
use crate::config::Config;
use crate::macros::{init_keyboard_macro_system, set_macro_enabled, cleanup_keyboard_hook};
use crate::visual_editor::VisualEditor;
use windows::Win32::UI::WindowsAndMessaging::HHOOK;

/// 主窗口应用
pub struct MacroApp {
    /// 宏配置
    config: Config,
    
    /// 配置文本（用于编辑）
    config_text: String,
    
    /// 可视化编辑器
    visual_editor: VisualEditor,
    
    /// 宏是否启用
    macro_enabled: bool,
    
    /// 键盘钩子句柄
    keyboard_hook: Option<HHOOK>,
    
    /// 日志消息
    log_messages: Vec<String>,
    
    /// 当前选中的标签页
    selected_tab: usize,
    
    /// 状态消息
    status_message: String,
    
    /// 是否显示配置错误
    show_config_error: bool,
    
    /// 配置错误信息
    config_error: String,
}

impl MacroApp {
    /// 创建新的应用实例
    pub fn new(cc: &eframe::CreationContext<'_>, config: Config) -> Self {
        // 设置支持中文的字体
        Self::setup_chinese_font(&cc.egui_ctx);
        
        // 设置浅色主题
        cc.egui_ctx.set_visuals(egui::Visuals::light());
        
        // 序列化配置为文本
        let config_text = serde_yaml::to_string(&config)
            .unwrap_or_else(|_| "# 配置序列化失败".to_string());
        
        // 创建可视化编辑器
        let visual_editor = VisualEditor::new(config.clone());
        
        // 初始化键盘宏系统
        let keyboard_hook = init_keyboard_macro_system(config.clone());
        
        Self {
            config,
            config_text,
            visual_editor,
            macro_enabled: false,
            keyboard_hook,
            log_messages: vec![
                "[INFO] 应用程序启动".to_string(),
                "[INFO] 键盘宏系统已初始化（默认禁用）".to_string(),
            ],
            selected_tab: 0,
            status_message: "就绪".to_string(),
            show_config_error: false,
            config_error: String::new(),
        }
    }
    
    /// 设置中文字体支持
    fn setup_chinese_font(ctx: &egui::Context) {
        use std::path::Path;
        
        let mut fonts = egui::FontDefinitions::default();
        
        // Windows 系统字体路径列表（按优先级）
        let font_paths = [
            "C:/Windows/Fonts/msyh.ttc",      // 微软雅黑
            "C:/Windows/Fonts/msyhbd.ttc",    // 微软雅黑粗体
            "C:/Windows/Fonts/simsun.ttc",    // 宋体
            "C:/Windows/Fonts/simhei.ttf",    // 黑体
            "C:/Windows/Fonts/mingliu.ttc",   // 细明体
        ];
        
        // 尝试加载第一个可用的中文字体
        for font_path in font_paths.iter() {
            if Path::new(font_path).exists() {
                if let Ok(font_bytes) = std::fs::read(font_path) {
                    fonts.font_data.insert(
                        "chinese_font".to_owned(),
                        egui::FontData::from_owned(font_bytes),
                    );
                    break;
                }
            }
        }
        
        // 设置字体优先级，中文字体排在前面
        fonts
            .families
            .entry(egui::FontFamily::Proportional)
            .or_default()
            .insert(0, "chinese_font".to_owned());
        
        fonts
            .families
            .entry(egui::FontFamily::Monospace)
            .or_default()
            .insert(0, "chinese_font".to_owned());
        
        ctx.set_fonts(fonts);
    }
    
    /// 应用配置更改
    fn apply_config(&mut self) {
        match serde_yaml::from_str::<Config>(&self.config_text) {
            Ok(new_config) => {
                // 清理旧的钩子
                if let Some(hook) = self.keyboard_hook.take() {
                    cleanup_keyboard_hook(hook);
                }
                
                // 更新配置并重新初始化
                self.config = new_config.clone();
                self.keyboard_hook = init_keyboard_macro_system(new_config);
                
                self.status_message = "配置已应用".to_string();
                self.log_messages.push("[INFO] 配置已更新并应用".to_string());
                self.show_config_error = false;
            }
            Err(e) => {
                self.show_config_error = true;
                self.config_error = e.to_string();
                self.status_message = "配置错误，请检查 YAML 格式".to_string();
                self.log_messages.push(format!("[ERROR] 配置解析失败: {}", e));
            }
        }
    }
    
    /// 保存配置到文件
    fn save_config(&mut self) {
        // 优先使用可视化编辑器的配置
        self.config = self.visual_editor.config.clone();
        
        // 同步到YAML文本
        self.config_text = serde_yaml::to_string(&self.config)
            .unwrap_or_else(|_| "# 配置序列化失败".to_string());
        
        let config_path = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.join("config.yaml")))
            .unwrap_or_else(|| "config.yaml".into());
        
        match std::fs::write(&config_path, &self.config_text) {
            Ok(_) => {
                // 清理旧的钩子并重新初始化
                if let Some(hook) = self.keyboard_hook.take() {
                    cleanup_keyboard_hook(hook);
                }
                self.keyboard_hook = init_keyboard_macro_system(self.config.clone());
                
                self.status_message = format!("✓ 配置已保存到: {}", config_path.display());
                self.log_messages.push(format!("[INFO] 配置已保存到: {}", config_path.display()));
                self.show_config_error = false;
            }
            Err(e) => {
                self.status_message = format!("✗ 保存失败: {}", e);
                self.log_messages.push(format!("[ERROR] 保存配置失败: {}", e));
            }
        }
    }
    
    /// 切换宏启用状态
    fn toggle_macro(&mut self) {
        self.macro_enabled = !self.macro_enabled;
        set_macro_enabled(self.macro_enabled);
        
        let state = if self.macro_enabled { "已启用" } else { "已禁用" };
        self.status_message = format!("宏 {}", state);
        self.log_messages.push(format!("[INFO] 宏 {}", state));
    }
}

impl eframe::App for MacroApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 检查配置是否已更改，如果是则重新初始化系统
        if self.visual_editor.config_changed {
            // 同步配置
            self.config = self.visual_editor.config.clone();
            
            // 清理旧的钩子并重新初始化
            if let Some(hook) = self.keyboard_hook.take() {
                cleanup_keyboard_hook(hook);
            }
            self.keyboard_hook = init_keyboard_macro_system(self.config.clone());
            
            // 重置标志位
            self.visual_editor.config_changed = false;
            
            self.status_message = "配置已应用并重新初始化".to_string();
            self.log_messages.push("[INFO] 配置已应用，系统已重新初始化".to_string());
        }
        
        // 顶部面板
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            show_menu_bar(self, ui);
        });
        
        // 底部状态栏
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(&self.status_message);
            });
        });
        
        // 中央内容区域
        egui::CentralPanel::default().show(ctx, |ui| {
            // 标签页选择
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.selected_tab, 0, "🎯 可视化编辑器");
                ui.selectable_value(&mut self.selected_tab, 1, "📝 YAML编辑器");
                ui.selectable_value(&mut self.selected_tab, 2, "📊 运行日志");
            });
            
            ui.separator();
            
            // 根据选中的标签页显示不同内容
            match self.selected_tab {
                0 => self.visual_editor.show(ui, &mut self.status_message, &mut self.log_messages),
                1 => yaml_editor::show(self, ui),
                2 => log_viewer::show(self, ui),
                _ => {}
            }
        });
    }
}

/// 显示菜单栏
fn show_menu_bar(app: &mut MacroApp, ui: &mut egui::Ui) {
    egui::menu::bar(ui, |ui| {
        ui.menu_button("文件", |ui| {
            if ui.button("保存配置").clicked() {
                app.save_config();
                ui.close_menu();
            }
            if ui.button("重新加载配置").clicked() {
                reload_config(app);
                ui.close_menu();
            }
            ui.separator();
            if ui.button("退出").clicked() {
                if let Some(hook) = app.keyboard_hook.take() {
                    cleanup_keyboard_hook(hook);
                }
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
        

        
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let status_color = if app.macro_enabled {
                egui::Color32::GREEN
            } else {
                egui::Color32::RED
            };
            
            ui.colored_label(status_color, if app.macro_enabled { "● 已启用" } else { "● 已禁用" });
            
            if ui.button(if app.macro_enabled { "禁用宏" } else { "启用宏" }).clicked() {
                app.toggle_macro();
            }
        });
    });
}

/// 重新加载配置
fn reload_config(app: &mut MacroApp) {
    let config_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("config.yaml")))
        .unwrap_or_else(|| "config.yaml".into());
    
    match Config::from_file(&config_path) {
        Ok(config) => {
            app.config = config.clone();
            app.config_text = serde_yaml::to_string(&config)
                .unwrap_or_default();
            app.visual_editor = VisualEditor::new(config.clone());
            
            // 清理旧的钩子并重新初始化
            if let Some(hook) = app.keyboard_hook.take() {
                cleanup_keyboard_hook(hook);
            }
            app.keyboard_hook = init_keyboard_macro_system(config);
            
            app.status_message = "✓ 配置已重新加载".to_string();
            app.log_messages.push("[INFO] 配置已重新加载".to_string());
        }
        Err(e) => {
            app.status_message = format!("✗ 加载失败: {}", e);
            app.log_messages.push(format!("[ERROR] 加载配置失败: {}", e));
        }
    }
}
