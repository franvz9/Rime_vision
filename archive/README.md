# RimeVision (Swift 版归档)

macOS 专用 Rime 配置工具，已归档。

## 归档说明

RimeVision 是项目的初始版本，使用 Swift + SwiftUI 开发，仅支持 macOS 平台。
随着跨平台版本 WeaselVision 的完成和稳定，此版本已归档保留。

## 技术栈

- Swift 5.9 + SwiftUI
- Swift Package Manager
- [Yams](https://github.com/jpsim/Yams) (YAML 解析)

## 从源码运行（仅限 macOS）

```bash
cd archive/RimeVision
swift run RimeVision
```

## 历史功能

- 主题编辑器（配色方案可视化编辑 + 实时预览）
- 方案管理（查看/配置输入法方案）
- 语言模型管理（扫描/导入/挂载 .gram 文件）
- 快捷键编辑器
- 通用设置（候选词数、标点符号）
- 配置安全（结构化 YAML 读写 + 自动备份）
- 一键部署

## 为什么归档

1. 仅支持 macOS，无法覆盖 Windows 用户
2. 跨平台版本 WeaselVision 已实现全部核心功能 + 新增模块（备份管理、词典管理、同步管理）
3. 维护两套代码的成本过高
