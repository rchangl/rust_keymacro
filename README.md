# Rust KeyMacro - 配置文件驱动键盘宏系统 (GUI 版)

本项目支持通过 YAML 配置文件来定义键盘宏操作，同时支持 **键盘热键** 和 **手柄按键** 触发，无需修改代码即可添加、修改或删除热键功能。

**v2.0.0 新增：** 全新 GUI 界面，提供可视化配置编辑、状态监控和日志查看功能！

## 全局开关

程序提供了一个全局开关热键 **Ctrl + `**（Ctrl + 反引号），用于启用或禁用所有键盘宏功能。

- 按下 **Ctrl + `** 可以快速开启或关闭键盘宏服务
- 开关状态切换时，GUI 界面会实时显示当前状态
- 关闭状态下，按配置的快捷键将不会触发任何宏操作

## 功能特性

- **🎯 可视化编辑器** - 无需手写YAML，通过图形界面直观编辑宏配置
- **🖥️ GUI 界面** - 全新的图形用户界面，可视化配置和管理宏
- **键盘热键触发** - 支持各种键盘按键作为触发器
- **手柄按键触发** - 支持 Xbox 协议手柄（有线/无线）
- **配置文件驱动** - 通过 YAML 文件定义宏，无需修改代码
- **多种操作类型** - 支持输入文本、按键序列、等待、鼠标操作等
- **随机延迟** - 支持固定或随机延迟，模拟人工操作
- **实时日志** - GUI 内置日志查看器，实时监控运行状态
- **双编辑模式** - 可视化编辑器和 YAML 编辑器实时同步，满足不同需求

## 项目结构

```
rust_keymacro/
├── Cargo.toml              # 项目配置
├── config.yaml              # 配置文件示例
├── src/
│   ├── main.rs             # 程序入口
│   ├── lib.rs              # 库入口
│   ├── bootstrap.rs        # 启动逻辑（配置加载、错误对话框）
│   ├── config.rs           # 配置解析（数据结构定义、YAML 反序列化）
│   ├── logger.rs           # 日志系统初始化
│   ├── overlay.rs          # 屏幕置顶提示窗口
│   ├── gui/                # GUI 模块
│   │   ├── mod.rs          # GUI 主入口（MacroApp 结构体、eframe::App 实现）
│   │   ├── yaml_editor.rs  # YAML 编辑器界面
│   │   ├── log_viewer.rs  # 日志查看器界面
│   │   └── help.rs        # 帮助页面
│   ├── visual_editor/      # 可视化编辑器模块
│   │   ├── mod.rs          # 编辑器主入口（VisualEditor 结构体）
│   │   ├── macro_list.rs   # 宏列表管理（左侧面板）
│   │   ├── macro_editor.rs # 宏详情编辑（右侧面板）
│   │   ├── step_editor.rs  # 步骤编辑器
│   │   └── key_selector.rs # 按键选择器
│   ├── macros/             # 宏执行模块
│   │   ├── mod.rs          # 模块入口（初始化、全局状态管理）
│   │   ├── executor.rs     # 宏执行器（输入文本、执行序列）
│   │   └── handler.rs      # 事件处理器（键盘钩子、手柄事件）
│   ├── gamepad/            # 手柄支持模块
│   │   └── mod.rs          # 手柄监听线程、按钮映射
│   └── winapi/             # Windows API 封装
│       ├── mod.rs          # 模块入口
│       ├── keyboard.rs     # 键盘钩子、按键模拟
│       ├── mouse.rs        # 鼠标控制（移动、点击、滚轮）
│       └── window.rs       # 窗口管理（创建、绘制、消息处理）
└── README.md
```

## GUI 界面使用

### 主窗口功能

启动程序后，将显示主窗口，包含以下功能区域：

#### 顶部菜单栏
- **文件菜单**
  - 保存配置：将当前配置保存到 config.yaml
  - 重新加载配置：从文件重新加载配置
  - 退出：关闭程序
  

#### 状态指示器（右上角）
- 绿色圆点 + "已启用"：宏系统正在运行
- 红色圆点 + "已禁用"：宏系统已停止
- 点击按钮可快速切换启用/禁用状态

#### 标签页
程序提供三个主要标签页：

1. **🎯 可视化编辑器** - 通过表单界面直观编辑宏配置 ⭐ 推荐
2. **📝 YAML编辑器** - 直接编辑 YAML 配置文件
3. **📊 运行日志** - 查看程序运行日志

### 🎯 可视化编辑器（推荐）

**全新的可视化编辑方式**，无需手写YAML，通过表单界面直观编辑：

#### 宏列表管理（左侧面板）
- 📋 显示所有已配置的宏列表
- ➕ 一键添加新宏
- ❌ 删除选中的宏
- 图标区分触发源（⌨ 键盘 / 🎮 手柄）

#### 宏详情编辑（右侧面板）

**触发源配置：**
- 选择触发类型：键盘或手柄
- 使用下拉菜单选择触发键
- 常用按键快速选择

**操作类型选择：**
- 📝 按键序列
- ✍ 输入文本

**按键序列编辑器：**
- 📊 显示步骤数量统计
- 可视化步骤列表，带图标标识：
  - ⌨ 按键步骤
  - ⏱ 等待步骤
  - ✍ 文本步骤
  - 🖱 鼠标操作步骤
- **添加步骤菜单：**
  - 按键（可选择具体按键和动作类型：按下/释放/完成）
  - 等待（支持固定时间和随机范围）
  - 文本
  - 鼠标点击
- 选中步骤后可编辑详细信息
- 一键删除步骤

**输入文本编辑器：**
- 多行文本输入框
- 延迟配置（固定值或随机范围）
- 实时应用配置

#### 使用流程
1. 点击"➕ 添加宏"创建新宏
2. 选择触发源和触发键
3. 选择操作类型
4. 根据操作类型填写相应参数
5. 点击"文件" → "保存配置"保存

### 📝 YAML编辑器

传统的YAML文本编辑方式，适合熟悉YAML格式的高级用户：

- 支持多行文本编辑，等宽字体显示
- 点击 **应用配置**：使配置立即生效（不保存到文件）
- 点击 **保存配置**：将配置保存到 config.yaml 文件
- 点击 **格式化配置**：自动格式化 YAML 格式
- 配置错误时会显示红色错误提示

### 运行日志

- 实时显示程序运行状态
- 错误信息用红色高亮显示
- 支持清空日志功能
- 自动滚动到最新日志

## 配置文件结构

配置文件 `config.yaml` 采用 YAML 格式，包含一个 `hotkeys` 数组，每个元素定义一个热键配置。

### 基本结构

```yaml
hotkeys:
  - type: "keyboard"  # 触发源类型：keyboard 或 gamepad
    key: "热键名称"   # 键盘热键名称（type=keyboard 时使用）
    key: "A"          # 手柄按键名称（type=gamepad 时使用）
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
- `key`: 手柄按键名称

**支持的按键：**
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

按顺序执行一系列按键、等待、文本输入和鼠标操作。

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

4. **mouse_click** - 鼠标点击
   - `button`: 鼠标按钮类型
     - `left`: 左键
     - `right`: 右键
     - `middle`: 中键
   - `delay` (可选): 点击后等待的毫秒数
     - 固定值: `delay: 50`
     - 随机范围: `delay: { min: 10, max: 30 }`

5. **mouse_action** - 鼠标按下/释放
   - `button`: 鼠标按钮类型 (`left`, `right`, `middle`)
   - `action`: 动作类型
     - `click`: 点击（按下+释放）
     - `down`: 按下（保持按住状态）
     - `up`: 释放
   - `delay` (可选): 动作后等待的毫秒数

6. **mouse_move** - 鼠标移动
   - `x`: X轴坐标（像素）
   - `y`: Y轴坐标（像素）
   - `relative` (可选): 是否相对移动
     - `true`: 相对当前位置移动
     - `false` 或不设置: 绝对移动到指定坐标
   - `delay` (可选): 移动后等待的毫秒数

7. **mouse_wheel** - 鼠标滚轮
   - `delta`: 滚轮滚动量
     - 正数: 向上滚动（通常为 120 的倍数）
     - 负数: 向下滚动
   - `delay` (可选): 滚动后等待的毫秒数

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
      # 鼠标左键点击
      - type: "mouse_click"
        button: "left"
        delay: 50
      # 鼠标右键点击
      - type: "mouse_click"
        button: "right"
        delay: 100
      # 鼠标绝对移动
      - type: "mouse_move"
        x: 500
        y: 300
        delay: 20
      # 鼠标相对移动（从当前位置向右100px，向下50px）
      - type: "mouse_move"
        x: 100
        y: 50
        relative: true
        delay: 20
      # 鼠标滚轮向上滚动
      - type: "mouse_wheel"
        delta: 120
      # 鼠标滚轮向下滚动
      - type: "mouse_wheel"
        delta: -120
      # 鼠标拖拽操作（按下左键 → 移动 → 释放）
      - type: "mouse_action"
        button: "left"
        action: "down"
      - type: "mouse_move"
        x: 200
        y: 100
        delay: 500
      - type: "mouse_action"
        button: "left"
        action: "up"
```

## 配置示例

### 示例 1: 键盘热键触发

```yaml
hotkeys:
  # 按 F2 输入 "hello"
  - type: keyboard
    key: F2
    action: type_text
    params:
      text: "hello"
      delay: 5

  # 按 F3 输入 "world"
  - type: keyboard
    key: F3
    action: type_text
    params:
      text: "world"
      delay: 10
```

### 示例 2: 手柄按键触发

```yaml
hotkeys:
  # 手柄 A 键触发空格键
  - type: gamepad
    key: A
    action: sequence
    params:
      steps:
        - type: key
          value: Space
          action: press
          delay: 50
        - type: key
          value: Space
          action: release
          delay: 50

  # 手柄 B 键输入文本
  - type: gamepad
    key: B
    action: type_text
    params:
      text: "Hello from gamepad!"
      delay: 10

  # 手柄 X 键执行复杂序列
  - type: gamepad
    key: X
    action: sequence
    params:
      steps:
        - type: key
          value: E
          delay: 20
        - type: key
          value: R
          delay: 20
        - type: key
          value: T
```

### 示例 3: 使用随机延迟

通过随机延迟让宏执行更具不确定性，模拟人工操作：

```yaml
hotkeys:
  - type: keyboard
    key: F5
    action: sequence
    params:
      steps:
        # 按键延迟在 10-30ms 之间随机
        - type: key
          value: A
          delay: { min: 10, max: 30 }
        
        # 等待时间在 0-500ms 之间随机
        - type: wait
          value: 500
          random: true
        
        # 输入文本，每个字符延迟在 5-15ms 之间随机
        - type: text
          value: "hello"
          delay: { min: 5, max: 15 }
```

### 示例 4: 分离按键按下和释放（高级）

通过 `action` 参数控制按键的按下和释放，实现组合键效果：

```yaml
hotkeys:
  - type: keyboard
    key: F4
    action: sequence
    params:
      steps:
        # 按下 Shift（保持按住状态）
        - type: key
          value: Shift
          action: press
        
        # 按下 A（由于Shift被按住，实际输入大写A）
        - type: key
          value: A
          action: press
        
        # 等待100毫秒
        - type: wait
          value: 100
        
        # 释放 A
        - type: key
          value: A
          action: release
        
        # 释放 Shift
        - type: key
          value: Shift
          action: release
```

### 示例 5: 鼠标控制操作（新功能）

使用鼠标控制功能实现自动化操作：

```yaml
hotkeys:
  # 简单鼠标点击
  - type: keyboard
    key: F6
    action: sequence
    params:
      steps:
        # 鼠标左键点击
        - type: mouse_click
          button: left
          delay: 100
        
        # 等待500毫秒
        - type: wait
          value: 500
        
        # 鼠标右键点击
        - type: mouse_click
          button: right
  
  # 鼠标移动和拖拽
  - type: keyboard
    key: F7
    action: sequence
    params:
      steps:
        # 移动到屏幕坐标 (500, 300)
        - type: mouse_move
          x: 500
          y: 300
          delay: 200
        
        # 按下鼠标左键
        - type: mouse_action
          button: left
          action: down
        
        # 相对移动（拖拽）
        - type: mouse_move
          x: 200
          y: 100
          relative: true
          delay: 500
        
        # 释放鼠标左键
        - type: mouse_action
          button: left
          action: up
  
  # 鼠标滚轮滚动
  - type: keyboard
    key: F8
    action: sequence
    params:
      steps:
        # 向上滚动（快速）
        - type: mouse_wheel
          delta: 600
          delay: 200
        
        # 向下滚动
        - type: mouse_wheel
          delta: -600
  
  # 双击操作
  - type: keyboard
    key: F9
    action: sequence
    params:
      steps:
        - type: mouse_click
          button: left
        - type: mouse_click
          button: left
          delay: 100
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

### 运行程序

直接运行可执行文件即可启动GUI界面：

```bash
# Windows
rust_keymacro.exe
```

程序启动后会显示主窗口，可以在界面中编辑配置、查看日志和监控状态。

## 配置文件位置

配置文件必须命名为 `config.yaml`，程序会按以下顺序查找：
1. 当前工作目录
2. 可执行文件所在目录

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

## 技术栈

- **GUI 框架**: egui + eframe 0.29
- **热键管理**: global-hotkey 0.6
- **手柄支持**: gilrs 0.11
- **配置处理**: serde + serde_yaml 0.9
- **Windows API**: windows 0.58
- **日志系统**: simplelog 0.12

## 许可证

MIT License
