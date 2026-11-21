//! Rime 配置核心逻辑库。
//!
//! - [`rime`]：Rime 配置文件操作核心（路径探测、YAML patch、配色、备份、部署触发）
//! - [`service`]：面向 UI 的服务层（原 Tauri commands 的逻辑本体，纯函数；长任务为同步函数，
//!   由各 UI 壳自行决定执行线程）
//!
//! 本 crate 不依赖任何 UI 框架；Tauri 壳与 GPUI 前端均直接调用。

pub mod rime;
pub mod service;

pub use rime::error::RimeError;
