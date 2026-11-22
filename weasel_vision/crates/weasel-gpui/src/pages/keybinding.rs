//! 快捷键子面板（对应 KeybindingEditor.vue，SchemeProfile 内嵌）。
//!
//! 列表行 = 只读配置摘要；点击行弹窗编辑；「+ 新增」弹空白表单。
//! 弹窗内 5 字段：条件/按键/发送/切换/选择。
use gpui_kit::component::{
    button::{Button, ButtonVariants as _},
    dialog::DialogButtonProps,
    form::{Field, Form},
    h_flex,
    input::{Input, InputState},
    select::{Select, SelectState},
    v_flex, ActiveTheme, IndexPath, WindowExt as _,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use rime_core::service::keybinding::{self, KeyBinding};

/// when 条件选项。
const WHEN_OPTIONS: &[&str] = &["composing", "always", "has_menu", "paging"];
/// send 选项（空串 = 不发送）。
const SEND_OPTIONS: &[&str] = &[
    "",
    "Up",
    "Down",
    "Left",
    "Right",
    "Home",
    "End",
    "Page_Up",
    "Page_Down",
    "Return",
    "Escape",
    "BackSpace",
    "Delete",
    "Tab",
    "Space",
];
/// toggle 开关选项。
const TOGGLE_OPTIONS: &[&str] = &[
    "",
    "ascii_mode",
    "full_shape",
    "simplification",
    "extended_charset",
    "ascii_punct",
];
/// select 候选选项。
const SELECT_OPTIONS: &[&str] = &["", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0"];

type StringSelect = Entity<SelectState<Vec<String>>>;

fn labels_of(options: &[&str]) -> Vec<String> {
    options
        .iter()
        .map(|o| {
            if o.is_empty() {
                "(无)".to_string()
            } else {
                o.to_string()
            }
        })
        .collect()
}

fn make_select(options: &[&str], value: &str, window: &mut Window, cx: &mut App) -> StringSelect {
    let labels = labels_of(options);
    let default = options
        .iter()
        .position(|o| *o == value)
        .map(|row| IndexPath::default().row(row));
    cx.new(|cx| SelectState::new(labels, default, window, cx))
}

fn select_value(sel: &StringSelect, options: &[&str], cx: &App) -> String {
    let Some(ix) = sel.read(cx).selected_index(cx) else {
        return String::new();
    };
    options
        .get(ix.row)
        .map(|s| s.to_string())
        .unwrap_or_default()
}

/// 选中项的文字展示（空值显示「无」）。
fn display(v: &str) -> SharedString {
    if v.is_empty() {
        "(无)".into()
    } else {
        v.into()
    }
}

pub struct KeybindingPanel {
    scope: Option<String>,
    bindings: Vec<KeyBinding>,
}

impl KeybindingPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut this = Self {
            scope: None,
            bindings: Vec::new(),
        };
        this.reload(None, window, cx);
        this
    }

    pub fn reload(&mut self, scope: Option<String>, _: &mut Window, cx: &mut Context<Self>) {
        self.scope = scope.clone();
        self.bindings = keybinding::get_keybindings(scope).unwrap_or_default();
        cx.notify();
    }

    pub fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Err(e) = keybinding::save_keybindings(self.scope.clone(), self.bindings.clone()) {
            window.push_notification(format!("保存快捷键失败：{}", e), cx);
            return;
        }
        window.push_notification("快捷键已保存", cx);
    }

    fn add_row(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.open_binding_dialog(None, window, cx);
    }

    fn remove_row(&mut self, ix: usize, cx: &mut Context<Self>) {
        self.bindings.remove(ix);
        cx.notify();
    }

    /// 新增（ix=None）或编辑（ix=Some）绑定弹窗。
    fn open_binding_dialog(
        &mut self,
        ix: Option<usize>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let existing = ix.and_then(|i| self.bindings.get(i).cloned());
        let empty = KeyBinding {
            when: "composing".into(),
            accept: String::new(),
            send: String::new(),
            toggle: String::new(),
            select: String::new(),
        };
        let b = existing.clone().unwrap_or(empty);

        let when_select = make_select(WHEN_OPTIONS, &b.when, window, cx);
        let accept_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("如 Control+p")
                .default_value(b.accept.clone())
        });
        let send_select = make_select(SEND_OPTIONS, &b.send, window, cx);
        let toggle_select = make_select(TOGGLE_OPTIONS, &b.toggle, window, cx);
        let select_select = make_select(SELECT_OPTIONS, &b.select, window, cx);

        let weak = cx.weak_entity();
        let when2 = when_select.clone();
        let accept2 = accept_input.clone();
        let send2 = send_select.clone();
        let toggle2 = toggle_select.clone();
        let select2 = select_select.clone();
        let title = if ix.is_some() {
            "编辑快捷键"
        } else {
            "新增快捷键"
        };

        window.open_dialog(cx, move |dialog, _, _| {
            let weak = weak.clone();
            let (when_select, accept_input) = (when2.clone(), accept2.clone());
            let (send_select, toggle_select, select_select) =
                (send2.clone(), toggle2.clone(), select2.clone());
            let ix = ix;
            dialog
                .title(title)
                .child(
                    Form::horizontal()
                        .label_width(px(130.))
                        .child(
                            Field::new()
                                .label("条件 (when)")
                                .child(Select::new(&when_select).w(px(220.))),
                        )
                        .child(
                            Field::new()
                                .label("按键 (accept)")
                                .child(Input::new(&accept_input).w(px(220.))),
                        )
                        .child(
                            Field::new()
                                .label("发送 (send)")
                                .child(Select::new(&send_select).w(px(220.))),
                        )
                        .child(
                            Field::new()
                                .label("切换 (toggle)")
                                .child(Select::new(&toggle_select).w(px(220.))),
                        )
                        .child(
                            Field::new()
                                .label("选择 (select)")
                                .child(Select::new(&select_select).w(px(220.))),
                        ),
                )
                .button_props(DialogButtonProps::default().ok_text("保存").on_ok({
                    let weak = weak.clone();
                    let (when_select, accept_input) = (when_select.clone(), accept_input.clone());
                    let (send_select, toggle_select, select_select) = (
                        send_select.clone(),
                        toggle_select.clone(),
                        select_select.clone(),
                    );
                    move |_, _, cx| {
                        let _ = weak.update(cx, |this, cx| {
                            let binding = KeyBinding {
                                when: select_value(&when_select, WHEN_OPTIONS, cx),
                                accept: accept_input.read(cx).value().trim().to_string(),
                                send: select_value(&send_select, SEND_OPTIONS, cx),
                                toggle: select_value(&toggle_select, TOGGLE_OPTIONS, cx),
                                select: select_value(&select_select, SELECT_OPTIONS, cx),
                            };
                            match ix {
                                Some(i) => {
                                    if let Some(slot) = this.bindings.get_mut(i) {
                                        *slot = binding;
                                    }
                                }
                                None => this.bindings.push(binding),
                            }
                            cx.notify();
                        });
                        true
                    }
                }))
                .overlay_closable(true)
        });
    }
}

/// 行摘要：composing ｜ Control+p → Up ｜ toggle ｜ select N
fn summary(b: &KeyBinding) -> SharedString {
    let mut parts = vec![b.accept.clone()];
    if !b.send.is_empty() {
        parts.push(format!("→ {}", b.send));
    }
    if !b.toggle.is_empty() {
        parts.push(format!("切换 {}", b.toggle));
    }
    if !b.select.is_empty() {
        parts.push(format!("选 {}", b.select));
    }
    format!("{}｜{}", b.when, parts.join(" ")).into()
}

impl Render for KeybindingPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cx_theme = cx.theme();
        let muted = cx_theme.muted_foreground;
        let border = cx_theme.border;
        let danger = cx_theme.danger;

        let rows: Vec<AnyElement> = self
            .bindings
            .iter()
            .cloned()
            .enumerate()
            .map(|(ix, b)| {
                let row_id = SharedString::from(format!("kb-row-{ix}"));
                let del_ix = ix;
                h_flex()
                    .id(row_id)
                    .gap_3()
                    .items_center()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(border)
                    .cursor_pointer()
                    .hover(|s| s.bg(cx_theme.secondary))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.open_binding_dialog(Some(ix), window, cx);
                    }))
                    .child(
                        div()
                            .text_sm()
                            .font_weight(FontWeight::MEDIUM)
                            .child(display(&b.when)),
                    )
                    .child(div().flex_1().min_w_0().text_sm().child(summary(&b)))
                    .child(
                        Button::new(SharedString::from(format!("kb-del-{ix}")))
                            .danger()
                            .label("删除")
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.remove_row(del_ix, cx);
                            })),
                    )
                    .into_any_element()
            })
            .collect();

        h_flex()
            .flex_1()
            .min_h_0()
            .gap_4()
            .items_start()
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .min_h_0()
                    .gap_3()
                    .child(
                        // 外框：flex 分配高度，内容在框内滚动
                        div()
                            .flex_1()
                            .min_h_0()
                            .rounded_lg()
                            .border_1()
                            .border_color(border)
                            .overflow_hidden()
                            .child(
                                div()
                                    .id("kb-list")
                                    .h_full()
                                    .overflow_y_scroll()
                                    .child(div().children(rows)),
                            ),
                    )
                    .child(
                        Button::new("kb-add")
                            .primary()
                            .label("+ 新增")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.add_row(window, cx);
                            })),
                    ),
            )
            .child(
                v_flex()
                    .w(px(260.))
                    .flex_none()
                    .h_full()
                    .gap_2()
                    .p_3()
                    .rounded_md()
                    .bg(cx_theme.accent)
                    .text_color(cx_theme.accent_foreground)
                    .child(div().font_weight(FontWeight::MEDIUM).child("使用说明"))
                    .child(
                        div()
                            .text_sm()
                            .text_color(muted)
                            .child("每行一个按键绑定。点击行即可编辑；删除直接移除，最后统一「保存当前配置」写入。"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(muted)
                            .child("示例：composing + Control+n → Down，在候选菜单中用 Ctrl+N 下移候选。")
                            .when(false, |el| el.child(div().text_color(danger))),
                    ),
            )
    }
}
