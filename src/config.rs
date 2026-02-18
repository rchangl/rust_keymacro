//! 配置文件加载和解析模块
//!
//! 支持从 YAML 文件加载键盘宏配置

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// 配置文件根结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub hotkeys: Vec<HotkeyConfig>,
}

/// 单个热键配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    /// 触发键，如 "F2", "Ctrl+Shift+A"
    pub key: String,
    /// 操作类型："type_text" 或 "sequence"
    pub action: String,
    /// 操作参数
    pub params: ActionParams,
}

/// 操作参数
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActionParams {
    TypeText(TypeTextParams),
    Sequence(SequenceParams),
}

/// 输入文本参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeTextParams {
    pub text: String,
    #[serde(default)]
    pub speed: Option<String>,
}

/// 序列参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceParams {
    pub steps: Vec<Step>,
}

/// 按键动作类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum KeyAction {
    Press,    // 只按下
    Release,  // 只释放
    Complete, // 按下+释放（默认）
}

impl Default for KeyAction {
    fn default() -> Self {
        KeyAction::Complete
    }
}

/// 序列中的单个步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Step {
    Key { 
        value: String, 
        #[serde(default)] 
        delay: Option<u64>,
        #[serde(default)]
        action: Option<KeyAction>,
    },
    Wait { value: u64 },
    Text { value: String, #[serde(default)] delay: Option<u64> },
}

impl Config {
    /// 从文件加载配置
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&content)?;
        Ok(config)
    }

    /// 从字符串加载配置（用于测试）
    #[allow(dead_code)]
    pub fn from_str(yaml_str: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config: Config = serde_yaml::from_str(yaml_str)?;
        Ok(config)
    }

    /// 查找指定键的配置
    pub fn find_hotkey(&self, key: &str) -> Option<&HotkeyConfig> {
        self.hotkeys.iter().find(|h| h.key.eq_ignore_ascii_case(key))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_type_text_config() {
        let yaml = r#"
hotkeys:
  - key: "F2"
    action: "type_text"
    params:
      text: "hello"
      speed: "fastest"
"#;
        let config = Config::from_str(yaml).unwrap();
        assert_eq!(config.hotkeys.len(), 1);
        
        let hotkey = &config.hotkeys[0];
        assert_eq!(hotkey.key, "F2");
        assert_eq!(hotkey.action, "type_text");
        
        if let ActionParams::TypeText(params) = &hotkey.params {
            assert_eq!(params.text, "hello");
            assert_eq!(params.speed, Some("fastest".to_string()));
        } else {
            panic!("Expected TypeText params");
        }
    }

    #[test]
    fn test_parse_sequence_config() {
        let yaml = r#"
hotkeys:
  - key: "Ctrl+Shift+A"
    action: "sequence"
    params:
      steps:
        - { type: "key", value: "a", delay: 50 }
        - { type: "wait", value: 100 }
        - { type: "text", value: "done" }
"#;
        let config = Config::from_str(yaml).unwrap();
        assert_eq!(config.hotkeys.len(), 1);
        
        let hotkey = &config.hotkeys[0];
        assert_eq!(hotkey.key, "Ctrl+Shift+A");
        assert_eq!(hotkey.action, "sequence");
        
        if let ActionParams::Sequence(params) = &hotkey.params {
            assert_eq!(params.steps.len(), 3);
            match &params.steps[0] {
                Step::Key { value, delay, action } => {
                    assert_eq!(value, "a");
                    assert_eq!(*delay, Some(50));
                    assert_eq!(*action, None); // 默认值为 None，会使用 KeyAction::Complete
                }
                _ => panic!("Expected Key step"),
            }
        } else {
            panic!("Expected Sequence params");
        }
    }

    #[test]
    fn test_parse_key_action_config() {
        let yaml = r#"
hotkeys:
  - key: "F1"
    action: "sequence"
    params:
      steps:
        - { type: "key", value: "Shift", action: "press" }
        - { type: "key", value: "a", action: "press" }
        - { type: "wait", value: 100 }
        - { type: "key", value: "a", action: "release" }
        - { type: "key", value: "Shift", action: "release" }
"#;
        let config = Config::from_str(yaml).unwrap();
        assert_eq!(config.hotkeys.len(), 1);
        
        if let ActionParams::Sequence(params) = &config.hotkeys[0].params {
            assert_eq!(params.steps.len(), 5);
            
            // 测试 press 动作
            match &params.steps[0] {
                Step::Key { value, action, .. } => {
                    assert_eq!(value, "Shift");
                    assert!(matches!(action, Some(KeyAction::Press)));
                }
                _ => panic!("Expected Key step"),
            }
            
            // 测试 release 动作
            match &params.steps[3] {
                Step::Key { value, action, .. } => {
                    assert_eq!(value, "a");
                    assert!(matches!(action, Some(KeyAction::Release)));
                }
                _ => panic!("Expected Key step"),
            }
        } else {
            panic!("Expected Sequence params");
        }
    }
}
