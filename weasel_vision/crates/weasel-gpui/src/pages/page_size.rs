//! 候选词数子面板（对应 PageSizeSettings.vue，SchemeProfile 内嵌）。
use gpui_kit::component::{
    WindowExt as _,
    ActiveTheme,
    form::{Field, Form},
    input::{Input, InputEvent, InputState},
    h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use rime_core::service::page_size;

pub struct PageSizePanel {
    scope: Option<String>,
    page_size: i64,
    is_custom: bool,
    dirty: bool,
    input: Entity<InputState>,
    _subscription: Option<Subscription>,
}

impl PageSizePanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut this = Self {
            scope: None,
            page_size: 5,
            is_custom: false,
            dirty: false,
            input: cx.new(|cx| {
                InputState::new(window, cx).placeholder("1–20").default_value("5")
            }),
            _subscription: None,
        };
        this.reload(None, window, cx);
        let input = this.input.clone();
        this._subscription = Some(cx.subscribe_in(
            &input,
            window,
            |this, input, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    let raw = input.read(cx).value().to_string();
                    this.dirty = true;
                    if let Ok(v) = raw.trim().parse::<i64>() {
                        this.page_size = v.clamp(1, 20);
                    }
                    cx.notify();
                }
            },
        ));
        this
    }

    pub fn reload(&mut self, scope: Option<String>, window: &mut Window, cx: &mut Context<Self>) {
        self.scope = scope.clone();
        match page_size::get_page_size_detail(scope) {
            Ok(d) => {
                self.page_size = d.page_size;
                self.is_custom = d.is_custom;
                let v = d.page_size.to_string();
                self.input.update(cx, |s, cx| s.set_value(v.as_str(), window, cx));
            }
            Err(e) => eprintln!("Warning: load page size failed: {}", e),
        }
        self.dirty = false;
        cx.notify();
    }

    pub fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Err(e) = page_size::save_page_size(self.scope.clone(), self.page_size) {
            window.push_notification(format!("保存候选词数失败：{}", e), cx);
            return;
        }
        let scope = self.scope.clone();
        self.reload(scope, window, cx);
        window.push_notification("候选词数已保存", cx);
    }
}

impl Render for PageSizePanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cx_theme = cx.theme();
        let badge_bg = cx_theme.secondary;
        let badge_fg = cx_theme.secondary_foreground;

        let body = Form::horizontal()
            .label_width(px(120.))
            .child(
                Field::new()
                    .label("候选词数")
                    .description("候选菜单每页显示的候选数量（1–20）")
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(Input::new(&self.input).w(px(100.)))
                            .child(
                                div()
                                    .text_xs()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_md()
                                    .bg(badge_bg)
                                    .text_color(badge_fg)
                                    .when(self.is_custom || self.dirty, |el| el.child("自定义"))
                                    .when(!(self.is_custom || self.dirty), |el| el.child("默认")),
                            ),
                    ),
            );

        v_flex().gap_3().child(body)
    }
}
