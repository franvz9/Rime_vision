# RimeVision

[Rime 输入法](https://rime.im/)可视化配置工具，无需手动编辑 YAML，通过直观的图形界面管理所有配置。

## 项目组成

本仓库包含两个独立的配置工具：

| 项目 | 目录 | 平台 | 技术栈 | 状态 |
|------|------|------|--------|------|
| **RimeVision** | `RimeVision/` | macOS | Swift + SwiftUI | 稳定版 |
| **WeaselVision** | `weasel_vision/` | macOS + Windows | Rust + Tauri 2.0 + Vue 3 | MVP（未真机测试） |

> **关于 WeaselVision**: 跨平台版本已完成开发并通过多轮代码审查，逻辑上支持 macOS（鼠须管）和 Windows（小狼毫）。但由于作者身边没有 Windows 设备，**尚未在 Windows 上进行真机测试**。macOS 端功能与 RimeVision 一致。

---

## RimeVision（macOS 版）

### 功能特性

- **主题编辑器** — 可视化编辑配色方案，实时预览候选窗口效果
- **方案管理** — 查看、配置输入法方案及其依赖关系
- **语言模型** — 扫描、导入、挂载/卸载 `.gram` 语言模型文件
- **快捷键编辑** — 直观配置全局按键绑定
- **通用设置** — 候选词数、标点符号等全局选项
- **配置安全** — 结构化读写 YAML，自动备份，防止配置损坏
- **一键部署** — 将配置部署到 Rime 输入法引擎

### 系统要求

- macOS 13 (Ventura) 或更高版本
- Apple Silicon (arm64)
- 已安装 [鼠须管 (Squirrel)](https://github.com/rime/squirrel)

### 安装

从 [Releases](../../releases) 下载最新的 DMG 文件，打开后将 RimeVision.app 拖入"应用程序"文件夹。

首次启动时，macOS 可能提示"无法验证开发者"，请前往**系统设置 → 隐私与安全性**点击"仍要打开"。

### 从源码构建

```bash
git clone https://github.com/franvz9/Rime_vision.git
cd Rime_vision/RimeVision
swift run RimeVision
```

运行测试：

```bash
swift run RimeVisionTestRunner
```

---

## WeaselVision（跨平台版）

### 功能特性

- **主题外观** — 可视化编辑配色方案，实时预览，支持亮色/暗色切换
- **方案管理** — 启用/禁用输入方案，调整候选词数
- **语言模型** — 管理 `.gram` 文件，批量挂载/卸载
- **快捷键** — 配置全局按键绑定
- **通用设置** — 候选词、翻译器、方案切换器、中英文切换
- **标点符号** — 编辑半角/全角标点规则
- **高级设置** — 同步、重置、配置文件状态

### 系统要求

| 平台 | 要求 |
|------|------|
| macOS | macOS 13+，已安装 [鼠须管](https://github.com/rime/squirrel) |
| Windows | Windows 10 (1803+)，已安装 [小狼毫](https://github.com/rime/weasel) |

### 从源码构建

需要 [Rust](https://www.rust-lang.org/tools/install) 和 [Node.js](https://nodejs.org/) (LTS)：

```bash
cd weasel_vision
npm install
npm run tauri dev      # 开发模式
npm run tauri build    # 构建安装包
```

### 当前状态

- ✅ 功能开发完成，覆盖 RimeVision 全部核心功能
- ✅ 七轮代码审查，所有发现的问题已修复
- ⚠️ **未在 Windows 真机测试**（作者无 Windows 设备）
- ⚠️ macOS 端尚未验证 Tauri 打包后的运行效果

如果你有 Windows 设备并安装了小狼毫，欢迎测试反馈。

---

## 技术栈

| 组件 | RimeVision (macOS) | WeaselVision (跨平台) |
|------|-------------------|----------------------|
| 语言 | Swift 5.9 | Rust |
| UI 框架 | SwiftUI + AppKit | Vue 3 + TypeScript |
| 框架 | — | Tauri 2.0 |
| YAML 解析 | [Yams](https://github.com/jpsim/Yams) | serde_yaml |
| 构建系统 | Swift Package Manager | Cargo + npm |

## 许可证

[MIT License](LICENSE)
