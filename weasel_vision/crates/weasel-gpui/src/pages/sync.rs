//! 同步管理页（对应 SyncManager.vue）。
//!
//! 状态卡 + 立即同步（后台线程执行）+ 修改设置模态（含目录选择对话框）+ 设备列表。
use gpui_kit::component::{
    ActiveTheme,
    scroll::ScrollableElement as _,
    button::{Button, ButtonVariants as _},
    Disableable as _,
    dialog::DialogButtonProps,
    form::{Field, Form},
    input::{Input, InputState},
    WindowExt as _, h_flex, v_flex,
};
use gpui_kit::assets::IconName;
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;
use rime_core::service::sync::{self, SyncResult, SyncSettings, SyncStatus, SyncedDevice};

pub struct SyncPage {
    settings: SyncSettings,
    status: SyncStatus,
    devices: Vec<SyncedDevice>,
    last_result: Option<SyncResult>,
    syncing: bool,
    /// 设置模态的输入（打开时创建）。
    dir_input: Option<Entity<InputState>>,
    id_input: Option<Entity<InputState>>,
}

impl SyncPage {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut this = Self {
            settings: SyncSettings {
                sync_dir: None,
                installation_id: String::new(),
                sync_user_dict: true,
                sync_config: true,
            },
            status: SyncStatus {
                configured: false,
                last_sync_time: None,
                sync_dir_exists: false,
                current_id: String::new(),
            },
            devices: Vec::new(),
            last_result: None,
            syncing: false,
            dir_input: None,
            id_input: None,
        };
        this.reload(window, cx);
        this
    }

    fn reload(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.settings = sync::get_sync_settings().unwrap_or_else(|_| self.settings.clone());
        self.status = sync::get_sync_status().unwrap_or_else(|_| self.status.clone());
        self.devices = sync::list_synced_devices().unwrap_or_default();
        cx.notify();
        let _ = window;
    }

    /// 立即同步：阻塞操作放后台线程，完成后展示结果并刷新。
    fn run_sync(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.syncing {
            return;
        }
        self.syncing = true;
        cx.notify();
        cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_spawn(async move { sync::execute_sync() })
                .await;
            this.update_in(cx, |this, window, cx| {
                this.syncing = false;
                match &result {
                    Ok(r) => {
                        this.last_result = Some(r.clone());
                        window.push_notification("同步完成", cx);
                    }
                    Err(e) => {
                        this.last_result = Some(SyncResult {
                            success: false,
                            uploaded: vec![],
                            downloaded: vec![],
                            errors: vec![e.to_string()],
                        });
                        window.push_notification(format!("同步失败：{}", e), cx);
                    }
                }
                this.reload(window, cx);
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn open_settings_dialog(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let dir_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("如 ~/Library/Mobile Documents/iCloud~Drive/RimeSync")
                .default_value(self.settings.sync_dir.clone().unwrap_or_default())
        });
        let id_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("设备标识，如 MyMac")
                .default_value(self.settings.installation_id.clone())
        });
        let weak = cx.weak_entity();
        self.dir_input = Some(dir_input.clone());
        self.id_input = Some(id_input.clone());
        let dir_input_c = dir_input.clone();
        let id_input_c = id_input.clone();
        let dir_input_ok = dir_input.clone();
        let id_input_ok = id_input.clone();

        window.open_dialog(cx, move |dialog, _, _| {
            let dir_input = dir_input_c.clone();
            let id_input = id_input_c.clone();
            dialog
                .title("同步设置")
                .child(
                    v_flex()
                        .gap_3()
                        .child(
                            div()
                                .p_3()
                                .rounded_md()
                                .text_sm()
                                .child("Rime 无自带云同步服务。将同步目录指向网盘（iCloud / WebDAV / 坚果云）目录即可在多台设备间同步；同步内容为 *.custom.yaml 与 *.userdb.txt 快照，不包含方案主配置与系统词典。"),
                        )
                        .child(
                            Form::horizontal()
                                .label_width(px(120.))
                                .child(
                                    Field::new().label("同步目录").child(
                                        h_flex()
                                            .gap_2()
                                            .items_center()
                                            .child(Input::new(&dir_input).flex_1())
                                            .child(
                                                Button::new("pick-sync-dir")
                                                    .outline()
                                                    .label("选择文件夹")
                                                    .on_click({
                                                        let dir_input = dir_input.clone();
                                                        move |_, window, cx| {
                                                            pick_sync_dir(&dir_input, window, cx);
                                                        }
                                                    }),
                                            ),
                                    ),
                                )
                                .child(
                                    Field::new().label("设备标识").child(
                                        Input::new(&id_input).w(px(240.)),
                                    ),
                                ),
                        ),
                )
                .button_props(
                    DialogButtonProps::default()
                        .ok_text("保存")
                        .on_ok({
                            let weak = weak.clone();
                            let dir_input = dir_input_ok.clone();
                            let id_input = id_input_ok.clone();
                            move |_, window, cx| {
                                let _ = weak.update(cx, |this, cx| {
                                    this.settings.sync_dir = {
                                        let v = dir_input.read(cx).value().trim().to_string();
                                        (!v.is_empty()).then_some(v)
                                    };
                                    this.settings.installation_id =
                                        id_input.read(cx).value().trim().to_string();
                                    // 落盘 + 刷新状态卡（S1：之前只更新内存，重启即丢失）
                                    match sync::save_sync_settings(this.settings.clone()) {
                                        Ok(()) => window.push_notification("同步设置已保存", cx),
                                        Err(e) => {
                                            window.push_notification(format!("保存同步设置失败：{}", e), cx)
                                        }
                                    }
                                    this.reload(window, cx);
                                });
                                true
                            }
                        }),
                )
                .overlay_closable(true)
        });
    }

    fn open_sync_dir(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(dir) = self.settings.sync_dir.clone() {
            if let Err(e) = rime_core::service::settings::open_dir(dir) {
                window.push_notification(format!("打开目录失败：{}", e), cx);
            }
        }
    }
}

/// 异步选目录：后台线程选择，结果经窗口回写输入框。
fn pick_sync_dir(dir_input: &Entity<InputState>, window: &mut Window, cx: &mut App) {
    let dir_input = dir_input.clone();
    let handle = window.window_handle();
    cx.spawn(async move |cx| {
        let picked = cx
            .background_spawn(async move {
                rfd::AsyncFileDialog::new().pick_folder().await
            })
            .await;
        let Some(file) = picked else { return };
        let path = file.path().display().to_string();
        handle
            .update(cx, |_, window, cx| {
                dir_input.update(cx, |s, cx| s.set_value(path.as_str(), window, cx));
            })
            .ok();
    })
    .detach();
}

impl Render for SyncPage {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let cx_theme = cx.theme();
        let muted = cx_theme.muted_foreground;
        let border = cx_theme.border;

        let ready = self.status.configured && self.status.sync_dir_exists;
        let last_sync = self
            .status
            .last_sync_time
            .clone()
            .unwrap_or_else(|| "从未同步".to_string());

        let result_card = self.last_result.clone().map(|r| {
            let title = if r.success { "同步完成" } else { "同步部分完成" };
            v_flex()
                .gap_2()
                .p_4()
                .rounded_lg()
                .border_1()
                .border_color(border)
                .child(div().font_weight(FontWeight::MEDIUM).child(title))
                .when(!r.uploaded.is_empty(), |el| {
                    el.child(div().text_sm().child(format!("上传：{}", r.uploaded.join("、"))))
                })
                .when(!r.downloaded.is_empty(), |el| {
                    el.child(div().text_sm().child(format!("下载：{}", r.downloaded.join("、"))))
                })
                .children(r.errors.iter().map(|e| {
                    div().text_sm().text_color(cx_theme.danger).child(e.clone())
                }))
                .into_any_element()
        });

        let device_rows: Vec<AnyElement> = self
            .devices
            .iter()
            .map(|d| {
                v_flex()
                    .gap_1()
                    .py_2()
                    .border_b_1()
                    .border_color(border)
                    .child(
                        h_flex()
                            .gap_2()
                            .child(div().font_weight(FontWeight::MEDIUM).child(d.id.clone()))
                            .child(div().text_sm().text_color(muted).child(d.last_sync.clone())),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(muted)
                            .child(format!("词典：{}", d.synced_dicts.join("、"))),
                    )
                    .into_any_element()
            })
            .collect();

        let status_row = |label: &'static str, value: AnyElement| {
            h_flex()
                .py_2()
                .border_b_1()
                .border_color(border)
                .justify_between()
                .child(div().text_sm().text_color(muted).child(label))
                .child(value)
        };

        v_flex()
            .size_full().overflow_y_scrollbar()
            .gap_4()
            .max_w(px(600.))
            .child(
                // 状态卡
                v_flex()
                    .p_4()
                    .rounded_lg()
                    .border_1()
                    .border_color(border)
                    .child(status_row(
                        "设备标识",
                        div().text_sm().child(self.status.current_id.clone()).into_any_element(),
                    ))
                    .child(status_row(
                        "同步目录",
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                div()
                                    .id("sync-dir")
                                    .text_sm()
                                    .cursor_pointer()
                                    .hover(|s| s.underline())
                                    .when_some(self.settings.sync_dir.clone(), |el, d| {
                                        el.text_color(cx_theme.primary).font_weight(FontWeight::MEDIUM).child(d)
                                    })
                                    .when(self.settings.sync_dir.is_none(), |el| {
                                        el.text_color(muted).child("未配置")
                                    }),
                            )
                            .when_some(self.settings.sync_dir.clone(), |el, _| {
                                el.child(
                                    Button::new("open-sync-dir")
                                        .ghost()
                                        .icon(IconName::FolderOpen)
                                        .label("打开")
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.open_sync_dir(window, cx);
                                        })),
                                )
                            })
                            .into_any_element(),
                    ))
                    .child(status_row(
                        "上次同步",
                        div().text_sm().child(last_sync).into_any_element(),
                    ))
                    .child(status_row(
                        "状态",
                        div()
                            .text_xs()
                            .px_2()
                            .py_0p5()
                            .rounded_md()
                            .when(ready, |el| {
                                el.bg(cx_theme.accent).text_color(cx_theme.accent_foreground).child("已配置")
                            })
                            .when(!ready, |el| {
                                el.bg(cx_theme.danger).text_color(cx_theme.danger_foreground).child("未就绪")
                            })
                            .into_any_element(),
                    ))
                    .child(
                        h_flex()
                            .gap_2()
                            .justify_end()
                            .pt_2()
                            .child(
                                Button::new("sync-edit")
                                    .outline()
                                    .label("修改设置")
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.open_settings_dialog(window, cx);
                                    })),
                            )
                            .child(
                                Button::new("sync-run")
                                    .primary()
                                    .disabled(self.syncing || !ready)
                                    .label(if self.syncing { "同步中…" } else { "立即同步" })
                                    .on_click(cx.listener(|this, _, window, cx| {
                                        this.run_sync(window, cx);
                                    })),
                            ),
                    ),
            )
            .children(result_card)
            .child(section_card_devices(cx, device_rows))
    }
}

fn section_card_devices(cx: &App, rows: Vec<AnyElement>) -> AnyElement {
    v_flex()
        .p_4()
        .rounded_lg()
        .border_1()
        .border_color(cx.theme().border)
        .child(
            div()
                .font_weight(FontWeight::MEDIUM)
                .mb_2()
                .child("已同步设备"),
        )
        .when(rows.is_empty(), |el| {
            el.child(div().text_sm().text_color(cx.theme().muted_foreground).child("暂无同步设备记录"))
        })
        .children(rows)
        .into_any_element()
}
