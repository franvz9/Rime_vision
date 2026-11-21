//! WeaselVision GPUI 前端入口。
//!
//! 窗口 1100×700（最小 900×600），与 Tauri 版保持一致；
//! 亮/暗主题跟随系统；逻辑直接调用 rime-core service 层。

mod app;
mod pages;
mod widgets;

use app::WeaselApp;
use gpui_kit::assets::Assets;
use gpui_kit::component::Theme;
use gpui_kit::*;

fn main() {
    let app = gpui_kit::application().with_assets(Assets);

    app.run(move |cx| {
        gpui_kit::init(cx);

        // 亮/暗主题跟随系统
        Theme::sync_system_appearance(None, cx);

        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::centered(size(px(1100.), px(700.)), cx)),
            window_min_size: Some(gpui_kit::Size {
                width: px(900.),
                height: px(600.),
            }),
            titlebar: Some(TitlebarOptions {
                title: Some("WeaselVision - Rime 配置工具".into()),
                ..Default::default()
            }),
            ..Default::default()
        };

        gpui_kit::open_window(window_options, cx, |window, cx| {
            // 全局基准字号 12px（默认 16 偏大）
            window.set_rem_size(px(12.));
            cx.new(|cx| {
                // 系统亮/暗切换时自动换主题
                cx.observe_window_appearance(window, |_, window, cx| {
                    Theme::sync_system_appearance(Some(window), cx);
                })
                .detach();
                WeaselApp::new(window, cx)
            })
        })
        .expect("failed to open window");
    });
}
