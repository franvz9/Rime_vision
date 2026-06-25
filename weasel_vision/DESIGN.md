# WeaselVision 设计文档

> Rime 输入法跨平台可视化配置工具

## 项目概述

WeaselVision 是 RimeVision 的跨平台版本，基于 Tauri 2.0 构建，支持 macOS（鼠须管）和 Windows（小狼毫）两个平台。

## 技术栈

| 组件 | 技术 | 说明 |
|------|------|------|
| 后端 | Rust + Tauri 2.0 | 核心逻辑、配置读写、跨平台适配 |
| 前端 | Vue 3 + TypeScript + Vite | UI 界面 |
| YAML 解析 | serde_yaml | 替代 Swift 版的 Yams |
| 配置备份 | 自研 | 带时间戳的增量备份 |

## 目录结构

```
weasel_vision/
├── weasel/                    # 小狼毫源码（参考用）
├── src-tauri/                 # Rust 后端
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── rime/              # Rime 配置操作核心
│   │   │   ├── mod.rs
│   │   │   ├── config.rs      # 配置路径、读写（跨平台）
│   │   │   ├── parser.rs      # YAML 解析
│   │   │   ├── patch.rs       # Rime patch 合并逻辑
│   │   │   ├── deployer.rs    # 部署触发（平台适配）
│   │   │   └── backup.rs      # 配置备份
│   │   ├── commands/          # Tauri IPC 命令
│   │   │   ├── mod.rs
│   │   │   ├── style.rs       # 样式相关命令
│   │   │   ├── schema.rs      # 方案管理命令
│   │   │   ├── grammar.rs     # 语言模型命令
│   │   │   ├── keybinding.rs  # 快捷键命令
│   │   │   └── settings.rs    # 通用设置命令
│   │   └── platform/          # 平台差异层
│   │       ├── mod.rs
│   │       ├── macos.rs       # macOS 适配
│   │       └── windows.rs     # Windows 适配
│   ├── Cargo.toml
│   └── tauri.conf.json
├── src/                       # Vue 3 前端
│   ├── App.vue
│   ├── components/
│   │   ├── Sidebar.vue
│   │   ├── ThemeEditor/
│   │   ├── SchemaManager/
│   │   ├── GrammarModel/
│   │   ├── KeybindingEditor/
│   │   ├── GeneralSettings/
│   │   └── PunctuationSettings/
│   ├── composables/
│   │   ├── useConfig.ts
│   │   └── useRime.ts
│   └── assets/
├── package.json
└── DESIGN.md
```

## 平台差异对比

### 配置文件

| 配置项 | macOS (鼠须管) | Windows (小狼毫) |
|--------|---------------|-----------------|
| 用户目录 | `~/Library/Rime` | `%AppData%\Rime` |
| 样式基础 | `squirrel.yaml` | `weasel.yaml` |
| 样式自定义 | `squirrel.custom.yaml` | `weasel.custom.yaml` |
| 通用配置 | `default.yaml` | `default.yaml` (**相同**) |
| 通用自定义 | `default.custom.yaml` | `default.custom.yaml` (**相同**) |

### 样式配置

两个平台的 `style` 和 `preset_color_schemes` 结构**高度一致**：

```yaml
# macOS: squirrel.yaml / Windows: weasel.yaml
style:
  color_scheme: native
  color_scheme_dark: native
  inline_preedit: true
  font_face: "PingFang SC"  # macOS 默认
  # Windows 默认: "Microsoft YaHei"
  font_point: 16
  # ... 其他样式属性相同

preset_color_schemes:
  native:
    back_color: 0xFFFFFF
    text_color: 0x000000
    # ... 颜色属性格式相同
```

### 部署机制

| 平台 | 机制 | 实现方式 |
|------|------|---------|
| macOS | DistributedNotificationCenter | 发送 `SquirrelReloadNotification` |
| Windows | WeaselDeployer | 调用 `WeaselDeployer.exe /deploy` |

### Windows 独有功能

Windows 版本有 `app_options` 配置，可针对不同应用设置不同的输入行为：

```yaml
# weasel.yaml
app_options:
  code.exe:
    ascii_mode: true
  cmd.exe:
    ascii_mode: true
```

## 核心模块设计

### 1. 配置路径 (config.rs)

```rust
pub struct RimeConfig {
    pub user_dir: PathBuf,
    pub style_file: String,       // "squirrel.yaml" 或 "weasel.yaml"
    pub style_custom: String,     // "squirrel.custom.yaml" 或 "weasel.custom.yaml"
    pub default_yaml: String,     // "default.yaml"（跨平台相同）
    pub default_custom: String,   // "default.custom.yaml"（跨平台相同）
}

impl RimeConfig {
    pub fn detect() -> Self {
        if cfg!(target_os = "macos") {
            Self {
                user_dir: dirs::home_dir().unwrap().join("Library/Rime"),
                style_file: "squirrel.yaml".into(),
                style_custom: "squirrel.custom.yaml".into(),
                default_yaml: "default.yaml".into(),
                default_custom: "default.custom.yaml".into(),
            }
        } else if cfg!(target_os = "windows") {
            Self {
                user_dir: dirs::config_dir().unwrap().join("Rime"),
                style_file: "weasel.yaml".into(),
                style_custom: "weasel.custom.yaml".into(),
                default_yaml: "default.yaml".into(),
                default_custom: "default.custom.yaml".into(),
            }
        } else {
            panic!("Unsupported platform")
        }
    }
}
```

### 2. Patch 合并 (patch.rs)

移植自 Swift 版的 `RimePatch.swift`，核心逻辑：

- `splitPath()`: 路径分割 (`"style/font_face"` → `["style", "font_face"]`)
- `expandedPatch()`: 展开斜杠路径
- `merge()`: 合并 base 和 patch，支持 `__delete__` 和 `__append__`
- `setValue()` / `removeValue()`: 嵌套字典操作

### 3. YAML 解析 (parser.rs)

使用 `serde_yaml` 替代自研解析器：

```rust
pub fn parse_yaml(text: &str) -> Result<serde_yaml::Value> {
    let value: serde_yaml::Value = serde_yaml::from_str(text)?;
    Ok(normalize(value))
}

pub fn dump_yaml(value: &serde_yaml::Value) -> Result<String> {
    Ok(serde_yaml::to_string(value)?)
}
```

### 4. 备份机制 (backup.rs)

```rust
pub fn write_if_changed(content: &str, path: &Path) -> Result<WriteResult> {
    if path.exists() {
        let existing = std::fs::read_to_string(path)?;
        if existing == content {
            return Ok(WriteResult::Unchanged);
        }
        // 创建带时间戳的备份
        let backup = timestamped_backup(path);
        std::fs::copy(path, backup)?;
    }
    std::fs::write(path, content)?;
    Ok(WriteResult::Written)
}

fn timestamped_backup(path: &Path) -> PathBuf {
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S%.3f");
    let ext = path.extension().unwrap_or_default().to_string_lossy();
    let stem = path.file_stem().unwrap().to_string_lossy();
    path.with_file_name(format!("{}.{}.{}", stem, ts, ext))
}
```

### 5. 部署触发 (deployer.rs)

```rust
pub fn deploy() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        // 通过 osascript 发送通知
        std::process::Command::new("osascript")
            .args(&["-e", r#"tell application "System Events" to ¬
                do shell script "defaults write ...""#])
            .output()?;
    }

    #[cfg(target_os = "windows")]
    {
        let deployer = dirs::program_files().unwrap()
            .join("Rime/WeaselDeployer.exe");
        std::process::Command::new(deployer)
            .arg("/deploy")
            .output()?;
    }
    Ok(())
}
```

## 数据模型

### RimeStyle

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RimeStyle {
    pub color_scheme_name: String,          // "native"
    pub color_scheme_dark_name: String,     // "native"
    pub status_message_type: String,        // "mix"
    pub candidate_format: String,           // "[label]. [candidate] [comment]"
    pub text_orientation: String,           // "horizontal"
    pub inline_preedit: bool,               // true
    pub inline_candidate: bool,             // false
    pub translucency: bool,                 // false
    pub font_face: String,                 // 平台默认字体
    pub font_point: f64,                   // 16
    pub label_font_face: String,           // "Lucida Grande" / "Segoe UI"
    pub label_font_point: f64,             // 16
    pub comment_font_face: String,         // 平台默认字体
    pub comment_font_point: f64,           // 14
    pub corner_radius: f64,                // 10
    pub line_spacing: f64,                 // 5
    pub spacing: f64,                      // 10
    // ... 其他属性
}
```

### RimeColorScheme

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RimeColorScheme {
    pub name: String,
    pub author: String,
    pub color_space: String,
    pub back_color: Option<RimeColor>,
    pub text_color: Option<RimeColor>,
    pub hilited_candidate_back_color: Option<RimeColor>,
    pub candidate_text_color: Option<RimeColor>,
    // ... 14 种颜色属性
}
```

### RimeColor

```rust
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RimeColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RimeColor {
    pub fn from_hex(hex: &str) -> Option<Self> {
        // 解析 0xBBGGRR 或 0xAABBGGRR 格式
    }

    pub fn to_hex(&self) -> String {
        if self.a < 255 {
            format!("0x{:02X}{:02X}{:02X}{:02X}", self.a, self.b, self.g, self.r)
        } else {
            format!("0x{:02X}{:02X}{:02X}", self.b, self.g, self.r)
        }
    }
}
```

## Tauri IPC 命令

### 样式相关

```rust
#[tauri::command]
fn get_style() -> Result<RimeStyle, String>

#[tauri::command]
fn save_style(style: RimeStyle) -> Result<(), String>

#[tauri::command]
fn get_color_schemes() -> Result<(Map, Map), String>  // (light, dark)

#[tauri::command]
fn save_color_scheme(name: String, scheme: RimeColorScheme) -> Result<(), String>

#[tauri::command]
fn delete_color_scheme(name: String, is_dark: bool) -> Result<(), String>
```

### 方案管理

```rust
#[tauri::command]
fn get_schemas() -> Result<Vec<RimeSchema>, String>

#[tauri::command]
fn save_schemas(schemas: Vec<RimeSchema>) -> Result<(), String>
```

### 快捷键

```rust
#[tauri::command]
fn get_keybindings() -> Result<Vec<KeyBinding>, String>

#[tauri::command]
fn save_keybindings(bindings: Vec<KeyBinding>) -> Result<(), String>
```

### 通用设置

```rust
#[tauri::command]
fn get_general_settings() -> Result<GeneralSettings, String>

#[tauri::command]
fn save_general_settings(settings: GeneralSettings) -> Result<(), String>
```

### 部署

```rust
#[tauri::command]
fn deploy() -> Result<(), String>
```

## 开发路线图

### Phase 1: 基础框架 (1-2 天)

- [x] Tauri 2.0 项目初始化
- [ ] Rust 后端骨架搭建
- [ ] Vue 3 前端骨架搭建
- [ ] 基础 IPC 通信测试

### Phase 2: 核心配置 (2-3 天)

- [ ] `config.rs` 跨平台路径检测
- [ ] `parser.rs` YAML 解析（serde_yaml）
- [ ] `patch.rs` Rime patch 合并逻辑
- [ ] `backup.rs` 配置备份机制

### Phase 3: 主题编辑器 (3-4 天)

- [ ] 配色方案列表展示
- [ ] 配色方案编辑（颜色选择器）
- [ ] 候选窗口实时预览
- [ ] 亮色/暗色主题切换

### Phase 4: 方案管理 (2-3 天)

- [ ] 输入方案列表
- [ ] 方案启用/禁用
- [ ] 方案依赖关系展示
- [ ] 方案复制/删除

### Phase 5: 其他模块 (3-4 天)

- [ ] 通用设置（候选词数、翻译器选项）
- [ ] 快捷键编辑
- [ ] 标点符号设置
- [ ] 语言模型管理

### Phase 6: 平台适配 (1-2 天)

- [ ] macOS 部署触发
- [ ] Windows 部署触发
- [ ] Windows `app_options` 支持
- [ ] 平台默认字体适配

### Phase 7: 打包发布 (1-2 天)

- [ ] macOS DMG 打包
- [ ] Windows NSIS 安装包
- [ ] CI/CD 配置
- [ ] 图标和品牌

## 从 Swift 版本移植的代码

| 源文件 | 目标文件 | 说明 |
|--------|---------|------|
| `RimePatch.swift` | `patch.rs` | 直接移植 patch 合并逻辑 |
| `ConfigBackup.swift` | `backup.rs` | 移植备份机制 |
| `RimeConfigStore.swift` | `config.rs` | 移植配置读写 |
| `RimeColor.swift` | `rime/color.rs` | 移植颜色解析 |
| `RimeStyle.swift` | `rime/style.rs` | 转为 Rust 结构体 |
| `ColorScheme.swift` | `rime/color_scheme.rs` | 转为 Rust 结构体 |
| `Schema.swift` | `rime/schema.rs` | 转为 Rust 结构体 |
| `GrammarModel.swift` | `rime/grammar.rs` | 转为 Rust 结构体 |

**不需要移植的代码**:
- `YAMLParser.swift` → 使用 `serde_yaml` 替代
- 所有 SwiftUI 视图 → 用 Vue 3 重写
- `KeyCaptureField.swift` → 前端实现
- `RimeDeployer.swift` → 重写为跨平台版本

## 注意事项

1. **颜色格式**: Rime 使用 `0xBBGGRR`（BGR 顺序），不是标准的 `0xRRGGBB`
2. **YAML 格式**: Rime 的 YAML 有特殊格式（如 `{ commit: "..." }`），需要确保序列化输出兼容
3. **配置文件编码**: 统一使用 UTF-8
4. **文件锁**: 写入配置时需要处理文件锁，避免与 Rime 引擎冲突
5. **默认值**: 不同平台的默认字体不同，需要按平台设置
