# RimeVision

macOS 上的 Rime 输入法可视化配置工具，基于 SwiftUI 构建。

## 功能

- 🎨 **主题编辑器** — 可视化编辑 Rime 配色方案和样式
- 📋 **方案管理** — 管理输入法方案及其依赖关系
- ⌨️ **快捷键编辑** — 配置按键绑定
- ⚙️ **通用设置** — 标点符号、候选词等全局选项
- 💾 **配置管理** — YAML 配置文件的导入、导出和备份
- 🚀 **一键部署** — 将配置部署到 Rime 输入法

## 技术栈

- **语言**: Swift
- **UI 框架**: SwiftUI
- **最低系统**: macOS 13 (Ventura)
- **依赖**: [Yams](https://github.com/jpsim/Yams) (YAML 解析)

## 构建 & 运行

```bash
cd RimeVision
swift run RimeVision
```

运行测试:

```bash
swift run RimeVisionTestRunner
```
