# RimeVision

macOS 上的 [Rime 输入法](https://rime.im/)可视化配置工具，基于 SwiftUI 构建。

无需手动编辑 YAML，通过直观的图形界面管理输入法方案、主题配色、快捷键、语言模型等所有配置。

## 功能特性

- 🎨 **主题编辑器** — 可视化编辑配色方案，实时预览候选窗口效果
- 📋 **方案管理** — 查看、配置输入法方案及其依赖关系
- 🧠 **语言模型** — 扫描、导入、挂载/卸载 `.gram` 语言模型文件
- ⌨️ **快捷键编辑** — 直观配置全局按键绑定
- ⚙️ **通用设置** — 候选词数、标点符号等全局选项
- 💾 **配置安全** — 结构化读写 YAML，自动备份，防止配置损坏
- 🚀 **一键部署** — 将配置部署到 Rime 输入法引擎

## 系统要求

- macOS 13 (Ventura) 或更高版本
- Apple Silicon (arm64)
- 已安装 [鼠须管 (Squirrel)](https://github.com/rime/squirrel) 或其他 Rime 前端

## 安装

从 [Releases](../../releases) 下载最新的 DMG 文件，打开后将 RimeVision.app 拖入"应用程序"文件夹。

首次启动时，macOS 可能提示"无法验证开发者"，请前往**系统设置 → 隐私与安全性**点击"仍要打开"。

## 从源码构建

```bash
git clone https://github.com/franvz9/Rime_vision.git
cd Rime_vision/RimeVision
swift run RimeVision
```

运行测试：

```bash
swift run RimeVisionTestRunner
```

## 技术栈

| 组件 | 技术 |
|------|------|
| 语言 | Swift 5.9 |
| UI 框架 | SwiftUI + AppKit |
| YAML 解析 | [Yams](https://github.com/jpsim/Yams) |
| 构建系统 | Swift Package Manager |

## 许可证

[MIT License](LICENSE)
