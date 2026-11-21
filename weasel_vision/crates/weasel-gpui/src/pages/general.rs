//! 通用设置页（对应 GeneralSettings.vue）。
//!
//! 两段设置：方案切换器（switcher）、中英文切换（ascii_composer）。
//! 每段独立保存，保存成功后节内显示「已保存」2 秒。

use gpui_kit::component::{
    ActiveTheme,
    scroll::ScrollableElement as _,
    button::{Button, ButtonVariants as _},
    checkbox::Checkbox,
    form::{Field, Form},
    input::{Input, InputEvent, InputState},
    select::{Select, SelectEvent, SelectState},
    IndexPath,
    switch::Switch,
    WindowExt as _,
    h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use rime_core::service::settings::{self, GeneralSettings};

use crate::widgets::{deploy_notice, hint, section_card};

/// Caps Lock 动作选项（value + 中文说明）。
const CAPS_OPTIONS: &[(&str, &str)] = &[
    ("commit_code", "commit_code (上屏原始编码)"),
    ("inline_ascii", "inline_ascii (临时英文模式)"),
    ("noop", "noop (无操作)"),
    ("clear", "clear (清空输入)"),
];

/// 左/右 Shift 动作选项。
const SHIFT_OPTIONS: &[(&str, &str)] = &[
    ("commit_code", "commit_code (上屏原始编码)"),
    ("inline_ascii", "inline_ascii (临时英文模式)"),
    ("noop", "noop (无操作)"),
];

/// 按键名美化映射（对应 Vue 版 keyDisplayMap）。
fn pretty_key(part: &str) -> &str {
    match part.to_ascii_lowercase().as_str() {
        "grave" => "` (反引号)",
        "minus" => "- (减号)",
        "equal" => "= (等号)",
        "bracketleft" => "[ (左方括号)",
        "bracketright" => "] (右方括号)",
        "backslash" => "\\ (反斜杠)",
        "semicolon" => "; (分号)",
        "apostrophe" => "' (单引号)",
        "comma" => ", (逗号)",
        "period" => ". (句号)",
        "slash" => "/ (斜杠)",
        _ => part,
    }
}

/// 将 "Control+grave,Control+Shift+grave" 美化为可读文本。
fn format_key_display(raw: &str) -> String {
    raw.split([',', ' '])
        .filter(|k| !k.trim().is_empty())
        .map(|key| key.split('+').map(pretty_key).collect::<Vec<_>>().join("+"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// 选项 label 形如 "commit_code (上屏原始编码)"，取首段还原参数值。
fn value_of(label: &str) -> String {
    label.split(' ').next().unwrap_or(label).to_string()
}

fn option_labels(opts: &[(&str, &str)]) -> Vec<String> {
    opts.iter().map(|(_, label)| label.to_string()).collect()
}

fn default_index(opts: &[(&str, &str)], value: &str) -> Option<IndexPath> {
    opts.iter()
        .position(|(v, _)| *v == value)
        .map(|row| IndexPath::default().row(row))
}

type StringSelect = Entity<SelectState<Vec<String>>>;

pub struct GeneralPage {
    // 受控字段（input Change 时写回）
    caption: String,
    hotkeys: String,
    fold_options: bool,
    abbreviate_options: bool,
    caps_lock_action: String,
    shift_left_action: String,
    shift_right_action: String,
    good_old_caps_lock: bool,

    caption_input: Entity<InputState>,
    hotkeys_input: Entity<InputState>,
    caps_select: StringSelect,
    shift_left_select: StringSelect,
    shift_right_select: StringSelect,
    show_saved_switcher: bool,
    show_saved_ascii: bool,
    _subscriptions: Vec<Subscription>,
}

impl GeneralPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let settings = settings::get_general_settings().unwrap_or_default();

        let caption_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("方案切换菜单标题")
                .default_value(settings.switcher_caption.clone())
        });
        let hotkeys_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("如 Control+grave")
                .default_value(settings.switcher_hotkeys.clone())
        });
        let caps_select: StringSelect = cx.new(|cx| {
            SelectState::new(
                option_labels(CAPS_OPTIONS),
                default_index(CAPS_OPTIONS, &settings.caps_lock_action),
                window,
                cx,
            )
        });
        let shift_left_select: StringSelect = cx.new(|cx| {
            SelectState::new(
                option_labels(SHIFT_OPTIONS),
                default_index(SHIFT_OPTIONS, &settings.shift_left_action),
                window,
                cx,
            )
        });
        let shift_right_select: StringSelect = cx.new(|cx| {
            SelectState::new(
                option_labels(SHIFT_OPTIONS),
                default_index(SHIFT_OPTIONS, &settings.shift_right_action),
                window,
                cx,
            )
        });

        let mut subscriptions = Vec::new();
        subscriptions.push(cx.subscribe_in(
            &caption_input,
            window,
            |this, input, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    this.caption = input.read(cx).value().to_string();
                    cx.notify();
                }
            },
        ));
        subscriptions.push(cx.subscribe_in(
            &hotkeys_input,
            window,
            |this, _, ev: &InputEvent, _, cx| {
                if let InputEvent::Change = ev {
                    this.hotkeys = this.hotkeys_input.read(cx).value().to_string();
                    cx.notify();
                }
            },
        ));
        subscriptions.push(cx.subscribe_in(
            &caps_select,
            window,
            |this, _, ev: &SelectEvent<_>, _, cx| {
                if let SelectEvent::Confirm(Some(label)) = ev {
                    this.caps_lock_action = value_of(label);
                    cx.notify();
                }
            },
        ));
        subscriptions.push(cx.subscribe_in(
            &shift_left_select,
            window,
            |this, _, ev: &SelectEvent<_>, _, cx| {
                if let SelectEvent::Confirm(Some(label)) = ev {
                    this.shift_left_action = value_of(label);
                    cx.notify();
                }
            },
        ));
        subscriptions.push(cx.subscribe_in(
            &shift_right_select,
            window,
            |this, _, ev: &SelectEvent<_>, _, cx| {
                if let SelectEvent::Confirm(Some(label)) = ev {
                    this.shift_right_action = value_of(label);
                    cx.notify();
                }
            },
        ));

        Self {
            caption: settings.switcher_caption.clone(),
            hotkeys: settings.switcher_hotkeys.clone(),
            fold_options: settings.switcher_fold_options,
            abbreviate_options: settings.switcher_abbreviate_options,
            caps_lock_action: settings.caps_lock_action.clone(),
            shift_left_action: settings.shift_left_action.clone(),
            shift_right_action: settings.shift_right_action.clone(),
            good_old_caps_lock: settings.good_old_caps_lock,
            caption_input,
            hotkeys_input,
            caps_select,
            shift_left_select,
            shift_right_select,
            show_saved_switcher: false,
            show_saved_ascii: false,
            _subscriptions: subscriptions,
        }
    }

    /// 组装当前编辑状态的完整设置。
    fn current_settings(&self) -> GeneralSettings {
        GeneralSettings {
            switcher_caption: self.caption.clone(),
            switcher_hotkeys: self.hotkeys.clone(),
            switcher_fold_options: self.fold_options,
            switcher_abbreviate_options: self.abbreviate_options,
            caps_lock_action: self.caps_lock_action.clone(),
            shift_left_action: self.shift_left_action.clone(),
            shift_right_action: self.shift_right_action.clone(),
            good_old_caps_lock: self.good_old_caps_lock,
        }
    }

    /// 保存 switcher 段并在节内显示「已保存」2 秒。
    fn save_switcher(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let result = settings::save_general_settings(self.current_settings());
        match result {
            Ok(()) => {
                self.show_saved_switcher = true;
                cx.notify();
                Self::clear_saved_after_delay(
                    window,
                    cx,
                    cx.weak_entity(),
                    |this, cx| {
                        this.show_saved_switcher = false;
                        cx.notify();
                    },
                );
            }
            Err(e) => window.push_notification(format!("保存失败：{}", e), cx),
        }
    }

    /// 保存 ascii_composer 段。
    fn save_ascii(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let result = settings::save_general_settings(self.current_settings());
        match result {
            Ok(()) => {
                self.show_saved_ascii = true;
                cx.notify();
                Self::clear_saved_after_delay(
                    window,
                    cx,
                    cx.weak_entity(),
                    |this, cx| {
                        this.show_saved_ascii = false;
                        cx.notify();
                    },
                );
            }
            Err(e) => window.push_notification(format!("保存失败：{}", e), cx),
        }
    }

    /// 2 秒后清除「已保存」标记（后台线程 sleep，不阻塞 UI）。
    fn clear_saved_after_delay(
        window: &mut Window,
        cx: &mut Context<Self>,
        this: WeakEntity<Self>,
        f: impl Fn(&mut Self, &mut Context<Self>) + Send + 'static,
    ) {
        cx.spawn_in(window, async move |_, cx| {
            cx.background_spawn(async move {
                std::thread::sleep(std::time::Duration::from_secs(2));
            })
            .await;
            this.update_in(cx, |this, _, cx| f(this, cx)).ok();
        })
        .detach();
    }
}

fn saved_badge(show: bool) -> AnyElement {
    div()
        .text_sm()
        .text_color(gpui_kit::rgb(0x22c55e))
        .when(show, |el| el.child("已保存"))
        .into_any_element()
}

impl Render for GeneralPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cx_theme = cx.theme();
        let _danger = cx_theme.danger;

        let switcher_body = Form::horizontal()
            .label_width(px(170.))
            .child(
                Field::new()
                    .label("切换标题")
                    .description("切出输入方案菜单时显示的标题文字")
                    .child(Input::new(&self.caption_input).w(px(320.))),
            )
            .child(
                Field::new()
                    .label("快捷键")
                    .description("多个快捷键用英文逗号分隔")
                    .child(
                        v_flex()
                            .gap_1()
                            .child(Input::new(&self.hotkeys_input).w(px(320.)))
                            .child(hint(cx, format!("按键显示：{}", format_key_display(&self.hotkeys)))),
                    ),
            )
            .child(
                Field::new()
                    .label("折叠选项")
                    .description("菜单中折叠选择列表选项")
                    .child(
                        Checkbox::new("fold")
                            .checked(self.fold_options)
                            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                this.fold_options = *checked;
                                cx.notify();
                            })),
                    ),
            )
            .child(
                Field::new()
                    .label("缩写选项")
                    .description("菜单中缩写长名称选项")
                    .child(
                        Checkbox::new("abbreviate")
                            .checked(self.abbreviate_options)
                            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                this.abbreviate_options = *checked;
                                cx.notify();
                            })),
                    ),
            );

        let ascii_body = Form::horizontal()
            .label_width(px(170.))
            .child(
                Field::new()
                    .label("经典 Caps Lock")
                    .description("Caps 键为小写锁定，与西文习惯一致")
                    .child(
                        Switch::new("good-caps")
                            .checked(self.good_old_caps_lock)
                            .on_click(cx.listener(|this, checked: &bool, _, cx| {
                                this.good_old_caps_lock = *checked;
                                cx.notify();
                            })),
                    ),
            )
            .child(
                Field::new()
                    .label("Caps Lock")
                    .child(Select::new(&self.caps_select).w(px(320.))),
            )
            .child(
                Field::new()
                    .label("左 Shift")
                    .child(Select::new(&self.shift_left_select).w(px(320.))),
            )
            .child(
                Field::new()
                    .label("右 Shift")
                    .child(Select::new(&self.shift_right_select).w(px(320.))),
            );

        v_flex()
            .size_full()
            .overflow_y_scrollbar()
            .child(
                v_flex()
                    .max_w(px(720.))
                    .gap_4()
                    .child(deploy_notice(cx))
                    .child(section_card(
                        cx,
                        "方案切换器 (switcher)",
                        Some(
                            h_flex()
                                .gap_2()
                                .items_center()
                                .child(saved_badge(self.show_saved_switcher))
                                .child(
                                    Button::new("save-switcher")
                                        .primary()
                                        .label("保存")
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.save_switcher(window, cx);
                                        })),
                                )
                                .into_any_element(),
                        ),
                        switcher_body.into_any_element(),
                    ))
                    .child(section_card(
                        cx,
                        "中英文切换 (ascii_composer)",
                        Some(
                            h_flex()
                                .gap_2()
                                .items_center()
                                .child(saved_badge(self.show_saved_ascii))
                                .child(
                                    Button::new("save-ascii")
                                        .primary()
                                        .label("保存")
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.save_ascii(window, cx);
                                        })),
                                )
                                .into_any_element(),
                        ),
                        ascii_body.into_any_element(),
                    )),
            )
    }
}
