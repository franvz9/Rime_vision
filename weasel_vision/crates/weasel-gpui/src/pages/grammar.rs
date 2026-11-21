//! 语言模型页（对应 GrammarModel.vue）。
//!
//! 左：可用模型列表；右：模型详情 + 各方案挂载状态。
//! 批量挂载模态（方案多选 + 搭配/惩罚参数）；导入模型；待删除标记（随部署删除）。
use std::collections::HashMap;

use gpui_kit::component::{
    ActiveTheme,
    button::{Button, ButtonVariants as _},
    Disableable as _,
    scroll::ScrollableElement as _,
    input::InputState,
    checkbox::Checkbox,
    WindowExt as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::InteractiveElement as _;
use gpui_kit::StatefulInteractiveElement as _;
use gpui_kit::*;
use rime_core::service::grammar::{self};
use rime_core::rime::grammar::{SchemaGrammarConfig};
use rime_core::service::settings::PendingDelete;
use rime_core::service::schema;

pub struct GrammarPage {
    models: Vec<rime_core::rime::grammar::GrammarModel>,
    mount_configs: HashMap<String, SchemaGrammarConfig>,
    schema_ids: Vec<String>,
    selected: Option<usize>,
    /// 待删除的模型文件名（部署时统一删除）。
    pending: Option<String>,
    /// 批量挂载模态的方案勾选。
    mount_pick: Vec<bool>,
    /// 打开弹窗时的勾选快照（保存按钮 dirty 判定依据）。
    mount_pick_initial: Vec<bool>,
    mount_inputs: Option<MountInputs>,
}

/// 批量挂载模态的参数输入集合。
struct MountInputs {
    collocation_max_length: Entity<InputState>,
    collocation_min_length: Entity<InputState>,
    collocation_penalty: Entity<InputState>,
    non_collocation_penalty: Entity<InputState>,
    weak_collocation_penalty: Entity<InputState>,
    rear_penalty: Entity<InputState>,
    contextual_suggestions: bool,
    max_homophones: Entity<InputState>,
    max_homographs: Entity<InputState>,
    _subs: Vec<Subscription>,
}

impl GrammarPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut this = Self {
            models: Vec::new(),
            mount_configs: HashMap::new(),
            schema_ids: Vec::new(),
            selected: None,
            pending: None,
            mount_pick: Vec::new(),
            mount_pick_initial: Vec::new(),
            mount_inputs: None,
        };
        this.reload(window, cx);
        this
    }

    fn reload(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        let schema_ids: Vec<String> = schema::get_schemas()
            .map(|d| d.schemas.iter().map(|s| s.schema_id.clone()).collect())
            .unwrap_or_default();
        match grammar::get_grammar_data(schema_ids.clone()) {
            Ok(data) => {
                self.models = data.models;
                self.mount_configs = data.mount_configs;
            }
            Err(e) => eprintln!("Warning: load grammar data failed: {}", e),
        }
        self.schema_ids = schema_ids;
        if self.selected.map(|ix| ix >= self.models.len()).unwrap_or(true) {
            self.selected = if self.models.is_empty() { None } else { Some(0) };
        }
        cx.notify();
    }

    pub fn pending_deletes(&self) -> Vec<PendingDelete> {
        self.pending
            .iter()
            .map(|f| PendingDelete {
                delete_type: "model".to_string(),
                identifier: f.clone(),
            })
            .collect()
    }

    pub fn on_deploy_complete(&mut self, cx: &mut Context<Self>) {
        self.pending = None;
        cx.notify();
    }

    fn import_model(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        cx.spawn_in(window, async move |this, cx| {
            let picked = cx
                .background_spawn(async move {
                    rfd::AsyncFileDialog::new()
                        .add_filter("语言模型 (.gram)", &["gram"])
                        .set_title("选择语言模型文件")
                        .pick_file()
                        .await
                })
                .await;
            let Some(file) = picked else { return };
            let path = file.path().display().to_string();
            this.update_in(cx, |this, window, cx| {
                if let Err(e) = grammar::import_grammar(path) {
                    window.push_notification(format!("导入模型失败：{}", e), cx);
                    return;
                }
                this.reload(window, cx);
                window.push_notification("模型已导入", cx);
            })
            .ok();
        })
        .detach();
    }

    fn toggle_pending(&mut self, cx: &mut Context<Self>) {
        let Some(ix) = self.selected else { return };
        let Some(model) = self.models.get(ix) else { return };
        let mounted = self
            .schema_ids
            .iter()
            .any(|sid| self.mount_configs.get(sid).map(|c| c.mounted_model.as_deref() == Some(model.filename.as_str())).unwrap_or(false));
        if mounted {
            return; // 已挂载模型不可删除（对齐 Vue 版禁用行为）
        }
        match &self.pending {
            Some(p) if *p == model.filename => self.pending = None,
            _ => self.pending = Some(model.filename.clone()),
        }
        cx.notify();
    }

    fn open_mount_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.selected.is_none() {
            return;
        }
        // 默认参数（对齐 Vue 版默认值）
        let mk = |v: i64, window: &mut Window, cx: &mut Context<Self>| {
            cx.new(|cx| InputState::new(window, cx).default_value(v.to_string()))
        };
        let inputs = MountInputs {
            collocation_max_length: mk(5, window, cx),
            collocation_min_length: mk(2, window, cx),
            collocation_penalty: mk(-16, window, cx),
            non_collocation_penalty: mk(-8, window, cx),
            weak_collocation_penalty: mk(-100, window, cx),
            rear_penalty: mk(-20, window, cx),
            contextual_suggestions: true,
            max_homophones: mk(7, window, cx),
            max_homographs: mk(7, window, cx),
            _subs: Vec::new(),
        };
        // 勾选状态 = 期望状态：已挂载的方案默认勾选
        self.mount_pick = self
            .schema_ids
            .iter()
            .map(|sid| {
                self.selected
                    .and_then(|ix| self.models.get(ix))
                    .map(|m| {
                        self.mount_configs
                            .get(sid)
                            .map(|c| c.mounted_model.as_deref() == Some(m.filename.as_str()))
                            .unwrap_or(false)
                    })
                    .unwrap_or(false)
            })
            .collect();
        self.mount_pick_initial = self.mount_pick.clone();
        self.mount_inputs = Some(inputs);

        let weak = cx.weak_entity();
        let schema_ids = self.schema_ids.clone();

        window.open_dialog(cx, move |dialog, _, cx| {
            let weak = weak.clone();
            let schema_ids = schema_ids.clone();
            // 每次重渲染实时读取：当前勾选、打开时快照、dirty 状态
            let (pick, dirty) = weak
                .upgrade()
                .map(|e| {
                    let pick = e.read(cx).mount_pick.clone();
                    let dirty = pick != e.read(cx).mount_pick_initial;
                    (pick, dirty)
                })
                .unwrap_or((Vec::new(), false));
            dialog
                .w(px(560.))
                .title("批量挂载")
                .child(
                    v_flex()
                        .gap_3()
                        .child(
                            // 内容超高时可滚动
                            div()
                                .id("mount-dialog-scroll")
                                .max_h(px(380.))
                                .overflow_y_scroll()
                                .child(
                                    v_flex().gap_2().children(schema_ids.iter().enumerate().map(
                                        |(i, sid)| {
                                            let weak = weak.clone();
                                            let i = i;
                                            Checkbox::new(SharedString::from(format!("mount-{i}")))
                                                .checked(*pick.get(i).unwrap_or(&false))
                                                .label(sid.clone())
                                                .on_click(move |checked: &bool, _, cx| {
                                                    weak.update(cx, |this, _| {
                                                        if let Some(slot) = this.mount_pick.get_mut(i)
                                                        {
                                                            *slot = *checked;
                                                        }
                                                    })
                                                    .ok();
                                                    cx.refresh_windows();
                                                })
                                        },
                                    )),
                                ),
                        )
                        .child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child("勾选 = 挂载；取消勾选 = 卸载。保存后需重新部署。"),
                        )
                        .child(
                            h_flex()
                                .justify_end()
                                .gap_2()
                                .child(
                                    Button::new("mount-cancel")
                                        .outline()
                                        .label("取消")
                                        .on_click(|_, window, cx| {
                                            window.close_dialog(cx);
                                        }),
                                )
                                .child(
                                    Button::new("mount-save")
                                        .primary()
                                        .disabled(!dirty)
                                        .label("保存")
                                        .on_click({
                                            let weak = weak.clone();
                                            move |_, window, cx| {
                                                let _ = weak.update(cx, |this, cx| {
                                                    this.sync_mounts(window, cx);
                                                });
                                                window.close_dialog(cx);
                                            }
                                        }),
                                ),
                        ),
                )
                .overlay_closable(true)
        });
    }

    /// 保存：按勾选差异同步挂载状态——勾选且未挂载 → 挂载；取消勾选但已挂载 → 卸载。
    fn sync_mounts(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(ix) = self.selected else { return };
        let Some(model) = self.models.get(ix) else { return };
        let Some(inputs) = self.mount_inputs.take() else { return };
        let read = |e: &Entity<InputState>| -> i64 {
            e.read(cx).value().trim().parse().unwrap_or(0)
        };
        let mut mounted = 0;
        let mut unmounted = 0;
        for (i, sid) in self.schema_ids.iter().enumerate() {
            let want = self.mount_pick.get(i).copied().unwrap_or(false);
            let have = self
                .mount_configs
                .get(sid)
                .map(|c| c.mounted_model.as_deref() == Some(model.filename.as_str()))
                .unwrap_or(false);
            if want && !have {
                let config = SchemaGrammarConfig {
                    schema_id: sid.clone(),
                    mounted_model: Some(model.filename.clone()),
                    collocation_max_length: read(&inputs.collocation_max_length),
                    collocation_min_length: read(&inputs.collocation_min_length),
                    collocation_penalty: read(&inputs.collocation_penalty),
                    non_collocation_penalty: read(&inputs.non_collocation_penalty),
                    weak_collocation_penalty: read(&inputs.weak_collocation_penalty),
                    rear_penalty: read(&inputs.rear_penalty),
                    contextual_suggestions: inputs.contextual_suggestions,
                    max_homophones: read(&inputs.max_homophones),
                    max_homographs: read(&inputs.max_homographs),
                };
                if grammar::mount_grammar(model.filename.clone(), sid.clone(), config).is_ok() {
                    mounted += 1;
                }
            } else if !want && have {
                if grammar::unmount_grammar(sid.clone()).is_ok() {
                    unmounted += 1;
                }
            }
        }
        self.reload(window, cx);
        window.push_notification(
            format!("已同步：挂载 {} 个方案，卸载 {} 个方案", mounted, unmounted),
            cx,
        );
    }
}

impl Render for GrammarPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cx_theme = cx.theme();
        let muted = cx_theme.muted_foreground;
        let border = cx_theme.border;

        let danger = cx_theme.danger;
        let model_rows: Vec<AnyElement> = self
            .models
            .iter()
            .enumerate()
            .map(|(ix, m)| {
                let mounted_count = self
                    .schema_ids
                    .iter()
                    .filter(|sid| {
                        self.mount_configs
                            .get(*sid)
                            .map(|c| c.mounted_model.as_deref() == Some(m.filename.as_str()))
                            .unwrap_or(false)
                    })
                    .count();
                let is_pending_del = self.pending.as_deref() == Some(m.filename.as_str());
                let selected = self.selected == Some(ix);
                h_flex()
                    .id(SharedString::from(format!("grammar-model-{ix}")))
                    .gap_2()
                    .items_center()
                    .px_3()
                    .py_2()
                    .rounded_md()
                    .cursor_pointer()
                    .when(selected, |el| {
                        el.bg(cx_theme.accent).text_color(cx_theme.accent_foreground)
                    })
                    .hover(|s| s.bg(cx_theme.secondary))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.selected = Some(ix);
                        cx.notify();
                    }))
                    .when(is_pending_del, |el| el.opacity(0.55))
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .child(div().child(m.display_name.clone()))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(muted)
                                    .child(format!(
                                        "{} · {} · {}",
                                        m.formatted_size,
                                        m.filename,
                                        if mounted_count > 0 {
                                            format!("{} 个方案已挂载", mounted_count)
                                        } else {
                                            "未挂载".to_string()
                                        }
                                    )),
                            ),
                    )
                    .when(is_pending_del, |el| {
                        el.child(
                            div()
                                .text_xs()
                                .px_2()
                                .py_0p5()
                                .rounded_md()
                                .bg(danger)
                                .text_color(cx_theme.danger_foreground)
                                .child("待删除"),
                        )
                    })
                    .into_any_element()
            })
            .collect();

        // 右侧详情
        let detail: AnyElement = match (self.selected, self.models.get(self.selected.unwrap_or(0))) {
            (Some(_), Some(m)) => {
                let mounted_rows: Vec<AnyElement> = self
                    .schema_ids
                    .iter()
                    .map(|sid| {
                        let mounted = self
                            .mount_configs
                            .get(sid)
                            .map(|c| c.mounted_model.as_deref() == Some(m.filename.as_str()))
                            .unwrap_or(false);
                        h_flex()
                            .justify_between()
                            .px_3()
                            .py_2()
                            .border_b_1()
                            .border_color(border)
                            .child(div().text_sm().child(sid.clone()))
                            .child(
                                div()
                                    .text_sm()
                                    .when(mounted, |el| el.text_color(gpui_kit::rgb(0x22c55e)).child("已挂载"))
                                    .when(!mounted, |el| el.text_color(muted).child("未挂载")),
                            )
                            .into_any_element()
                    })
                    .collect();
                let can_delete = self.pending.as_deref() != Some(m.filename.as_str());
                let mounted_anywhere = self
                    .schema_ids
                    .iter()
                    .any(|sid| {
                        self.mount_configs
                            .get(sid)
                            .map(|c| c.mounted_model.as_deref() == Some(m.filename.as_str()))
                            .unwrap_or(false)
                    });
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    .overflow_y_scrollbar()
                    .gap_3()
                    .child(div().font_weight(FontWeight::MEDIUM).child(m.display_name.clone()))
                    .child(div().text_sm().text_color(muted).child(format!("文件：{}（{}）", m.filename, m.formatted_size)))
                    .child(div().text_sm().text_color(muted).child(format!("路径：{}", m.file_path)))
                    .child(
                        v_flex()
                            .rounded_md()
                            .border_1()
                            .border_color(border)
                            .child(div().p_2().text_sm().text_color(muted).child("挂载状态"))
                            .children(mounted_rows),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("grammar-mount")
                                    .primary()
                                    .label("批量挂载")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.open_mount_dialog(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("grammar-refresh")
                                    .outline()
                                    .label("刷新")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.reload(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("grammar-import")
                                    .outline()
                                                                        .label("导入模型")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.import_model(window, cx);
                                    })),
                            )
                            .child(
                                Button::new(SharedString::from(format!(
                                    "grammar-del-{}",
                                    self.selected.unwrap_or(0)
                                )))
                                .when(self.pending.is_some() && self.pending.as_deref() == Some(m.filename.as_str()), |el| {
                                    el.success().label("取消删除")
                                })
                                .when(!(self.pending.is_some() && self.pending.as_deref() == Some(m.filename.as_str())), |el| {
                                    el.danger().label("删除").disabled(mounted_anywhere)
                                })
                                .when(can_delete, |el| el)
                                .on_click(cx.listener(|this, _, _, cx| {
                                    this.toggle_pending(cx);
                                })),
                            ),
                    )
                    .into_any_element()
            }
            _ => div()
                .flex_1()
                .text_color(muted)
                .child("将 .gram 语言模型文件放入 Rime 用户目录后点击刷新")
                .into_any_element(),
        };

        v_flex()
            .size_full()
            .gap_4()
            .child(deploy_notice_widget(cx))
            .child(
                h_flex()
                    .gap_3()
                    .items_start()
                    .flex_1()
                    .min_h_0()
                    .child(
                        v_flex()
                            .w(px(280.))
                            .flex_none()
                            .h_full()
                            .min_h_0()
                            .gap_1()
                            .child(div().text_sm().text_color(muted).child("可用模型"))
                            .child(
                                div()
                                    .flex_1()
                                    .min_h_0()
                                    .overflow_y_scrollbar()
                                    .child(div().children(model_rows)),
                            )
                            .when(self.models.is_empty(), |el| {
                                el.child(div().text_sm().text_color(muted).child("暂无模型"))
                            }),
                    )
                    .child(detail),
            )
    }
}

use crate::widgets::deploy_notice as deploy_notice_widget;
