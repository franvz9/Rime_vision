//! 方案列表页（对应 SchemaManager.vue）。
//!
//! 左：方案列表（启用开关 + 使用中/待删除 badge）；右：方案详情预览 + 切换按钮。
//! 保存配置（save_schemas）、导入方案（rfd）、待删除标记（随部署删除）。
use std::collections::HashSet;

use gpui_kit::component::{
    ActiveTheme,
    button::{Button, ButtonVariants as _},
    Disableable as _,
    scroll::ScrollableElement as _,
    switch::Switch,
    WindowExt as _, h_flex, v_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use rime_core::rime::schema::RimeSchema;
use rime_core::service::schema::{self, SchemaListData};
use rime_core::service::settings::PendingDelete;

pub struct SchemaPage {
    schemas: Vec<RimeSchema>,
    current: String,
    preview: usize,
    pending: HashSet<String>,
}

impl SchemaPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut this = Self {
            schemas: Vec::new(),
            current: String::new(),
            preview: 0,
            pending: HashSet::new(),
        };
        this.load();
        this.preview = this
            .schemas
            .iter()
            .position(|s| s.schema_id == this.current)
            .unwrap_or(0);
        cx.notify();
        let _ = window;
        this
    }

    fn load(&mut self) {
        let data: Result<SchemaListData, _> = schema::get_schemas();
        match data {
            Ok(d) => {
                self.schemas = d.schemas;
                self.current = d.current_schema;
            }
            Err(e) => eprintln!("Warning: load schemas failed: {}", e),
        }
    }

    pub fn pending_deletes(&self) -> Vec<PendingDelete> {
        self.pending
            .iter()
            .map(|f| PendingDelete {
                delete_type: "schema".to_string(),
                identifier: f.clone(),
            })
            .collect()
    }

    pub fn on_deploy_complete(&mut self, cx: &mut Context<Self>) {
        self.pending.clear();
        self.load();
        cx.notify();
    }

    fn switch_schema(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(schema_id) = self.schemas.get(self.preview).map(|s| s.schema_id.clone()) else {
            return;
        };
        if let Err(e) = schema::set_current_schema(schema_id.clone()) {
            window.push_notification(format!("切换方案失败：{}", e), cx);
            return;
        }
        self.load();
        window.push_notification(format!("已切换到 {}", schema_id), cx);
    }

    fn save_schemas(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Err(e) = schema::save_schemas(self.schemas.clone()) {
            window.push_notification(format!("保存方案配置失败：{}", e), cx);
            return;
        }
        self.load();
        window.push_notification("方案已保存", cx);
    }

    fn import(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        cx.spawn_in(window, async move |this, cx| {
            let picked = cx
                .background_spawn(async move {
                    rfd::AsyncFileDialog::new()
                        .add_filter("方案文件 (.schema.yaml)", &["yaml", "yml"])
                        .set_title("选择输入方案文件")
                        .pick_file()
                        .await
                })
                .await;
            let Some(file) = picked else { return };
            let path = file.path().display().to_string();
            this.update_in(cx, |this, window, cx| {
                if let Err(e) = schema::import_schema(path) {
                    window.push_notification(format!("导入方案失败：{}", e), cx);
                    return;
                }
                this.load();
                window.push_notification("方案已导入", cx);
            })
            .ok();
        })
        .detach();
    }

    fn toggle_pending(&mut self, schema_id: String, cx: &mut Context<Self>) {
        if self.pending.contains(&schema_id) {
            self.pending.remove(&schema_id);
        } else {
            self.pending.insert(schema_id);
        }
        cx.notify();
    }
}

impl Render for SchemaPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cx_theme = cx.theme();
        let muted = cx_theme.muted_foreground;
        let border = cx_theme.border;
        let green = gpui_kit::rgb(0x22c55e);
        let danger = cx_theme.danger;

        let rows: Vec<AnyElement> = self
            .schemas
            .iter()
            .cloned()
            .enumerate()
            .map(|(ix, s)| {
                let is_current = s.schema_id == self.current;
                let is_pending = self.pending.contains(&s.schema_id);
                let is_preview = self.preview == ix;
                let id = s.schema_id.clone();
                h_flex()
                    .id(SharedString::from(format!("schema-{ix}")))
                    .gap_3()
                    .items_center()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(border)
                    .cursor_pointer()
                    .when(is_preview, |el| el.bg(cx_theme.secondary))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.preview = ix;
                        cx.notify();
                    }))
                    .child(
                        Switch::new(SharedString::from(format!("schema-enable-{ix}")))
                            .checked(s.enabled)
                            .on_click(cx.listener(move |this, checked: &bool, _, cx| {
                                if let Some(slot) = this.schemas.get_mut(ix) {
                                    slot.enabled = *checked;
                                }
                                cx.notify();
                            })),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .child(div().child(s.name.clone()))
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(muted)
                                    .child(SharedString::from(s.schema_id.clone())),
                            ),
                    )
                    .when(is_current, |el| {
                        el.child(
                            div()
                                .text_xs()
                                .px_2()
                                .py_0p5()
                                .rounded_md()
                                .bg(green)
                                .text_color(gpui_kit::black())
                                .child("使用中"),
                        )
                    })
                    .when(is_pending, |el| {
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
                    .child(
                        Button::new(SharedString::from(format!("schema-del-{ix}")))
                            .when(is_pending, |el| el.success().label("取消删除"))
                            .when(!is_pending, |el| el.danger().label("删除"))
                            .disabled(is_current)
                            .on_click(cx.listener(move |this, _, _, cx| {
                                this.toggle_pending(id.clone(), cx);
                            })),
                    )
                    .into_any_element()
            })
            .collect();

        // 右侧预览
        let preview: AnyElement = match self.schemas.get(self.preview) {
            Some(s) => {
                let is_current = s.schema_id == self.current;
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .gap_2()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(s.name.clone()),
                            )
                            .when(is_current, |el| {
                                el.child(
                                    div()
                                        .text_xs()
                                        .px_2()
                                        .py_0p5()
                                        .rounded_md()
                                        .bg(green)
                                        .text_color(gpui_kit::black())
                                        .child("使用中"),
                                )
                            }),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(muted)
                            .child(format!("ID：{} · 版本：{}", s.schema_id, s.version)),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(muted)
                            .child(SharedString::from(format!("作者：{}", s.author))),
                    )
                    .child(div().text_sm().flex_wrap().child(s.description.clone()))
                    .when(!s.dependencies.is_empty(), |el| {
                        el.child(
                            h_flex()
                                .gap_1()
                                .flex_wrap()
                                .children(s.dependencies.iter().map(|d| {
                                    div()
                                        .text_xs()
                                        .px_2()
                                        .py_0p5()
                                        .rounded_md()
                                        .bg(cx_theme.secondary)
                                        .text_color(cx_theme.secondary_foreground)
                                        .child(d.clone())
                                })),
                        )
                    })
                    .child(
                        Button::new("schema-switch")
                            .primary()
                            .disabled(is_current)
                            .label(if is_current { "当前使用中" } else { "切换到此方案" })
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.switch_schema(window, cx);
                            })),
                    )
                    .into_any_element()
            }
            None => div()
                .flex_1()
                .text_color(muted)
                .child("暂无方案，点击「导入方案」添加")
                .into_any_element(),
        };

        v_flex()
            .size_full()
            .gap_4()
            .child(deploy_notice(cx))
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(div().font_weight(FontWeight::MEDIUM).child("输入方案列表"))
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new("schema-save")
                                    .primary()
                                    .label("保存配置")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.save_schemas(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("schema-import")
                                    .outline()
                                                                        .label("导入方案")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.import(window, cx);
                                    })),
                            ),
                    ),
            )
            .child(hint(cx, "点击方案查看详情；开关控制启用；删除标记将在下次「重新部署」时生效"))
            .child(
                h_flex()
                    .gap_4()
                    .items_start()
                    .flex_1()
                    .min_h_0()
                    .child(
                        v_flex()
                            .w(px(360.))
                            .flex_none()
                            .h_full()
                            .min_h_0()
                            .rounded_lg()
                            .border_1()
                            .border_color(border)
                            .overflow_hidden()
                            .child(
                                div().h_full().overflow_y_scrollbar().child(div().children(rows)),
                            )
                            .when(self.schemas.is_empty(), |el| {
                                el.child(
                                    div()
                                        .p_3()
                                        .text_sm()
                                        .text_color(muted)
                                        .child("暂无方案"),
                                )
                            }),
                    )
                    .child(preview),
            )
    }
}

use crate::widgets::{deploy_notice, hint};
