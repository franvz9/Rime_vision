//! 方案配置页（对应 SchemeProfile.vue）。
//!
//! 「配置范围」下拉 + 4 个子面板 Tab（标点/快捷键/翻译器/候选词数）。
//! 面板常驻（切 Tab/scope 不销毁，仅重载数据）；「保存当前配置」调当前面板的 save。
use gpui_kit::component::{
    ActiveTheme,
    button::{Button, ButtonVariants as _},
    select::{Select, SelectEvent, SelectState},
    IndexPath,
    tab::{Tab, TabBar}, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use rime_core::service::schema;

use crate::widgets::{deploy_notice, hint};

type StringSelect = Entity<SelectState<Vec<String>>>;

/// Tab 顺序与名称（与 Vue 版一致）。
const TAB_LABELS: [&str; 4] = ["标点符号", "快捷键", "翻译器", "候选词数"];

pub struct ProfilePage {
    scope_select: StringSelect,
    schema_ids: Vec<String>,
    active_tab: usize,
    punct: Entity<crate::pages::punctuation::PunctPanel>,
    keybinding: Entity<crate::pages::keybinding::KeybindingPanel>,
    translator: Entity<crate::pages::translator::TranslatorPanel>,
    page_size: Entity<crate::pages::page_size::PageSizePanel>,
    _subscription: Subscription,
}

impl ProfilePage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let punct = cx.new(|cx| crate::pages::punctuation::PunctPanel::new(window, cx));
        let keybinding = cx.new(|cx| crate::pages::keybinding::KeybindingPanel::new(window, cx));
        let translator = cx.new(|cx| crate::pages::translator::TranslatorPanel::new(window, cx));
        let page_size = cx.new(|cx| crate::pages::page_size::PageSizePanel::new(window, cx));

        // scope 下拉：首项「全局默认」，其后各方案 id
        let schema_ids: Vec<String> = schema::get_schemas()
            .map(|d| d.schemas.into_iter().map(|s| s.schema_id).collect())
            .unwrap_or_default();
        let mut labels = vec!["全局默认".to_string()];
        labels.extend(schema_ids.iter().cloned());
        let scope_select: StringSelect =
            cx.new(|cx| SelectState::new(labels, Some(IndexPath::default().row(0)), window, cx));

        let sub = cx.subscribe_in(&scope_select, window, |this, _, ev: &SelectEvent<_>, window, cx| {
            if let SelectEvent::Confirm(Some(label)) = ev {
                let scope = (label != "全局默认").then(|| label.clone());
                this.punct.update(cx, |p, cx| p.reload(scope.clone(), window, cx));
                this.keybinding.update(cx, |p, cx| p.reload(scope.clone(), window, cx));
                this.translator.update(cx, |p, cx| p.reload(scope.clone(), window, cx));
                this.page_size.update(cx, |p, cx| p.reload(scope.clone(), window, cx));
            }
        });

        Self {
            scope_select,
            schema_ids,
            active_tab: 0,
            punct,
            keybinding,
            translator,
            page_size,
            _subscription: sub,
        }
    }

    /// 保存当前激活 Tab 的面板。
    fn save_active(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        match self.active_tab {
            0 => self.punct.update(cx, |p, cx| p.save(window, cx)),
            1 => self.keybinding.update(cx, |p, cx| p.save(window, cx)),
            2 => self.translator.update(cx, |p, cx| p.save(window, cx)),
            _ => self.page_size.update(cx, |p, cx| p.save(window, cx)),
        }
    }
}

impl Render for ProfilePage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cx_theme = cx.theme();
        let accent_bg = cx_theme.accent;
        let accent_fg = cx_theme.accent_foreground;
        let is_global = self
            .scope_select
            .read(cx)
            .selected_index(cx)
            .map(|ix| ix.row == 0)
            .unwrap_or(true);

        let scope_row = self.scope_select.read(cx).selected_index(cx).map(|ix| ix.row);
        let scope_file = match scope_row {
            None | Some(0) => "default.custom.yaml".to_string(),
            Some(row) => {
                let id = self.schema_ids.get(row - 1).cloned().unwrap_or_default();
                format!("{}.custom.yaml", id)
            }
        };

        let body = match self.active_tab {
            0 => self.punct.clone().into_any_element(),
            1 => self.keybinding.clone().into_any_element(),
            2 => self.translator.clone().into_any_element(),
            _ => self.page_size.clone().into_any_element(),
        };

        v_flex()
            .size_full()
            .gap_4()
            .max_w(px(900.))
            .child(deploy_notice(cx))
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(div().text_sm().child("配置范围"))
                    .child(Select::new(&self.scope_select).w(px(260.)))
                    .when(self.schema_ids.is_empty(), |el| {
                        el.child(hint(cx, "未加载到方案列表"))
                    }),
            )
            .child(
                // 范围说明区
                v_flex()
                    .gap_1()
                    .p_3()
                    .rounded_md()
                    .bg(accent_bg)
                    .text_color(accent_fg)
                    .child(
                        div().text_sm().when(is_global, |el| {
                            el.child("🌐 正在查看全局默认配置，它作为所有方案的基础。")
                        }).when(!is_global, |el| {
                            el.child("🌐 全局默认配置为各方案的基础，方案自定义的配置项以方案为准，优先生效。")
                        }),
                    )
                    .child(
                        div()
                            .text_sm()
                            .child(format!("下方配置写入 {}；未自定义的项显示「默认」，修改后保存即转为自定义。", scope_file)),
                    ),
            )
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(
                        TabBar::new("profile-tab")
                            .selected_index(self.active_tab)
                            .on_click(cx.listener(|this, ix: &usize, _, cx| {
                                this.active_tab = *ix;
                                cx.notify();
                            }))
                            .children(TAB_LABELS.map(|l| Tab::new().label(l))),
                    )
                    .child(
                        Button::new("profile-save")
                            .primary()
                            .label("保存当前配置")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.save_active(window, cx);
                            })),
                    ),
            )
            .child(body)
    }
}
