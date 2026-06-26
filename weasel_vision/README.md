# WeaselVision

Rime 输入法跨平台可视化配置工具。基于 Tauri 2.0 构建，支持 macOS（鼠须管）和 Windows（小狼毫）。

无需手动编辑 YAML，通过直观的图形界面管理输入法的所有配置。

## 功能特性

- **主题外观** — 可视化编辑配色方案，实时预览候选窗口效果，支持亮色/暗色主题切换
- **方案管理** — 查看、启用/禁用输入方案，调整每页候选词数
- **语言模型** — 管理 .gram 语言模型文件，批量挂载/卸载到输入方案
- **快捷键** — 直观配置全局按键绑定
- **通用设置** — 候选词数、翻译器选项、方案切换器、中英文切换
- **标点符号** — 编辑半角/全角标点规则，支持直接上屏、配对输入、候选列表三种类型
- **配置备份** — 手动/自动/部署前备份，备份列表、差异对比、一键回滚
- **词典管理** — 词条浏览/搜索/排序/分页、词频编辑、批量删除低频词条、导出码表、清空词典
- **同步管理** — 基于共享文件夹的多设备词典同步，支持配置上传/下载/合并
- **高级设置** — 同步用户数据、重置配置、查看配置文件状态

## 系统要求

### macOS

- macOS 13 (Ventura) 或更高版本
- 已安装 [鼠须管 (Squirrel)](https://github.com/rime/squirrel)

### Windows

- Windows 10 (1803+) 或更高版本
- 已安装 [小狼毫 (Weasel)](https://github.com/rime/weasel)

## 安装

从 [Releases](../../releases) 下载对应平台的安装包：

- **macOS**: `.dmg` 文件，打开后拖入「应用程序」文件夹
- **Windows**: `.exe` 安装包，双击运行

## 从源码构建

### 前置要求

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) (LTS)
- macOS: Xcode 或 Xcode Command Line Tools
- Windows: Microsoft C++ Build Tools

### 构建步骤

```bash
git clone https://github.com/franvz9/Rime_vision.git
cd Rime_vision/weasel_vision
npm install
npm run tauri build
```

### 开发模式

```bash
npm run tauri dev
```

## 使用说明

1. 启动 WeaselVision
2. 在左侧导航栏选择要配置的模块
3. 修改配置后点击「保存」或工具栏中的「重新部署」使配置生效

### 配置文件位置

| 平台 | 用户目录 |
|------|---------|
| macOS | `~/Library/Rime` |
| Windows | `%AppData%\Rime` |

WeaselVision 会自动读取和写入该目录下的配置文件。修改配置前会自动创建带时间戳的备份，也可通过「备份管理」手动创建和管理备份。

## 项目状态

- ✅ 功能开发完成，覆盖全部核心功能 + 备份/词典/同步三大模块
- ✅ 多轮无偏代码审查，共修复 19 项问题
- ✅ `cargo test` 全部通过，`cargo clippy` 零警告
- ⚠️ **未在 Windows 真机测试**（作者无 Windows 设备）
- ⚠️ macOS 端尚未验证 Tauri 打包后的运行效果

## 技术栈

| 组件 | 技术 |
|------|------|
| 后端 | Rust + Tauri 2.0 |
| 前端 | Vue 3 + TypeScript + Vite |
| YAML 解析 | serde_yaml |
| 安装包 | DMG (macOS) / NSIS (Windows) |

## 许可证

[MIT License](../LICENSE)
