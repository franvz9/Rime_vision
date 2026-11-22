//! 标点符号子面板（对应 PunctuationSettings.vue，SchemeProfile 内嵌）。
use gpui_kit::component::{
    button::{Button, ButtonVariants as _},
    dialog::DialogButtonProps,
    h_flex,
    input::{Input, InputState},
    radio::{Radio, RadioGroup},
    tab::{Tab, TabBar},
    v_flex, ActiveTheme, WindowExt as _,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use rime_core::service::punctuation::{self, PunctRule};
use std::collections::HashSet;

pub struct PunctPanel {
    scope: Option<String>,
    active_half: bool,
    /// 编辑模态当前选中的类型（0=commit 1=pair 2=list）
    edit_kind: std::cell::Cell<u8>,
    half: Vec<PunctRule>,
    full: Vec<PunctRule>,
    /// 原始 custom 集合（保存时与 dirty 合并过滤）。
    custom_keys: HashSet<String>,
    /// 本次编辑过的键。
    dirty_keys: HashSet<String>,
}

impl PunctPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut this = Self {
            scope: None,
            active_half: true,
            edit_kind: std::cell::Cell::new(0),
            half: Vec::new(),
            full: Vec::new(),
            custom_keys: HashSet::new(),
            dirty_keys: HashSet::new(),
        };
        this.reload(None, window, cx);
        this
    }

    pub fn reload(&mut self, scope: Option<String>, _: &mut Window, cx: &mut Context<Self>) {
        self.scope = scope.clone();
        match punctuation::get_punctuation_profile(scope) {
            Ok(p) => {
                self.half = p
                    .half_shape
                    .into_iter()
                    .filter(|r| !r.key.is_empty())
                    .collect();
                self.full = p
                    .full_shape
                    .into_iter()
                    .filter(|r| !r.key.is_empty())
                    .collect();
                self.custom_keys = p.custom_keys.into_iter().collect();
            }
            Err(e) => eprintln!("Warning: load punctuation failed: {}", e),
        }
        self.dirty_keys.clear();
        cx.notify();
    }

    pub fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let keep = |keys: &HashSet<String>, rules: &[PunctRule]| -> Vec<PunctRule> {
            rules
                .iter()
                .filter(|r| keys.contains(&r.key))
                .cloned()
                .collect()
        };
        let mut keys = self.custom_keys.clone();
        keys.extend(self.dirty_keys.clone());
        let half = keep(&keys, &self.half);
        let full = keep(&keys, &self.full);
        if let Err(e) = punctuation::save_punctuation(self.scope.clone(), half, full) {
            window.push_notification(format!("保存标点符号失败：{}", e), cx);
            return;
        }
        let scope = self.scope.clone();
        self.reload(scope, window, cx);
        window.push_notification("标点符号已保存", cx);
    }

    fn rules(&self) -> &[PunctRule] {
        if self.active_half {
            &self.half
        } else {
            &self.full
        }
    }

    fn rules_mut(&mut self) -> &mut Vec<PunctRule> {
        if self.active_half {
            &mut self.half
        } else {
            &mut self.full
        }
    }

    fn upsert_rule(&mut self, rule: PunctRule) {
        self.dirty_keys.insert(rule.key.clone());
        let rules = self.rules_mut();
        if let Some(slot) = rules.iter_mut().find(|r| r.key == rule.key) {
            *slot = rule;
        } else {
            rules.push(rule);
            rules.sort_by(|a, b| a.key.cmp(&b.key));
        }
    }

    fn delete_rule(&mut self, key: &str, cx: &mut Context<Self>) {
        self.rules_mut().retain(|r| r.key != key);
        self.dirty_keys.remove(key);
        self.custom_keys.remove(key);
        cx.notify();
    }

    /// 打开编辑/新增模态框。
    fn open_editor(
        &mut self,
        edit_key: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let existing = edit_key
            .as_ref()
            .and_then(|k| self.rules().iter().find(|r| &r.key == k));
        let (kind, default_value) = match existing {
            Some(r) if !r.pair.is_empty() => (1u8, r.pair.join(",")),
            Some(r) if !r.list.is_empty() => (2u8, r.list.join(",")),
            Some(r) => (0u8, r.commit.clone()),
            None => (0u8, String::new()),
        };
        let is_half = self.active_half;
        self.edit_kind.set(kind);
        let key_input = cx.new(|cx| {
            let mut s = InputState::new(window, cx).placeholder("按键，如 。（最多 4 字符）");
            if let Some(k) = &edit_key {
                s = s.default_value(k.clone());
            }
            s
        });
        let value_input = cx.new(|cx| {
            let mut s = InputState::new(window, cx).placeholder("输出内容；配对/候选用逗号分隔");
            if !default_value.is_empty() {
                s = s.default_value(default_value);
            }
            s
        });
        let weak = cx.weak_entity();

        let self_kind = self.edit_kind.clone();
        let editing_key = edit_key.clone();
        let title = if edit_key.is_some() {
            "编辑标点"
        } else {
            "新增标点"
        };
        window.open_dialog(cx, move |dialog, _, _| {
            dialog
                .title(title)
                .child(
                    v_flex()
                        .gap_3()
                        .child(
                            v_flex()
                                .gap_1()
                                .child(div().text_sm().child("按键"))
                                .child(Input::new(&key_input)),
                        )
                        .child(
                            v_flex().gap_1().child(div().text_sm().child("类型")).child(
                                RadioGroup::horizontal("punct-kind")
                                    .child(Radio::new("kind-commit").label("直接上屏"))
                                    .child(Radio::new("kind-pair").label("配对输入"))
                                    .child(Radio::new("kind-list").label("候选列表"))
                                    .selected_index(Some(self_kind.get() as usize))
                                    .on_click({
                                        let weak = weak.clone();
                                        move |ix: &usize, _, cx| {
                                            weak.update(cx, |this, _| {
                                                this.edit_kind.set(*ix as u8);
                                            })
                                            .ok();
                                            cx.refresh_windows();
                                        }
                                    }),
                            ),
                        )
                        .child(
                            v_flex()
                                .gap_1()
                                .child(div().text_sm().child("输出"))
                                .child(Input::new(&value_input)),
                        ),
                )
                .button_props(DialogButtonProps::default().ok_text("保存").on_ok({
                    let weak = weak.clone();
                    let key_input = key_input.clone();
                    let value_input = value_input.clone();
                    let editing_key = editing_key.clone();
                    move |_, _, cx| {
                        let key = key_input.read(cx).value().to_string();
                        let value = value_input.read(cx).value().to_string();
                        let is_half = is_half;
                        let editing_key = editing_key.clone();
                        let _ = weak.update(
                            cx,
                            |this: &mut PunctPanel, cx: &mut Context<PunctPanel>| {
                                let kind = this.edit_kind.get();
                                if key.is_empty() {
                                    return;
                                }
                                let rule = match kind {
                                    1 => PunctRule {
                                        key: key.clone(),
                                        commit: String::new(),
                                        pair: value
                                            .split(',')
                                            .map(|s| s.trim().to_string())
                                            .filter(|s| !s.is_empty())
                                            .collect(),
                                        list: Vec::new(),
                                    },
                                    2 => PunctRule {
                                        key: key.clone(),
                                        commit: String::new(),
                                        pair: Vec::new(),
                                        list: value
                                            .split(',')
                                            .map(|s| s.trim().to_string())
                                            .filter(|s| !s.is_empty())
                                            .collect(),
                                    },
                                    _ => PunctRule {
                                        key: key.clone(),
                                        commit: value,
                                        pair: Vec::new(),
                                        list: Vec::new(),
                                    },
                                };
                                // 编辑时若改了键名，先移除旧规则
                                if let Some(old) = &editing_key {
                                    if old != &key {
                                        if is_half {
                                            this.half.retain(|r| &r.key != old);
                                        } else {
                                            this.full.retain(|r| &r.key != old);
                                        }
                                        this.dirty_keys.remove(old);
                                    }
                                }
                                this.upsert_rule(rule);
                                cx.notify();
                            },
                        );
                        true
                    }
                }))
                .overlay_closable(true)
        });
    }
}

fn rule_value(r: &PunctRule) -> String {
    if !r.commit.is_empty() {
        r.commit.clone()
    } else if !r.pair.is_empty() {
        format!("[{}]", r.pair.join(", "))
    } else {
        format!("[{}]", r.list.join(", "))
    }
}

fn rule_kind(r: &PunctRule) -> &'static str {
    if !r.commit.is_empty() {
        "上屏"
    } else if !r.pair.is_empty() {
        "配对"
    } else {
        "候选"
    }
}

impl Render for PunctPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cx_theme = cx.theme();
        let muted = cx_theme.muted_foreground;
        let border = cx_theme.border;
        let accent_bg = cx_theme.accent;
        let accent_fg = cx_theme.accent_foreground;
        let is_half = self.active_half;

        let header = h_flex()
            .justify_between()
            .items_center()
            .child(
                TabBar::new("punct-tab")
                    .selected_index(if is_half { 0 } else { 1 })
                    .on_click(cx.listener(|this, ix: &usize, _, cx| {
                        this.active_half = *ix == 0;
                        cx.notify();
                    }))
                    .children([Tab::new().label("半角标点"), Tab::new().label("全角标点")]),
            )
            .child(
                Button::new("punct-add")
                    .primary()
                    .label("+ 新增")
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.open_editor(None, window, cx);
                    })),
            );

        let rows: Vec<AnyElement> = self
            .rules()
            .iter()
            .cloned()
            .enumerate()
            .map(|(ix, r)| {
                let is_custom =
                    self.custom_keys.contains(&r.key) || self.dirty_keys.contains(&r.key);
                let key_click = r.key.clone();
                let key_del = r.key.clone();
                h_flex()
                    .id(SharedString::from(format!("punct-row-{ix}")))
                    .gap_3()
                    .items_center()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(border)
                    .cursor_pointer()
                    .hover(|s| s.bg(cx_theme.secondary))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.open_editor(Some(key_click.clone()), window, cx);
                    }))
                    .child(div().w(px(60.)).child(r.key.clone()))
                    .child(div().flex_1().child(rule_value(&r)))
                    .child(
                        div()
                            .w(px(50.))
                            .text_sm()
                            .text_color(muted)
                            .child(rule_kind(&r)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .px_2()
                            .py_0p5()
                            .rounded_md()
                            .when(is_custom, |el| {
                                el.bg(accent_bg).text_color(accent_fg).child("自定义")
                            })
                            .when(!is_custom, |el| {
                                el.bg(cx_theme.secondary)
                                    .text_color(cx_theme.secondary_foreground)
                                    .child("默认")
                            }),
                    )
                    .child(
                        Button::new(SharedString::from(format!("punct-del-{ix}")))
                            .ghost()
                            .label("删除")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.delete_rule(&key_del, cx);
                            })),
                    )
                    .into_any_element()
            })
            .collect();

        v_flex().flex_1().min_h_0().gap_3().child(header).child(
            div()
                .flex_1()
                .min_h_0()
                .rounded_md()
                .border_1()
                .border_color(border)
                .overflow_hidden()
                .child(
                    div()
                        .id("punct-list")
                        .h_full()
                        .overflow_scroll()
                        .child(div().min_w(px(700.)).children(rows)),
                ),
        )
    }
}
