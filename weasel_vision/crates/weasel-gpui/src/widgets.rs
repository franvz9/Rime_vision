//! 跨页面通用小部件：部署提示条、设置区块卡片、表单行辅助。

use gpui_kit::component::Icon;
use gpui_kit::component::{h_flex, ActiveTheme as _, Sizable as _};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

/// 黄色部署提示条（对应 DeployNotice.vue）。
pub fn deploy_notice(cx: &App) -> AnyElement {
    h_flex()
        .w_full()
        .rounded_md()
        .px_3()
        .py_2()
        .gap_2()
        .text_sm()
        .bg(cx.theme().warning)
        .text_color(cx.theme().warning_foreground)
        .child(
            Icon::new(gpui_kit::assets::IconName::TriangleAlert)
                .with_size(gpui_kit::component::Size::Small),
        )
        .child("本页配置修改保存后，需要重新部署才能生效")
        .into_any_element()
}

/// 设置区块：标题行（标题 + 右侧可选操作）+ 内容，边框卡片样式。
pub fn section_card(
    cx: &App,
    title: impl Into<SharedString>,
    header_right: Option<AnyElement>,
    body: AnyElement,
) -> AnyElement {
    div()
        .w_full()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .p_4()
        .child(
            h_flex()
                .justify_between()
                .items_center()
                .mb_3()
                .child(div().font_weight(FontWeight::MEDIUM).child(title.into()))
                .when_some(header_right, |this, el| this.child(el)),
        )
        .child(body)
        .into_any_element()
}

/// 只读说明行（灰色小字）。
pub fn hint(cx: &App, text: impl Into<SharedString>) -> AnyElement {
    div()
        .text_sm()
        .text_color(cx.theme().muted_foreground)
        .child(text.into())
        .into_any_element()
}
