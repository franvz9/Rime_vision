//! 词典管理页（对应 DictManager.vue，功能最重的页面）。
//!
//! 左：用户词典列表；右：词条/快照双 Tab。
//! 词条：搜索 + 排序 + 分页表格 + 行内词频编辑 + 新增/批量删除低频/导出/清空。
//! 快照：快照卡片 + 应用修改（触发 Rime 重新部署）。
use gpui_kit::assets::IconName;
use gpui_kit::component::{
    button::{Button, ButtonVariants as _},
    dialog::DialogButtonProps,
    h_flex,
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement as _,
    select::{Select, SelectEvent, SelectState},
    v_flex, ActiveTheme, Disableable as _, IndexPath, WindowExt as _,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use rime_core::service::dict::{self, DictEntry, SnapshotInfo, UserDictInfo};

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

// 键名须与 rime-core service::dict::list_entries 的 sort_by 分支一致
const SORT_OPTIONS: &[(&str, &str)] = &[
    ("frequency_desc", "词频 ↓"),
    ("frequency_asc", "词频 ↑"),
    ("word_asc", "文字 ↑"),
    ("code_asc", "编码 ↑"),
];

pub struct DictPage {
    dicts: Vec<UserDictInfo>,
    selected: Option<String>,
    tab_entries: bool,
    // 词条状态
    entries: Vec<DictEntry>,
    total: usize,
    total_frequency: i64,
    page: usize,
    per_page: usize,
    sort_by: String,
    search: String,
    search_input: Entity<InputState>,
    sort_select: Entity<SelectState<Vec<String>>>,
    editing: Option<(String, String, Entity<InputState>)>, // (word, code, freq input)
    snapshots: Vec<SnapshotInfo>,
    _search_sub: Option<Subscription>,
}

impl DictPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search_input = cx.new(|cx| InputState::new(window, cx).placeholder("搜索编码或文字"));
        let sort_labels: Vec<String> = SORT_OPTIONS.iter().map(|(_, l)| l.to_string()).collect();
        let sort_select: Entity<SelectState<Vec<String>>> = cx
            .new(|cx| SelectState::new(sort_labels, Some(IndexPath::default().row(0)), window, cx));
        let mut this = Self {
            dicts: Vec::new(),
            selected: None,
            tab_entries: true,
            entries: Vec::new(),
            total: 0,
            total_frequency: 0,
            page: 0,
            per_page: 50,
            sort_by: "frequency_desc".to_string(),
            search: String::new(),
            search_input,
            sort_select: sort_select.clone(),
            editing: None,
            snapshots: Vec::new(),
            _search_sub: None,
        };
        let input = this.search_input.clone();
        this._search_sub =
            Some(
                cx.subscribe_in(&input, window, |this, input, ev: &InputEvent, _, cx| {
                    if let InputEvent::Change = ev {
                        this.search = input.read(cx).value().to_string();
                        this.page = 0;
                        let selected = this.selected.clone();
                        this.load_entries(selected, cx);
                    }
                }),
            );
        let sort_sub = cx.subscribe_in(
            &sort_select.clone(),
            window,
            |this, _, ev: &SelectEvent<_>, _, cx| {
                if let SelectEvent::Confirm(Some(label)) = ev {
                    let key = SORT_OPTIONS
                        .iter()
                        .find(|(_, l)| l == label)
                        .map(|(k, _)| k.to_string())
                        .unwrap_or_else(|| "frequency_desc".to_string());
                    this.set_sort(key, cx);
                }
            },
        );
        this._search_sub = Some(sort_sub);
        this.reload_dicts(cx);
        this
    }

    fn reload_dicts(&mut self, cx: &mut Context<Self>) {
        self.dicts = dict::list_user_dictionaries().unwrap_or_default();
        cx.notify();
    }

    fn load_entries(&mut self, dict_id: Option<String>, cx: &mut Context<Self>) {
        let Some(dict_id) = dict_id else { return };
        match dict::load_user_dict_entries(
            dict_id,
            self.page,
            self.per_page,
            self.sort_by.clone(),
            (!self.search.is_empty()).then_some(self.search.clone()),
        ) {
            Ok(r) => {
                self.entries = r.entries;
                self.total = r.total;
                self.total_frequency = r.total_frequency;
            }
            Err(e) => eprintln!("Warning: load entries failed: {}", e),
        }
        cx.notify();
    }

    fn select_dict(&mut self, dict_id: String, cx: &mut Context<Self>) {
        self.selected = Some(dict_id.clone());
        self.page = 0;
        self.editing = None;
        self.load_entries(Some(dict_id.clone()), cx);
        self.snapshots = dict::list_snapshots(dict_id).unwrap_or_default();
        cx.notify();
    }

    fn goto_page(&mut self, delta: i64, cx: &mut Context<Self>) {
        let max_page = if self.total == 0 {
            0
        } else {
            self.total.div_ceil(self.per_page) - 1
        };
        let next = (self.page as i64 + delta).clamp(0, max_page as i64) as usize;
        if next != self.page {
            self.page = next;
            let selected = self.selected.clone();
            self.load_entries(selected, cx);
        }
    }

    fn set_sort(&mut self, key: String, cx: &mut Context<Self>) {
        self.sort_by = key;
        self.page = 0;
        let selected = self.selected.clone();
        self.load_entries(selected, cx);
    }

    /// 创建快照：阻塞轮询（最长 10s）放后台线程，完成后刷新。
    fn create_snapshot(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(dict_id) = self.selected.clone() else {
            return;
        };
        cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_spawn(async move { dict::create_snapshot(dict_id.clone()) })
                .await;
            this.update_in(cx, |this, window, cx| {
                match &result {
                    Ok(_) => window.push_notification("快照已生成", cx),
                    Err(e) => {
                        let msg = e.to_string();
                        if msg.contains("SYNC_NOT_CONFIGURED") {
                            window.push_notification(
                                "需要先在「同步管理」中配置同步目录才能生成快照",
                                cx,
                            );
                        } else {
                            window.push_notification(format!("生成快照失败：{}", msg), cx);
                        }
                    }
                }
                if let Some(id) = &this.selected {
                    this.snapshots = dict::list_snapshots(id.clone()).unwrap_or_default();
                }
                let selected = this.selected.clone();
                this.load_entries(selected, cx);
                this.reload_dicts(cx);
            })
            .ok();
        })
        .detach();
    }

    fn open_add_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(dict_id) = self.selected.clone() else {
            return;
        };
        let word_input = cx.new(|cx| InputState::new(window, cx).placeholder("文字"));
        let code_input = cx.new(|cx| InputState::new(window, cx).placeholder("编码"));
        let freq_input = cx.new(|cx| InputState::new(window, cx).placeholder("初始词频（≥1）"));
        let weak = cx.weak_entity();
        let word_input2 = word_input.clone();
        let code_input2 = code_input.clone();
        let freq_input2 = freq_input.clone();
        let dict_id2 = dict_id.clone();
        window.open_dialog(cx, move |dialog, _, _| {
            let weak = weak.clone();
            let word_input = word_input2.clone();
            let code_input = code_input2.clone();
            let freq_input = freq_input2.clone();
            let dict_id = dict_id2.clone();
            dialog
                .title("新增词条")
                .child(
                    v_flex()
                        .gap_2()
                        .child(Input::new(&word_input))
                        .child(Input::new(&code_input))
                        .child(Input::new(&freq_input)),
                )
                .button_props(DialogButtonProps::default().ok_text("添加").on_ok({
                    let weak = weak.clone();
                    let word_input = word_input.clone();
                    let code_input = code_input.clone();
                    let freq_input = freq_input.clone();
                    let dict_id = dict_id.clone();
                    move |_, window, cx| {
                        let word = word_input.read(cx).value().trim().to_string();
                        let code = code_input.read(cx).value().trim().to_string();
                        let freq = freq_input
                            .read(cx)
                            .value()
                            .trim()
                            .parse::<i64>()
                            .unwrap_or(1)
                            .max(1);
                        let _ = weak.update(cx, |this, cx| {
                            if word.is_empty() || code.is_empty() {
                                return;
                            }
                            if dict::add_dict_entry(
                                dict_id.clone(),
                                rime_core::service::dict::NewDictEntry {
                                    word,
                                    code,
                                    frequency: freq,
                                },
                            )
                            .is_ok()
                            {
                                let selected = this.selected.clone();
                                this.load_entries(selected, cx);
                                this.reload_dicts(cx);
                                window.push_notification("词条已添加", cx);
                            } else {
                                window.push_notification("添加词条失败", cx);
                            }
                        });
                        true
                    }
                }))
                .overlay_closable(true)
        });
    }

    fn open_threshold_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(dict_id) = self.selected.clone() else {
            return;
        };
        let threshold_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("词频阈值（≥1）"));
        let weak = cx.weak_entity();
        let threshold_input2 = threshold_input.clone();
        let dict_id2 = dict_id.clone();
        window.open_dialog(cx, move |dialog, _, _| {
            let weak = weak.clone();
            let threshold_input = threshold_input2.clone();
            let dict_id = dict_id2.clone();
            dialog
                .title("批量删除低频词条")
                .child(
                    v_flex().gap_2().child(
                        v_flex()
                            .gap_1()
                            .child(div().text_sm().child("删除词频低于阈值的词条（不可撤销）"))
                            .child(Input::new(&threshold_input)),
                    ),
                )
                .button_props(DialogButtonProps::default().ok_text("删除").on_ok({
                    let weak = weak.clone();
                    let threshold_input = threshold_input.clone();
                    let dict_id = dict_id.clone();
                    move |_, window, cx| {
                        let threshold = threshold_input
                            .read(cx)
                            .value()
                            .trim()
                            .parse::<i64>()
                            .unwrap_or(0)
                            .max(1);
                        let _ =
                            weak.update(cx, |this, cx| {
                                match dict::batch_delete_low_frequency(dict_id.clone(), threshold) {
                                    Ok(n) => {
                                        window.push_notification(
                                            format!("已删除 {} 条低频词条", n),
                                            cx,
                                        );
                                        let selected = this.selected.clone();
                                        this.load_entries(selected, cx);
                                        this.reload_dicts(cx);
                                    }
                                    Err(e) => {
                                        window.push_notification(format!("批量删除失败：{}", e), cx)
                                    }
                                }
                            });
                        true
                    }
                }))
                .overlay_closable(true)
        });
    }

    fn open_clear_confirm(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(dict_id) = self.selected.clone() else {
            return;
        };
        let weak = cx.weak_entity();
        let dict_id2 = dict_id.clone();
        window.open_alert_dialog(cx, move |alert, _, _| {
            let weak = weak.clone();
            let dict_id = dict_id2.clone();
            alert
                .confirm()
                .title("清空词典")
                .description(format!(
                    "将清空词典 {} 的全部词条（直接操作快照文件），此操作不可撤销。",
                    dict_id
                ))
                .ok_text("确认清空")
                .on_ok({
                    let weak = weak.clone();
                    let dict_id = dict_id.clone();
                    move |_, window, cx| {
                        let _ = weak.update(cx, |this, cx| {
                            match dict::clear_user_dict(dict_id.clone()) {
                                Ok(()) => {
                                    window.push_notification("词典已清空", cx);
                                    let selected = this.selected.clone();
                                    this.load_entries(selected, cx);
                                    this.reload_dicts(cx);
                                }
                                Err(e) => window.push_notification(format!("清空失败：{}", e), cx),
                            }
                        });
                        true
                    }
                })
        });
    }

    fn open_apply_confirm(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(dict_id) = self.selected.clone() else {
            return;
        };
        let Some(first) = self.snapshots.first().cloned() else {
            return;
        };
        let weak = cx.weak_entity();
        let file_name = first.file_name.clone();
        let dict_id2 = dict_id.clone();
        window.open_alert_dialog(cx, move |alert, _, _| {
            let weak = weak.clone();
            let dict_id = dict_id2.clone();
            let file_name = file_name.clone();
            alert
                .confirm()
                .title("确认应用修改")
                .description("将把已修改的快照写回用户词典，并触发 Rime 重新部署。")
                .ok_text("确认应用")
                .on_ok({
                    let weak = weak.clone();
                    let dict_id = dict_id.clone();
                    let file_name = file_name.clone();
                    move |_, window, cx| {
                        let _ = weak.update(cx, |this, cx| {
                            match dict::apply_modified_snapshot(dict_id.clone(), file_name.clone())
                            {
                                Ok(()) => {
                                    window.push_notification("修改已应用！Rime 正在重新部署…", cx);
                                    let selected = this.selected.clone();
                                    this.load_entries(selected, cx);
                                    this.reload_dicts(cx);
                                }
                                Err(e) => window.push_notification(format!("应用失败：{}", e), cx),
                            }
                        });
                        true
                    }
                })
        });
    }

    fn export_dict(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(dict_id) = self.selected.clone() else {
            return;
        };
        let default_name = format!("{}_export.txt", dict_id);
        cx.spawn_in(window, async move |this, cx| {
            let picked = cx
                .background_spawn(async move {
                    rfd::AsyncFileDialog::new()
                        .add_filter("文本文件 (.txt)", &["txt"])
                        .set_file_name(&default_name)
                        .save_file()
                        .await
                })
                .await;
            let Some(file) = picked else { return };
            let output = file.path().display().to_string();
            this.update_in(cx, |this, window, cx| {
                match dict::export_user_dict(dict_id.clone(), output.clone()) {
                    Ok(n) => {
                        window.push_notification(format!("已导出 {} 条词条到 {}", n, output), cx)
                    }
                    Err(e) => window.push_notification(format!("导出失败：{}", e), cx),
                }
                let selected = this.selected.clone();
                this.load_entries(selected, cx);
                this.reload_dicts(cx);
            })
            .ok();
        })
        .detach();
    }

    fn save_edit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some((word, code, freq_input)) = self.editing.take() else {
            return;
        };
        let new_freq = freq_input
            .read(cx)
            .value()
            .trim()
            .parse::<i64>()
            .unwrap_or(0);
        let Some(dict_id) = self.selected.clone() else {
            return;
        };
        if let Err(e) = dict::update_entry_frequency(dict_id, word.clone(), code.clone(), new_freq)
        {
            window.push_notification(format!("更新词频失败：{}", e), cx);
            return;
        }
        let selected = self.selected.clone();
        self.load_entries(selected, cx);
        window.push_notification("词频已更新", cx);
    }

    fn delete_entry(
        &mut self,
        word: String,
        code: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(dict_id) = self.selected.clone() else {
            return;
        };
        if let Err(e) = dict::delete_entries(
            dict_id,
            vec![rime_core::service::dict::DictEntryKey { word, code }],
        ) {
            window.push_notification(format!("删除词条失败：{}", e), cx);
        }
        let selected = self.selected.clone();
        self.load_entries(selected, cx);
        self.reload_dicts(cx);
        cx.notify();
    }
}

impl Render for DictPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cx_theme = cx.theme();
        let muted = cx_theme.muted_foreground;
        let border = cx_theme.border;

        let dict_rows: Vec<AnyElement> = self
            .dicts
            .iter()
            .cloned()
            .enumerate()
            .map(|(ix, d)| {
                let id = d.dict_id.clone();
                let selected = self.selected.as_deref() == Some(d.dict_id.as_str());
                let need_sync = d.entry_count < 0;
                h_flex()
                    .id(SharedString::from(format!("dict-{ix}")))
                    .gap_2()
                    .items_center()
                    .px_3()
                    .py_2()
                    .border_b_1()
                    .border_color(border)
                    .cursor_pointer()
                    .when(selected, |el| el.bg(cx_theme.secondary))
                    .hover(|s| s.bg(cx_theme.secondary))
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.select_dict(id.clone(), cx);
                    }))
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .child(div().text_sm().child(d.display_name.clone()))
                            .child(div().text_xs().text_color(muted).child(if need_sync {
                                "需同步生成快照".to_string()
                            } else {
                                format!("{} 词条 · {}", d.entry_count, format_size(d.file_size))
                            })),
                    )
                    .into_any_element()
            })
            .collect();

        h_flex()
            .size_full()
            .gap_4()
            .min_h_0()
            .child(
                v_flex()
                    .w(px(260.))
                    .flex_none()
                    .h_full()
                    .rounded_lg()
                    .border_1()
                    .border_color(border)
                    .overflow_y_scrollbar()
                    .child(div().children(dict_rows))
                    .when(self.dicts.is_empty(), |el| {
                        el.child(
                            div()
                                .p_3()
                                .text_sm()
                                .text_color(muted)
                                .child("未发现用户词典"),
                        )
                    }),
            )
            .child(self.detail_area(cx))
    }
}

impl DictPage {
    fn detail_area(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let cx_theme = cx.theme();
        let muted = cx_theme.muted_foreground;
        let border = cx_theme.border;
        let _green = gpui_kit::rgb(0x22c55e);
        let warning_bg = cx_theme.warning;
        let warning_fg = cx_theme.warning_foreground;

        // Tab 条
        let tabs = h_flex()
            .gap_2()
            .child(
                Button::new("dict-tab-entries")
                    .when(self.tab_entries, |el| el.primary())
                    .when(!self.tab_entries, |el| el.ghost())
                    .label("词条")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.tab_entries = true;
                        cx.notify();
                    })),
            )
            .child(
                Button::new("dict-tab-snapshots")
                    .when(!self.tab_entries, |el| el.primary())
                    .when(self.tab_entries, |el| el.ghost())
                    .label("快照")
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.tab_entries = false;
                        cx.notify();
                    })),
            );

        let Some(dict_id) = self.selected.clone() else {
            return v_flex()
                .flex_1()
                .min_w_0()
                .gap_3()
                .child(tabs)
                .child(
                    div()
                        .text_sm()
                        .text_color(muted)
                        .child("从左侧选择一个用户词典"),
                )
                .into_any_element();
        };

        let max_page = if self.total == 0 {
            0
        } else {
            self.total.div_ceil(self.per_page) - 1
        };

        let body: AnyElement = if self.tab_entries {
            // 词条工具条（S2：此前构建后从未挂进渲染树，新增/导出/清空均不可达）
            let toolbar = h_flex()
                .flex_none()
                .gap_2()
                .items_center()
                .child(Input::new(&self.search_input).w(px(200.)))
                .child(Select::new(&self.sort_select).w(px(120.)))
                .child(
                    Button::new("dict-add")
                        .primary()
                        .label("+ 新增词条")
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.open_add_dialog(window, cx);
                        })),
                )
                .child(
                    Button::new("dict-lowfreq")
                        .outline()
                        .label("批量删低频")
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.open_threshold_dialog(window, cx);
                        })),
                )
                .child(
                    Button::new("dict-export")
                        .outline()
                        .label("导出码表")
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.export_dict(window, cx);
                        })),
                )
                .child(
                    Button::new("dict-clear")
                        .danger()
                        .label("清空词典")
                        .on_click(cx.listener(|this, _, window, cx| {
                            this.open_clear_confirm(window, cx);
                        })),
                );

            // 词条表头
            let header = h_flex()
                .px_3()
                .py_2()
                .border_b_1()
                .border_color(border)
                .text_sm()
                .text_color(muted)
                .child(div().w(px(40.)).child("#"))
                .child(div().w(px(160.)).child("文字"))
                .child(div().w(px(200.)).child("编码"))
                .child(div().w(px(120.)).child("词频"))
                .child(div().w(px(150.)).child("操作"));

            let rows: Vec<AnyElement> = self
                .entries
                .iter()
                .cloned()
                .enumerate()
                .map(|(ix, e)| {
                    let word = e.word.clone();
                    let code = e.code.clone();
                    let row_key = (e.word.clone(), e.code.clone());
                    let editing_word = self
                        .editing
                        .as_ref()
                        .map(|(w, c, _)| (w.clone(), c.clone()));
                    let is_editing = editing_word.as_ref() == Some(&row_key);
                    let freq_cell = if is_editing {
                        let freq_input = self
                            .editing
                            .as_ref()
                            .map(|(_, _, input)| input.clone())
                            .unwrap();
                        h_flex()
                            .gap_1()
                            .child(Input::new(&freq_input).w(px(80.)))
                            .child(
                                Button::new(SharedString::from(format!("edit-save-{ix}")))
                                    .primary()
                                    .label("✓")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.save_edit(window, cx);
                                    })),
                            )
                            .child(
                                Button::new(SharedString::from(format!("edit-cancel-{ix}")))
                                    .ghost()
                                    .label("✕")
                                    .on_click(cx.listener(|this, _, _, cx| {
                                        this.editing = None;
                                        cx.notify();
                                    })),
                            )
                            .into_any_element()
                    } else {
                        h_flex()
                            .gap_1()
                            .child(div().w(px(90.)).text_sm().child(e.frequency.to_string()))
                            .into_any_element()
                    };
                    h_flex()
                        .px_3()
                        .py_1p5()
                        .border_b_1()
                        .border_color(border)
                        .text_sm()
                        .child(
                            div()
                                .w(px(40.))
                                .text_color(muted)
                                .child((ix + 1).to_string()),
                        )
                        .child(div().w(px(160.)).child(e.word.clone()))
                        .child(div().w(px(200.)).text_color(muted).child(e.code.clone()))
                        .child(freq_cell)
                        .child(
                            h_flex()
                                .gap_1()
                                .child(
                                    Button::new(SharedString::from(format!("entry-edit-{ix}")))
                                        .ghost()
                                        .label("编辑")
                                        .on_click({
                                            let word = word.clone();
                                            let code = code.clone();
                                            let freq = e.frequency;
                                            cx.listener(move |this, _, window, cx| {
                                                let input = cx.new(|cx| {
                                                    InputState::new(window, cx)
                                                        .default_value(freq.to_string())
                                                });
                                                this.editing =
                                                    Some((word.clone(), code.clone(), input));
                                                cx.notify();
                                            })
                                        }),
                                )
                                .child(
                                    Button::new(SharedString::from(format!("entry-del-{ix}")))
                                        .ghost()
                                        .label("删除")
                                        .on_click(cx.listener(move |this, _, window, cx| {
                                            this.delete_entry(
                                                word.clone(),
                                                code.clone(),
                                                window,
                                                cx,
                                            );
                                        })),
                                ),
                        )
                        .into_any_element()
                })
                .collect();

            v_flex()
                .flex_1()
                .min_h_0()
                .gap_2()
                .child(toolbar)
                .child(
                    div()
                        .flex_1()
                        .min_h_0()
                        .rounded_lg()
                        .border_1()
                        .border_color(border)
                        .overflow_hidden()
                        .child(
                            div()
                                .h_full()
                                .overflow_y_scrollbar()
                                .child(header)
                                .child(div().children(rows))
                                .when(self.entries.is_empty(), |el| {
                                    el.child(
                                        div().p_3().text_sm().text_color(muted).child("无词条数据"),
                                    )
                                }),
                        ),
                )
                .child(
                    // 分页条
                    h_flex()
                        .justify_between()
                        .items_center()
                        .child(
                            h_flex()
                                .gap_2()
                                .child(
                                    Button::new("page-prev")
                                        .outline()
                                        .disabled(self.page == 0)
                                        .label("上一页")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.goto_page(-1, cx);
                                        })),
                                )
                                .child(div().text_sm().text_color(muted).child(format!(
                                    "第 {} / {} 页",
                                    self.page + 1,
                                    max_page + 1
                                )))
                                .child(
                                    Button::new("page-next")
                                        .outline()
                                        .disabled(self.page >= max_page)
                                        .label("下一页")
                                        .on_click(cx.listener(|this, _, _, cx| {
                                            this.goto_page(1, cx);
                                        })),
                                ),
                        )
                        .child(div().text_xs().text_color(muted).child(format!(
                            "共 {} 词条 · 总词频 {}",
                            self.total, self.total_frequency
                        ))),
                )
                .into_any_element()
        } else {
            // 快照 tab
            let snap = self.snapshots.first().cloned();
            v_flex()
                .gap_3()
                .child(
                    div()
                        .p_3()
                        .rounded_md()
                        .bg(warning_bg)
                        .text_color(warning_fg)
                        .text_sm()
                        .child("仅修改快照不会立即生效——点击「应用修改」写回用户词典后才会生效。"),
                )
                .child(match snap {
                    Some(s) => v_flex()
                        .gap_2()
                        .p_4()
                        .rounded_lg()
                        .border_1()
                        .border_color(border)
                        .child(div().font_weight(FontWeight::MEDIUM).child(s.file_name.clone()))
                        .child(
                            div()
                                .text_sm()
                                .text_color(muted)
                                .child(format!(
                                    "类型：{} · 创建：{} · 大小：{}",
                                    s.snapshot_type,
                                    s.created_at,
                                    format_size(s.size)
                                )),
                        )
                        .child(
                            Button::new("dict-apply")
                                .primary()
                                .icon(IconName::Check)
                                .label("应用修改")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.open_apply_confirm(window, cx);
                                })),
                        )
                        .into_any_element(),
                    None => v_flex()
                        .gap_2()
                        .child(
                            div().text_sm().text_color(muted).child(format!(
                                "词典 {} 暂无快照。需要先在「同步管理」配置同步目录，然后触发同步生成。",
                                dict_id
                            )),
                        )
                        .child(
                            Button::new("dict-snapshot")
                                .primary()
                                .icon(IconName::RefreshCw)
                                .label("触发同步并生成快照")
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.create_snapshot(window, cx);
                                })),
                        )
                        .into_any_element(),
                })
                .into_any_element()
        };

        v_flex()
            .flex_1()
            .min_w_0()
            .h_full()
            .min_h_0()
            .gap_3()
            .overflow_hidden()
            .child(tabs)
            .child(body)
            .into_any_element()
    }
}
