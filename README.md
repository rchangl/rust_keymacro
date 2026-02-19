# 配置文件驱动键盘宏系统

本项目现已支持通过 YAML 配置文件来定义键盘宏操作，无需修改代码即可添加、修改或删除热键功能。

## 全局开关

程序提供了一个全局开关热键 **Ctrl + `**（Ctrl + 反引号），用于启用或禁用所有键盘宏功能。

- 按下 **Ctrl + `** 可以快速开启或关闭键盘宏服务
- 开关状态切换时，会有弹出文字和图标变化来指示当前状态
- 关闭状态下，按配置的快捷键将不会触发任何宏操作

## 配置文件结构

配置文件 `config.yaml` 采用 YAML 格式，包含一个 `hotkeys` 数组，每个元素定义一个热键配置。

### 基本结构

```yaml
hotkeys:
  - key: "热键名称"
    action: "操作类型"
    params:
      # 操作参数
```

## 支持的操作类型

### 1. type_text - 输入文本

快速输入指定文本，支持设置输入延迟。

**参数：**
- `text` (必需): 要输入的文本字符串
- `delay` (可选): 每个字符输入后的等待毫秒数，默认为 10ms

**示例：**
```yaml
- key: "F2"
  action: "type_text"
  params:
    text: "hello world"
    delay: 10
```

### 2. sequence - 执行按键序列

按顺序执行一系列按键、等待和文本输入操作。

**参数：**
- `steps` (必需): 步骤数组，每个步骤可以是以下类型：

#### 步骤类型

1. **key** - 按键
   - `value`: 按键名称 (A-Z, 0-9, Space, Enter等)
   - `delay` (可选): 按键后等待的毫秒数
   - `action` (可选): 按键动作类型
     - `press`: 只按下按键（不释放）
     - `release`: 只释放按键
     - `complete`: 按下并释放按键（默认）

2. **wait** - 等待
   - `value`: 等待的毫秒数

3. **text** - 输入文本
   - `value`: 要输入的文本字符串
   - `delay` (可选): 每个字符输入后的等待毫秒数

**示例：**
```yaml
- key: "'"
  action: "sequence"
  params:
    steps:
      - type: "key"
        value: "E"
        delay: 17
      - type: "key"
        value: "R"
        delay: 17
      - type: "key"
        value: "T"
      - type: "wait"
        value: 100
      - type: "text"
        value: "done"
        delay: 50
```

## 支持的按键名称

### 字母和数字
- 字母：`A` - `Z`（不区分大小写）
- 数字：`0` - `9`

### 特殊按键
- `Space` - 空格键
- `Enter` - 回车键
- `Tab` - Tab键
- `Backspace` - 退格键
- `Escape` - Esc键
- `Shift` - Shift键
- `Ctrl` - Ctrl键
- `Alt` - Alt键
- `F1` - `F24` - 功能键

## 配置示例

### 示例 1: 快速输入常用文本
```yaml
hotkeys:
  - key: "F2"
    action: "type_text"
    params:
      text: "hello"
      delay: 5

  - key: "F3"
    action: "type_text"
    params:
      text: "world"
      delay: 10
```

### 示例 2: 复杂按键序列
```yaml
hotkeys:
  - key: "'"
    action: "sequence"
    params:
      steps:
        - type: "key"
          value: "E"
          delay: 17
        - type: "key"
          value: "R"
          delay: 17
        - type: "key"
          value: "E"
        - type: "key"
          value: "T"
          delay: 17
        - type: "key"
          value: "R"
        - type: "key"
          value: "Space"
```

### 示例 3: 带等待的序列
```yaml
hotkeys:
  - key: "A"
    action: "sequence"
    params:
      steps:
        - type: "key"
          value: "a"
          delay: 50
        - type: "key"
          value: "b"
          delay: 50
        - type: "wait"
          value: 100
        - type: "text"
          value: "done"
```

### 示例 4: 分离按键按下和释放（高级）

通过 `action` 参数控制按键的按下和释放，实现组合键效果：

```yaml
hotkeys:
  - key: "F4"
    action: "sequence"
    params:
      steps:
        # 按下 Shift（保持按住状态）
        - type: "key"
          value: "Shift"
          action: "press"
        
        # 按下 A（由于Shift被按住，实际输入大写A）
        - type: "key"
          value: "A"
          action: "press"
        
        # 等待100毫秒
        - type: "wait"
          value: 100
        
        # 释放 A
        - type: "key"
          value: "A"
          action: "release"
        
        # 释放 Shift
        - type: "key"
          value: "Shift"
          action: "release"
```

在这个示例中：
1. 首先按下 `Shift` 键并保持
2. 然后按下 `A` 键，由于 Shift 处于按住状态，会输入大写字母 A
3. 等待 100 毫秒
4. 释放 `A` 键
5. 释放 `Shift` 键

**注意：** 无论是 `press`、`release` 还是默认的 `complete` 动作，都可以使用 `delay` 参数来控制在按键动作后等待的时间（毫秒）。例如：

```yaml
- type: "key"
  value: "A"
  action: "press"
  delay: 50  # 按下后等待50毫秒
```

这种控制方式可以实现复杂的组合键操作和精确的按键时序控制。

## 热键冲突处理

如果配置文件中定义了相同的热键，只有第一个会被使用。

## 运行时配置重载

目前配置在程序启动时加载。要应用新的配置，需要重启程序。

## 调试

如果配置加载失败：
1. 检查 YAML 语法是否正确（可使用在线 YAML 验证工具）
2. 确保所有必需字段都存在
3. 查看控制台输出（如果使用 `--console` 参数运行）
4. 检查按键名称是否支持

## 配置文件位置

配置文件必须命名为 `config.yaml`，并与程序可执行文件放在同一目录下。

## 示例配置文件

项目中包含一个 `config.yaml` 示例文件，展示了所有支持的功能。你可以基于该文件进行修改。
