//! Tauri IPC 命令薄壳聚合模块：全部为 `#[tauri::command]` 委托，
//! 实际逻辑统一在 rime-core::service（纯函数层），供 GPUI 前端直接复用。

pub mod backup;
pub mod dict;
pub mod grammar;
pub mod keybinding;
pub mod page_size;
pub mod punctuation;
pub mod schema;
pub mod settings;
pub mod style;
pub mod sync;
pub mod translator;
