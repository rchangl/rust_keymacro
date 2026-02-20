# 配置文件驱动键盘宏系统

本项目支持通过 YAML 配置文件来定义键盘宏操作，同时支持 **键盘热键** 和 **手柄按键** 触发，无需修改代码即可添加、修改或删除热键功能。

## 全局开关

程序提供了一个全局开关热键 **Ctrl + `**（Ctrl + 反引号），用于启用或禁用所有键盘宏功能。

- 按下 **Ctrl + `** 可以快速开启或关闭键盘宏服务
- 开关状态切换时，会有弹出文字和图标变化来指示当前状态
- 关闭状态下，按配置的快捷键将不会触发任何宏操作

## 功能特性

- **键盘热键触发** - 支持各种键盘按键作为触发器
- **手柄按键触发** - 支持 Xbox 协议手柄（有线/无线）
- **配置文件驱动** - 通过 YAML 文件定义宏，无需修改代码
- **多种操作类型** - 支持输入文本、按键序列、等待等
- **随机延迟** - 支持固定或随机延迟，模拟人工操作
- **全局热键** - 全局开关控制所有宏功能
- **系统托盘** - 最小化到系统托盘，显示当前状态

## 配置文件结构

配置文件 `config.yaml` 采用 YAML 格式，包含一个 `hotkeys` 数组，每个元素定义一个热键配置。

### 基本结构

```yaml
hotkeys:
  - type: "keyboard"  # 触发源类型：keyboard 或 gamepad
    key: "热键名称"   # 键盘热键名称（type=keyboard 时使用）
    button: "A"       # 手柄按钮名称（type=gamepad 时使用）
    action: "操作类型"
    params:
      # 操作参数
```

### 触发源类型

#### 1. 键盘触发 (`type: keyboard`)

使用键盘按键作为触发器。

**必需字段：**
- `key`: 键盘按键名称

**支持的按键：**
- 字母：`A` - `Z`
- 数字：`0` - `9`
- 功能键：`F1` - `F24`
- 特殊键：`Space`, `Enter`, `Tab`, `Backspace`, `Escape`
- 修饰键：`Shift`, `Ctrl`, `Alt`

#### 2. 手柄触发 (`type: gamepad`)

使用 Xbox 协议手柄按键作为触发器。

**必需字段：**
- `button`: 手柄按钮名称

**支持的按钮：**
| 按钮名 | 说明 |
|-------|------|
| `A` | A 键（底部） |
| `B` | B 键（右侧） |
| `X` | X 键（左侧） |
| `Y` | Y 键（顶部） |
| `LB` | 左肩键 |
| `RB` | 右肩键 |
| `LT` | 左扳机（暂未支持） |
| `RT` | 右扳机（暂未支持） |
| `Start` | 菜单键 |
| `Back` | 返回/视图键 |
| `Guide` | Xbox 按钮 |
| `LS` | 左摇杆按下 |
| `RS` | 右摇杆按下 |
| `DUp` | 十字键上 |
| `DDown` | 十字键下 |
| `DLeft` | 十字键左 |
| `DRight` | 十字键右 |

**注意：** 支持国产 Xbox 兼容手柄和官方 Xbox 手柄。

## 支持的操作类型

### 1. type_text - 输入文本

快速输入指定文本，支持设置输入延迟。

**参数：**
- `text` (必需): 要输入的文本字符串
- `delay` (可选): 每个字符输入后的等待毫秒数，默认为 10ms
  - 支持固定值: `delay: 10`
  - 支持随机范围: `delay: { min: 5, max: 15 }` (在5-15毫秒之间随机)

**示例：**
```yaml
- type: "keyboard"
  key: "F2"
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
     - 固定值: `delay: 50`
     - 随机范围: `delay: { min: 10, max: 30 }`
   - `action` (可选): 按键动作类型
     - `press`: 只按下按键（不释放）
     - `release`: 只释放按键
     - `complete`: 按下并释放按键（默认）

2. **wait** - 等待
   - `value`: 等待的毫秒数
   - `random` (可选): 设置为 `true` 时在 `0 ~ value` 范围内随机等待

3. **text** - 输入文本
   - `value`: 要输入的文本字符串
   - `delay` (可选): 每个字符输入后的等待毫秒数
     - 固定值: `delay: 50`
     - 随机范围: `delay: { min: 5, max: 15 }`

**示例：**
```yaml
- type: "keyboard"
  key: "'"
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

## 配置示例

### 示例 1: 键盘热键触发

```yaml
hotkeys:
  # 按 F2 输入 "hello"
  - type: "keyboard"
    key: "F2"
    action: "type_text"
    params:
      text: "hello"
      delay: 5

  # 按 F3 输入 "world"
  - type: "keyboard"
    key: "F3"
    action: "type_text"
    params:
      text: "world"
      delay: 10
```

### 示例 2: 手柄按键触发

```yaml
hotkeys:
  # 手柄 A 键触发空格键
  - type: "gamepad"
    button: "A"
    action: "sequence"
    params:
      steps:
        - type: "key"
          value: "Space"
          action: "press"
          delay: 50
        - type: "key"
          value: "Space"
          action: "release"
          delay: 50

  # 手柄 B 键输入文本
  - type: "gamepad"
    button: "B"
    action: "type_text"
    params:
      text: "Hello from gamepad!"
      delay: 10

  # 手柄 X 键执行复杂序列
  - type: "gamepad"
    button: "X"
    action: "sequence"
    params:
      steps:
        - type: "key"
          value: "E"
          delay: 20
        - type: "key"
          value: "R"
          delay: 20
        - type: "key"
          value: "T"
```

### 示例 3: 使用随机延迟

通过随机延迟让宏执行更具不确定性，模拟人工操作：

```yaml
hotkeys:
  - type: "keyboard"
    key: "F5"
    action: "sequence"
    params:
      steps:
        # 按键延迟在 10-30ms 之间随机
        - type: "key"
          value: "A"
          delay: { min: 10, max: 30 }
        
        # 等待时间在 0-500ms 之间随机
        - type: "wait"
          value: 500
          random: true
        
        # 输入文本，每个字符延迟在 5-15ms 之间随机
        - type: "text"
          value: "hello"
          delay: { min: 5, max: 15 }
```

### 示例 4: 分离按键按下和释放（高级）

通过 `action` 参数控制按键的按下和释放，实现组合键效果：

```yaml
hotkeys:
  - type: "keyboard"
    key: "F4"
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

## 编译和运行

### Debug 模式（开发调试）

```bash
cargo run
```

- 会创建 `app.log` 日志文件记录调试信息
- 包含详细的日志输出，便于排查问题

### Release 模式（正式发布）

```bash
cargo build --release
```

- 不会创建任何日志文件
- 不会输出任何日志信息
- 性能更优，适合日常使用

编译完成后，可执行文件位于：
- Debug: `target/debug/rust_keymacro.exe`
- Release: `target/release/rust_keymacro.exe`

## 配置文件位置

配置文件必须命名为 `config.yaml`，并与程序可执行文件放在同一目录下。

## 热键冲突处理

- 如果配置文件中定义了相同的热键，只有第一个会被使用
- 键盘热键和手柄热键相互独立，不会冲突

## 运行时配置重载

目前配置在程序启动时加载。要应用新的配置，需要重启程序。

## 故障排查

### 手柄无法识别

1. 确保手柄已通过 USB 连接或无线接收器已插入
2. 在 Windows 中测试手柄：按 `Win + R`，输入 `joy.cpl` 回车
3. 确保手柄是 Xbox 兼容协议
4. 查看 Debug 模式的日志文件了解详细信息

### 配置加载失败

1. 检查 YAML 语法是否正确（可使用在线 YAML 验证工具）
2. 确保所有必需字段都存在
3. 在 Debug 模式下查看 `app.log` 日志文件
4. 检查按键/按钮名称是否支持

### 宏不执行

1. 确认全局开关已开启（按 `Ctrl + ` 查看状态）
2. 检查目标窗口是否有焦点
3. 某些游戏可能需要以管理员身份运行本程序
4. 杀毒软件可能会拦截键盘模拟，尝试添加白名单

## 项目结构

```
rust_keymacro/
├── Cargo.toml          # 项目配置
├── config.yaml         # 配置文件示例
├── src/
│   ├── main.rs         # 程序入口
│   ├── lib.rs          # 库入口
│   ├── app.rs          # 托盘应用
│   ├── bootstrap.rs    # 启动逻辑
│   ├── config.rs       # 配置解析
│   ├── gamepad/        # 手柄支持模块
│   │   └── mod.rs
│   ├── macros/         # 宏执行模块
│   │   ├── mod.rs
│   │   ├── executor.rs
│   │   └── handler.rs
│   ├── overlay.rs      # 屏幕提示
│   └── winapi/         # Windows API 封装
│       └── keyboard.rs
└── README.md
```

## 许可证

MIT License
