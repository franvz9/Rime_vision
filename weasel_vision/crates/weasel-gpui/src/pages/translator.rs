//! 翻译器子面板（对应 TranslatorSettings.vue，SchemeProfile 内嵌）。
use gpui_kit::component::{
    button::Button, checkbox::Checkbox, h_flex, v_flex, ActiveTheme, WindowExt as _,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use rime_core::service::translator::{self, TranslatorPatch, TranslatorSettings};

/// 4 个开关的字段访问器（读 / 写 / dirty 标记名）。
const FIELDS: [(&str, &str, &str); 4] = [
    (
        "enable_encoder",
        "启用自动造词",
        "(enable_encoder) 输入时自动学习并造词",
    ),
    (
        "enable_sentence",
        "启用自动句子输入",
        "(enable_sentence) 自动对短语进行编码，形成句子",
    ),
    (
        "enable_user_dict",
        "启用用户词典",
        "(enable_user_dict) 允许使用用户词典",
    ),
    (
        "encode_commit_history",
        "自动编码上屏词语",
        "(encode_commit_history) 对已上屏的词语进行编码",
    ),
];

pub struct TranslatorPanel {
    scope: Option<String>,
    settings: TranslatorSettings,
    is_custom: bool,
    dirty: [bool; 4],
}

impl TranslatorPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut this = Self {
            scope: None,
            settings: TranslatorSettings {
                enable_encoder: true,
                enable_sentence: true,
                enable_user_dict: true,
                encode_commit_history: true,
            },
            is_custom: false,
            dirty: [false; 4],
        };
        this.reload(None, window, cx);
        this
    }

    /// 重新加载指定 scope 的翻译器配置（scope 变化或保存后调用）。
    pub fn reload(&mut self, scope: Option<String>, _: &mut Window, cx: &mut Context<Self>) {
        self.scope = scope.clone();
        match translator::get_translator_profile(scope) {
            Ok(profile) => {
                self.settings = profile.settings;
                self.is_custom = profile.is_custom;
            }
            Err(e) => {
                eprintln!("Warning: load translator profile failed: {}", e);
                self.settings = TranslatorSettings::default();
                self.is_custom = false;
            }
        }
        self.dirty = [false; 4];
        cx.notify();
    }

    fn get_field(&self, ix: usize) -> bool {
        match ix {
            0 => self.settings.enable_encoder,
            1 => self.settings.enable_sentence,
            2 => self.settings.enable_user_dict,
            _ => self.settings.encode_commit_history,
        }
    }

    fn set_field(&mut self, ix: usize, v: bool) {
        match ix {
            0 => self.settings.enable_encoder = v,
            1 => self.settings.enable_sentence = v,
            2 => self.settings.enable_user_dict = v,
            _ => self.settings.encode_commit_history = v,
        }
    }

    /// 保存：差异写入，仅传被改动的字段。
    pub fn save(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let mut patch = TranslatorPatch {
            enable_encoder: None,
            enable_sentence: None,
            enable_user_dict: None,
            encode_commit_history: None,
        };
        if self.dirty[0] {
            patch.enable_encoder = Some(self.settings.enable_encoder);
        }
        if self.dirty[1] {
            patch.enable_sentence = Some(self.settings.enable_sentence);
        }
        if self.dirty[2] {
            patch.enable_user_dict = Some(self.settings.enable_user_dict);
        }
        if self.dirty[3] {
            patch.encode_commit_history = Some(self.settings.encode_commit_history);
        }
        if let Err(e) = translator::save_translator(self.scope.clone(), patch) {
            window.push_notification(format!("保存翻译器设置失败：{}", e), cx);
            return;
        }
        let scope = self.scope.clone();
        self.reload(scope, window, cx);
        window.push_notification("翻译器设置已保存", cx);
    }

    /// 恢复默认（删除该 scope 的 translator 自定义键）。
    fn reset(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Err(e) = translator::reset_translator(self.scope.clone()) {
            window.push_notification(format!("恢复默认失败：{}", e), cx);
            return;
        }
        let scope = self.scope.clone();
        self.reload(scope, window, cx);
        window.push_notification("已恢复默认", cx);
    }
}

impl Render for TranslatorPanel {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cx_theme = cx.theme();
        let muted = cx_theme.muted_foreground;
        let border = cx_theme.border;

        let rows: Vec<AnyElement> = FIELDS
            .iter()
            .enumerate()
            .map(|(ix, (_key, title, sub))| {
                let title = *title;
                let sub = *sub;
                h_flex()
                    .gap_3()
                    .items_center()
                    .py_2()
                    .border_b_1()
                    .border_color(border)
                    .child(
                        Checkbox::new(SharedString::from(format!("translator-{ix}")))
                            .checked(self.get_field(ix))
                            .on_click(cx.listener(move |this, checked: &bool, _, cx| {
                                this.set_field(ix, *checked);
                                this.dirty[ix] = true;
                                cx.notify();
                            })),
                    )
                    .child(
                        v_flex()
                            .child(div().font_weight(FontWeight::MEDIUM).child(title))
                            .child(div().text_sm().text_color(muted).child(sub)),
                    )
                    .into_any_element()
            })
            .collect();

        v_flex().gap_3().child(div().children(rows)).child(
            h_flex()
                .gap_2()
                .items_center()
                .child(
                    div()
                        .text_xs()
                        .px_2()
                        .py_0p5()
                        .rounded_md()
                        .bg(cx_theme.secondary)
                        .text_color(cx_theme.secondary_foreground)
                        .when(self.is_custom || self.dirty.iter().any(|d| *d), |el| {
                            el.child("自定义")
                        })
                        .when(!(self.is_custom || self.dirty.iter().any(|d| *d)), |el| {
                            el.child("默认")
                        }),
                )
                .when(self.is_custom, |el| {
                    el.child(
                        Button::new("translator-reset")
                            .outline()
                            .label("恢复默认")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.reset(window, cx);
                            })),
                    )
                }),
        )
    }
}
