//! 备份管理页（对应 BackupManager.vue）。
//!
//! 左：备份列表；右：备份详情（文件多选恢复 + 对比 diff 双栏）。
//! 四个模态：创建备份（7 类别 + 全量 + 备注）、恢复、对比、确认删除。
use gpui_kit::assets::IconName;
use gpui_kit::component::{
    button::{Button, ButtonVariants as _},
    checkbox::Checkbox,
    dialog::DialogButtonProps,
    h_flex,
    input::{Input, InputState},
    scroll::ScrollableElement as _,
    v_flex, ActiveTheme, WindowExt as _,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use rime_core::service::backup::{self, BackupDetail, BackupInfo};
use std::collections::HashSet;

/// 备份类别（key, 标签, 说明）。
const CATEGORIES: &[(&str, &str, &str)] = &[
    (
        "core",
        "核心配置",
        "default.yaml, installation.yaml, user.yaml, squirrel.yaml 等",
    ),
    ("schemas", "方案定义", "*.schema.yaml 文件"),
    (
        "themes",
        "主题配色",
        "squirrel.custom.yaml, weasel.custom.yaml 等",
    ),
    ("dicts", "用户词典", "user_dictionaries/*.userdb.txt"),
    ("models", "语言模型", "*.gram 文件"),
    ("opencc", "OpenCC 数据", "opencc/ 目录"),
    ("lua", "Lua 脚本", "lua/ 目录"),
];

fn format_size(bytes: i64) -> String {
    let b = bytes as f64;
    if b >= 1024.0 * 1024.0 {
        format!("{:.1} MB", b / 1024.0 / 1024.0)
    } else if b >= 1024.0 {
        format!("{:.1} KB", b / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

fn type_name(t: &str) -> &str {
    match t {
        "manual" => "手动",
        "deploy" => "部署前",
        "auto" => "自动",
        other => other,
    }
}

pub struct BackupPage {
    backups: Vec<BackupInfo>,
    selected: Option<BackupDetail>,
    restore_files: HashSet<String>,
    deleting_id: Option<String>,
    note_input: Entity<InputState>,
    categories: Vec<bool>,
    full_mode: bool,
}

impl BackupPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let note_input = cx.new(|cx| InputState::new(window, cx).placeholder("可选备注"));
        let mut this = Self {
            backups: Vec::new(),
            selected: None,
            restore_files: HashSet::new(),
            deleting_id: None,
            note_input,
            categories: vec![true, true, true, true, false, false, false],
            full_mode: false,
        };
        this.load(window, cx);
        this
    }

    fn load(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.backups = backup::list_backups().unwrap_or_default();
        if let Some(sel) = &self.selected {
            let id = sel.info.id.clone();
            self.selected = backup::get_backup_detail(id).ok();
            if let Some(detail) = &self.selected {
                self.restore_files = detail.files.iter().map(|f| f.name.clone()).collect();
            }
        }
        cx.notify();
        let _ = window;
    }

    fn open_create_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let weak = cx.weak_entity();
        let note_input = self.note_input.clone();
        self.note_input
            .update(cx, |s, cx| s.set_value("", window, cx));
        window.open_dialog(cx, move |dialog, _, cx| {
            let weak = weak.clone();
            let note_input = note_input.clone();
            // 每次重渲染实时读取勾选状态（S3：捕获快照会导致勾选 UI 永不更新）
            let categories = weak
                .upgrade()
                .map(|e| e.read(cx).categories.clone())
                .unwrap_or_default();
            let full_mode = weak
                .upgrade()
                .map(|e| e.read(cx).full_mode)
                .unwrap_or(false);
            dialog
                .title("创建备份")
                .child(
                    v_flex()
                        .gap_3()
                        .child(
                            div()
                                .p_3()
                                .rounded_md()
                                .text_sm()
                                .text_color(gpui_kit::rgb(0xff6b6b))
                                .child("此备份由本工具执行（复制配置目录），不是 Rime 原生备份，也不会被同步。"),
                        )
                        .child(
                            v_flex()
                                .gap_2()
                                .children(CATEGORIES.iter().enumerate().map(|(i, (key, label, desc))| {
                                    let key = *key;
                                    let label = *label;
                                    let desc = *desc;
                                    let weak = weak.clone();
                                    h_flex()
                                        .gap_2()
                                        .items_start()
                                        .child(
                                            Checkbox::new(SharedString::from(format!("bk-cat-{key}")))
                                                .checked(*categories.get(i).unwrap_or(&false))
                                                .on_click({
                                                    let weak = weak.clone();
                                                    move |checked: &bool, _, cx| {
                                                        weak.update(cx, |this, _| {
                                                            if let Some(slot) = this.categories.get_mut(i) {
                                                                *slot = *checked;
                                                            }
                                                            this.full_mode = false;
                                                        }).ok();
                                                        cx.refresh_windows();
                                                    }
                                                }),
                                        )
                                        .child(
                                            v_flex()
                                                .child(div().text_sm().child(label))
                                                .child(div().text_xs().child(desc)),
                                        )
                                        .into_any_element()
                                })),
                        )
                        .child(
                            h_flex().gap_2().child(
                                Checkbox::new("bk-full")
                                    .checked(full_mode)
                                    .label("整个配置文件夹（全量）")
                                    .on_click({
                                        let weak = weak.clone();
                                        move |checked: &bool, _, cx| {
                                            weak.update(cx, |this, _| {
                                                this.full_mode = *checked;
                                                if *checked {
                                                    // 全量模式：勾选全部类别（UI 仅展示，提交时只传 ["full"]）
                                                    this.categories = vec![true; CATEGORIES.len()];
                                                } else {
                                                    this.categories = vec![true, true, true, true, false, false, false];
                                                }
                                            }).ok();
                                            cx.refresh_windows();
                                        }
                                    }),
                            ),
                        )
                        .child(
                            v_flex()
                                .gap_1()
                                .child(div().text_sm().child("备注"))
                                .child(Input::new(&note_input)),
                        ),
                )
                .button_props(
                    DialogButtonProps::default().ok_text("创建").on_ok({
                        let weak = weak.clone();
                        move |_, window, cx| {
                            let _ = weak.update(cx, |this, cx| {
                                this.submit_create(window, cx);
                            });
                            true
                        }
                    }),
                )
                .overlay_closable(true)
        });
    }

    fn submit_create(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let note = self.note_input.read(cx).value().to_string();
        let note = (!note.trim().is_empty()).then_some(note);
        let categories: Vec<String> = if self.full_mode {
            vec!["full".to_string()]
        } else {
            CATEGORIES
                .iter()
                .enumerate()
                .filter(|(i, _)| self.categories.get(*i).copied().unwrap_or(false))
                .map(|(_, (k, _, _))| k.to_string())
                .collect()
        };
        if categories.is_empty() {
            return;
        }
        match backup::create_backup(categories, note) {
            Ok(_) => {
                self.selected = None;
                self.backups = backup::list_backups().unwrap_or_default();
                window.push_notification("备份已创建", cx);
                cx.notify();
            }
            Err(e) => window.push_notification(format!("创建备份失败：{}", e), cx),
        }
    }

    fn open_restore_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(detail) = self.selected.clone() else {
            return;
        };
        let weak = cx.weak_entity();
        let files: Vec<String> = detail.files.iter().map(|f| f.name.clone()).collect();
        let backup_id = detail.info.id.clone();
        window.open_dialog(cx, move |dialog, _, cx| {
            let weak = weak.clone();
            let files = files.clone();
            // 每次重渲染实时读取勾选状态（S3：快照捕获导致勾选 UI 永不更新）
            let checked: Vec<bool> = weak
                .upgrade()
                .map(|e| {
                    let this = e.read(cx);
                    files
                        .iter()
                        .map(|f| this.restore_files.contains(f))
                        .collect()
                })
                .unwrap_or_default();
            dialog
                .title(format!("恢复备份 — {}", backup_id))
                .child(
                    v_flex()
                        .gap_2()
                        .child(div().text_sm().child(
                            "恢复前当前文件会被自动备份到 backups/auto/。请选择要恢复的文件：",
                        ))
                        .child(v_flex().gap_1().children(files.iter().enumerate().map(
                            |(i, f)| {
                                let weak = weak.clone();
                                let f = f.clone();
                                let checked = checked.get(i).copied().unwrap_or(false);
                                Checkbox::new(SharedString::from(format!("restore-{i}")))
                                    .checked(checked)
                                    .label(f.clone())
                                    .on_click(move |c: &bool, _, cx| {
                                        weak.update(cx, |this, _| {
                                            if *c {
                                                this.restore_files.insert(f.clone());
                                            } else {
                                                this.restore_files.remove(&f);
                                            }
                                        })
                                        .ok();
                                        cx.refresh_windows();
                                    })
                                    .into_any_element()
                            },
                        ))),
                )
                .button_props(DialogButtonProps::default().ok_text("确认恢复").on_ok({
                    let weak = weak.clone();
                    move |_, window, cx| {
                        let _ = weak.update(cx, |this, cx| {
                            this.do_restore(window, cx);
                        });
                        true
                    }
                }))
                .overlay_closable(true)
        });
    }

    fn do_restore(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(detail) = &self.selected else { return };
        let id = detail.info.id.clone();
        let files: Vec<String> = self.restore_files.iter().cloned().collect();
        if files.is_empty() {
            return;
        }
        match backup::restore_backup(id, files, true) {
            Ok(()) => {
                window.push_notification("备份已恢复", cx);
                self.load(window, cx);
            }
            Err(e) => window.push_notification(format!("恢复失败：{}", e), cx),
        }
    }

    fn open_compare(&mut self, file_name: String, window: &mut Window, cx: &mut Context<Self>) {
        let Some(detail) = &self.selected else { return };
        let id = detail.info.id.clone();
        let diff = match backup::compare_backup(id, file_name) {
            Ok(diff) => diff,
            Err(e) => {
                window.push_notification(format!("读取对比失败：{}", e), cx);
                return;
            }
        };
        let current = diff
            .current
            .clone()
            .unwrap_or_else(|| "（当前文件不存在）".to_string());
        let title = format!("对比：{}", diff.file_name);
        let theme = cx.theme();
        let border = theme.border;
        let accent_bg = theme.accent;
        let accent_fg = theme.accent_foreground;
        window.open_dialog(cx, move |dialog, _, _| {
            let current = current.clone();
            let title = title.clone();
            dialog
                .title(title)
                .child(
                    h_flex()
                        .gap_2()
                        .items_start()
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .gap_1()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .child("当前文件"),
                                )
                                .child(
                                    div()
                                        .p_2()
                                        .rounded_md()
                                        .border_1()
                                        .border_color(border)
                                        .text_xs()
                                        .max_h(px(400.))
                                        .child(current),
                                ),
                        )
                        .child(
                            v_flex()
                                .flex_1()
                                .min_w_0()
                                .gap_1()
                                .child(
                                    div()
                                        .text_sm()
                                        .font_weight(FontWeight::MEDIUM)
                                        .child("备份版本"),
                                )
                                .child(
                                    div()
                                        .p_2()
                                        .rounded_md()
                                        .bg(accent_bg)
                                        .text_color(accent_fg)
                                        .text_xs()
                                        .max_h(px(400.))
                                        .child(diff.backup.clone()),
                                ),
                        ),
                )
                .button_props(DialogButtonProps::default().ok_text("关闭"))
                .overlay_closable(true)
        });
    }

    fn open_delete_confirm(&mut self, id: String, window: &mut Window, cx: &mut Context<Self>) {
        let weak = cx.weak_entity();
        self.deleting_id = Some(id.clone());
        window.open_alert_dialog(cx, move |alert, _, _| {
            let weak = weak.clone();
            let id = id.clone();
            alert
                .confirm()
                .title("确认删除")
                .description(format!("将删除备份 {}，此操作不可撤销。", id))
                .ok_text("删除")
                .on_ok(move |_, _, cx| {
                    let _ = weak.update(cx, |this, cx| {
                        this.deleting_id = None;
                        if let Some(id) = Some(id.clone()) {
                            if backup::delete_backup(id).is_ok() {
                                this.selected = None;
                                this.backups = backup::list_backups().unwrap_or_default();
                            } else {
                                // 保留状态，提示由 alert 自动关闭后重试
                            }
                            cx.notify();
                        }
                    });
                    true
                })
        });
    }
}

impl Render for BackupPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cx_theme = cx.theme();
        let muted = cx_theme.muted_foreground;
        let border = cx_theme.border;

        let list_rows: Vec<AnyElement> = self
            .backups
            .iter()
            .cloned()
            .enumerate()
            .map(|(ix, b)| {
                let id = b.id.clone();
                let id_for_delete = b.id.clone();
                let selected = self
                    .selected
                    .as_ref()
                    .map(|d| d.info.id == b.id)
                    .unwrap_or(false);
                h_flex()
                    .id(SharedString::from(format!("backup-{ix}")))
                    .gap_2()
                    .items_center()
                    .justify_between()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(border)
                    .cursor_pointer()
                    .when(selected, |el| el.bg(cx_theme.secondary))
                    .hover(|s| s.bg(cx_theme.secondary))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        this.selected = backup::get_backup_detail(id.clone()).ok();
                        if let Some(d) = &this.selected {
                            this.restore_files = d.files.iter().map(|f| f.name.clone()).collect();
                        }
                        cx.notify();
                        let _ = window;
                    }))
                    .child(
                        v_flex()
                            .gap_0p5()
                            .child(div().text_sm().child(b.id.clone()))
                            .child(div().text_xs().text_color(muted).child(format!(
                                "{} · {} 文件 · {}",
                                type_name(&b.backup_type),
                                b.file_count,
                                format_size(b.total_size)
                            ))),
                    )
                    .child(
                        Button::new(SharedString::from(format!("bk-del-{ix}")))
                            .danger()
                            .label("删除")
                            .on_click(cx.listener(move |this, _, window, cx| {
                                this.open_delete_confirm(id_for_delete.clone(), window, cx);
                            })),
                    )
                    .into_any_element()
            })
            .collect();

        let detail: AnyElement = match &self.selected {
            Some(d) => {
                let info = &d.info;
                let file_rows: Vec<AnyElement> = d
                    .files
                    .iter()
                    .cloned()
                    .enumerate()
                    .map(|(i, f)| {
                        let name = f.name.clone();
                        h_flex()
                            .gap_2()
                            .items_center()
                            .py_1()
                            .border_b_1()
                            .border_color(border)
                            .child(div().flex_1().text_sm().child(f.name.clone()))
                            .child(div().text_xs().text_color(muted).child(format_size(f.size)))
                            .child(
                                Button::new(SharedString::from(format!("bk-cmp-{i}")))
                                    .ghost()
                                    .label("对比")
                                    .on_click(cx.listener(move |this, _, window, cx| {
                                        this.open_compare(name.clone(), window, cx);
                                    })),
                            )
                            .into_any_element()
                    })
                    .collect();
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .min_h_0()
                    .overflow_y_scrollbar()
                    .gap_3()
                    .child(
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(format!("备份 {}", info.id)),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .px_2()
                                    .py_0p5()
                                    .rounded_md()
                                    .bg(cx_theme.secondary)
                                    .text_color(cx_theme.secondary_foreground)
                                    .child(type_name(&info.backup_type).to_string()),
                            ),
                    )
                    .child(div().text_sm().text_color(muted).child(format!(
                                "时间：{} · 文件数：{} · 大小：{}{}",
                                info.created_at,
                                info.file_count,
                                format_size(info.total_size),
                                info.note
                                    .as_ref()
                                    .map(|n| format!(" · 备注：{}", n))
                                    .unwrap_or_default()
                            )))
                    .child(div().children(file_rows))
                    .child(
                        Button::new("bk-restore")
                            .primary()
                            .label("恢复此备份")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_restore_dialog(window, cx);
                            })),
                    )
                    .into_any_element()
            }
            None => div()
                .flex_1()
                .text_color(muted)
                .child("从左侧选择一个备份查看详情")
                .into_any_element(),
        };

        v_flex()
            .size_full()
            .gap_4()
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(div().font_weight(FontWeight::MEDIUM).child("备份列表"))
                    .child(
                        Button::new("bk-create")
                            .primary()
                            .icon(IconName::Plus)
                            .label("创建备份")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.open_create_dialog(window, cx);
                            })),
                    ),
            )
            .child(
                div()
                    .p_3()
                    .rounded_md()
                    .text_sm()
                    .child("系统会在每次部署前自动创建部署前备份，最多保留若干份。"),
            )
            .child(
                h_flex()
                    .gap_4()
                    .items_start()
                    .flex_1()
                    .min_h_0()
                    .child(
                        v_flex()
                            .w(px(320.))
                            .flex_none()
                            .h_full()
                            .min_h_0()
                            .rounded_lg()
                            .border_1()
                            .border_color(border)
                            .overflow_hidden()
                            .child(
                                div()
                                    .h_full()
                                    .overflow_y_scrollbar()
                                    .child(div().children(list_rows)),
                            )
                            .when(self.backups.is_empty(), |el| {
                                el.child(div().p_3().text_sm().text_color(muted).child("暂无备份"))
                            }),
                    )
                    .child(detail),
            )
    }
}
