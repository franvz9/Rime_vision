//! 高级设置页（对应 AdvancedSettings.vue）。
//!
//! Rime 目录 / 配置文件列表 / 操作（重置）/ 关于。
use gpui_kit::assets::IconName;
use gpui_kit::component::Icon;
use gpui_kit::component::{
    button::{Button, ButtonVariants as _},
    h_flex,
    scroll::ScrollableElement as _,
    v_flex, ActiveTheme, WindowExt as _,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use rime_core::service::settings::{self, ConfigFileInfo};

use crate::widgets::{hint, section_card};

pub struct AdvancedPage {
    user_dir: String,
    config_files: Vec<ConfigFileInfo>,
}

impl AdvancedPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let user_dir = settings::get_rime_user_dir().unwrap_or_default();
        let config_files = settings::get_config_files().unwrap_or_default();
        let this = Self {
            user_dir,
            config_files,
        };
        // 首次进入提示一次加载失败场景
        if this.user_dir.is_empty() {
            window.push_notification("未能定位 Rime 用户目录", cx);
        }
        this
    }

    fn reload(&mut self, cx: &mut Context<Self>) {
        self.user_dir = settings::get_rime_user_dir().unwrap_or_default();
        self.config_files = settings::get_config_files().unwrap_or_default();
        cx.notify();
    }

    fn open_dir(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Err(e) = settings::open_rime_dir() {
            window.push_notification(format!("打开目录失败：{}", e), cx);
        }
    }

    fn reset_config(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Err(e) = settings::reset_config() {
            window.push_notification(format!("重置失败：{}", e), cx);
            return;
        }
        self.reload(cx);
        window.push_notification("配置已重置", cx);
    }

    /// 确认重置：用系统级 alert 弹窗（M7：原手搓模态遮罩为纯黑不透明、无居中）。
    fn confirm_reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let names: Vec<String> = self
            .config_files
            .iter()
            .filter(|f| !f.is_main)
            .map(|f| f.name.clone())
            .collect();
        let custom_file_names = if names.is_empty() {
            "自定义配置文件".to_string()
        } else {
            names.join("、")
        };
        let weak = cx.weak_entity();
        window.open_alert_dialog(cx, move |alert, _, _| {
            let weak = weak.clone();
            let custom_file_names = custom_file_names.clone();
            alert
                .confirm()
                .title("确认重置")
                .description(format!(
                    "将删除自定义配置文件（{}），此操作不可撤销。",
                    custom_file_names
                ))
                .ok_text("确认重置")
                .on_ok(move |_, window, cx| {
                    let _ = weak.update(cx, |this, cx| {
                        this.reset_config(window, cx);
                    });
                    true
                })
        });
    }
}

impl Render for AdvancedPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cx_theme = cx.theme();
        let muted = cx_theme.muted_foreground;
        let border = cx_theme.border;

        let file_rows: Vec<AnyElement> = self
            .config_files
            .iter()
            .map(|f| {
                let name = f.name.clone();
                let desc = if f.is_main {
                    "主配置文件"
                } else {
                    "自定义配置 (patch)"
                };
                h_flex()
                    .gap_3()
                    .items_center()
                    .py_2()
                    .border_b_1()
                    .border_color(border)
                    .child(Icon::new(IconName::FileText))
                    .child(
                        v_flex()
                            .child(
                                // 文件名超链接：点击在文件管理器中定位该文件
                                div()
                                    .id(SharedString::from(format!("cfg-file-{name}")))
                                    .when(f.exists, |el| el)
                                    .text_sm()
                                    .font_weight(FontWeight::MEDIUM)
                                    .cursor_pointer()
                                    .text_color(cx_theme.primary)
                                    .hover(|s| s.underline())
                                    .when(f.exists, |el| {
                                        el.on_click({
                                            let name = name.clone();
                                            cx.listener(move |this, _, window, cx| {
                                                let path = std::path::Path::new(&this.user_dir)
                                                    .join(&name)
                                                    .display()
                                                    .to_string();
                                                match settings::open_file_in_manager(path) {
                                                    Ok(()) => {}
                                                    Err(e) => {
                                                        window.push_notification(
                                                            format!("打开文件位置失败：{}", e),
                                                            cx,
                                                        );
                                                    }
                                                }
                                            })
                                        })
                                    })
                                    .child(SharedString::from(name.clone())),
                            )
                            .child(div().text_sm().text_color(muted).child(desc)),
                    )
                    .when(!f.exists, |el| {
                        el.child(
                            div()
                                .flex_1()
                                .text_right()
                                .text_sm()
                                .text_color(muted)
                                .child("保存设置后自动生成"),
                        )
                    })
                    .when(!f.exists, |el| el.opacity(0.5))
                    .into_any_element()
            })
            .collect();

        v_flex().size_full().overflow_y_scrollbar().child(
            v_flex()
                .max_w(px(720.))
                .gap_4()
                .child(section_card(
                    cx,
                    "Rime 目录",
                    Some(
                        Button::new("adv-open")
                            .outline()
                            .label("打开")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_dir(window, cx);
                            }))
                            .into_any_element(),
                    ),
                    v_flex()
                        .child(
                            // 可点击的目录路径
                            div()
                                .id("user-dir")
                                .text_sm()
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(cx_theme.primary)
                                .cursor_pointer()
                                .hover(|s| s.underline())
                                .child(self.user_dir.clone())
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.open_dir(window, cx);
                                })),
                        )
                        .child(hint(cx, "点击路径或按钮可在文件管理器中打开 Rime 用户目录"))
                        .into_any_element(),
                ))
                .child(section_card(
                    cx,
                    "配置文件",
                    None,
                    div().children(file_rows).into_any_element(),
                ))
                .child(section_card(
                    cx,
                    "操作",
                    None,
                    v_flex()
                        .gap_2()
                        .child(
                            Button::new("adv-reset")
                                .danger()
                                .label("重置自定义配置")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.confirm_reset(window, cx);
                                })),
                        )
                        .into_any_element(),
                ))
                .child(section_card(
                    cx,
                    "关于",
                    None,
                    v_flex()
                        .gap_1()
                        .child(
                            div()
                                .text_sm()
                                .child("WeaselVision — Rime 输入法可视化配置工具"),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(muted)
                                .child("GPUI 原生版 · 本工具不参与输入法核心分发"),
                        )
                        .into_any_element(),
                )),
        )
    }
}
