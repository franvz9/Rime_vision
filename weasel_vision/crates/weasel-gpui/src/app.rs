//! 根视图：左侧导航 + 顶部工具栏（标题 / 待删除提示 / 部署按钮）+ 页面容器。
//!
//! 对应 Vue 版 App.vue 的布局与交互约定：
//! - 页面惰性创建并缓存（切页不丢未保存编辑）
//! - 「待删除」项由三个管理页（方案/模型/主题）汇聚到顶栏，点部署统一删除
//! - 部署在后台线程执行，成功后清空待删除并通知相关页刷新

use gpui_kit::component::{
    button::{Button, ButtonVariants as _},
    sidebar::{Sidebar, SidebarGroup, SidebarHeader, SidebarMenu, SidebarMenuItem},
    ActiveTheme, Disableable as _, IconName, WindowExt as _, v_flex, h_flex,
};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::canvas;
use gpui_kit::*;
use rime_core::service::settings::PendingDelete;

use crate::pages::{self, *};

pub struct WeaselApp {
    page: Page,
    pages: pages::PageCache,
    deploying: bool,
}

/// 侧边栏宽度（全局态：拖拽分隔条时鼠标监听在 App 层，需要可从任意上下文读写）。
#[derive(Default)]
struct SidebarResizeState {
    width: f32,
    resizing: bool,
    start_x: f32,
    start_w: f32,
}

impl Global for SidebarResizeState {}

/// 宽度约束：最小不挤压文字，最大为窗口宽度的 1/3。
const SIDEBAR_MIN_W: f32 = 170.0;
const SIDEBAR_DEFAULT_W: f32 = 200.0;

impl WeaselApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut this = Self {
            page: Page::General,
            pages: pages::PageCache::default(),
            deploying: false,
        };
        if !cx.has_global::<SidebarResizeState>() {
            cx.set_global(SidebarResizeState { width: SIDEBAR_DEFAULT_W, ..Default::default() });
        }
        this.ensure_page(Page::General, window, cx);
        this
    }

    /// 惰性创建页面实体（首次进入时）。
    fn ensure_page(&mut self, page: Page, window: &mut Window, cx: &mut Context<Self>) {
        let cache = &mut self.pages;
        macro_rules! ensure {
            ($field:ident, $ty:path) => {
                if cache.$field.is_none() {
                    cache.$field = Some(cx.new(|cx| <$ty>::new(window, cx)));
                }
            };
        }
        match page {
            Page::General => ensure!(general, general::GeneralPage),
            Page::Theme => ensure!(theme, theme_editor::ThemePage),
            Page::Grammar => ensure!(grammar, grammar::GrammarPage),
            Page::Backup => ensure!(backup, backup::BackupPage),
            Page::Dict => ensure!(dict, dict::DictPage),
            Page::Sync => ensure!(sync, sync::SyncPage),
            Page::Advanced => ensure!(advanced, advanced::AdvancedPage),
            Page::Schema => ensure!(schema, schema::SchemaPage),
            Page::Profile => ensure!(profile, profile::ProfilePage),
        }
    }

    fn current_page_view(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        self.ensure_page(self.page, window, cx);
        let cache = &mut self.pages;
        match self.page {
            Page::General => cache.general.as_ref().unwrap().clone().into_any_element(),
            Page::Theme => cache.theme.as_ref().unwrap().clone().into_any_element(),
            Page::Grammar => cache.grammar.as_ref().unwrap().clone().into_any_element(),
            Page::Backup => cache.backup.as_ref().unwrap().clone().into_any_element(),
            Page::Dict => cache.dict.as_ref().unwrap().clone().into_any_element(),
            Page::Sync => cache.sync.as_ref().unwrap().clone().into_any_element(),
            Page::Advanced => cache.advanced.as_ref().unwrap().clone().into_any_element(),
            Page::Schema => cache.schema.as_ref().unwrap().clone().into_any_element(),
            Page::Profile => cache.profile.as_ref().unwrap().clone().into_any_element(),
        }
    }

    fn pending_deletes(&self, cx: &App) -> Vec<PendingDelete> {
        let mut out = Vec::new();
        self.pages.collect_pending(cx, &mut out);
        out
    }

    /// 重新部署：后台线程执行（含 ≥2s 阻塞等待），完成后刷新相关页并通知。
    fn deploy(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.deploying {
            return;
        }
        let pending = self.pending_deletes(cx);
        self.deploying = true;
        cx.notify();

        cx.spawn_in(window, async move |this, cx| {
            let result = cx
                .background_spawn(async move {
                    rime_core::service::settings::deploy(Some(pending))
                })
                .await;
            this.update_in(cx, |this, window, cx| {
                this.deploying = false;
                match result {
                    Ok(()) => {
                        window.push_notification("部署成功", cx);
                        this.pages.on_deploy_complete(window, cx);
                    }
                    Err(e) => window.push_notification(format!("部署失败：{}", e), cx),
                }
                cx.notify();
            })
            .ok();
        })
        .detach();
    }

    fn nav_global_group(&mut self, cx: &mut Context<Self>) -> SidebarGroup<SidebarMenu> {
        let page = self.page;
        SidebarGroup::new("全局配置").child(SidebarMenu::new().children([
            nav_item(cx, Page::General, "通用设置", IconName::Settings, page),
            nav_item(cx, Page::Theme, "主题外观", IconName::Palette, page),
            nav_item(cx, Page::Grammar, "语言模型", IconName::Cpu, page),
            nav_item(cx, Page::Backup, "备份管理", IconName::HardDrive, page),
            nav_item(cx, Page::Dict, "词典管理", IconName::BookOpen, page),
            nav_item(cx, Page::Sync, "同步管理", IconName::RefreshCw, page),
            nav_item(cx, Page::Advanced, "高级设置", IconName::Settings2, page),
        ]))
    }

    fn nav_schema_group(&mut self, cx: &mut Context<Self>) -> SidebarGroup<SidebarMenu> {
        let page = self.page;
        SidebarGroup::new("方案配置").child(SidebarMenu::new().children([
            nav_item(cx, Page::Schema, "方案列表", IconName::FileText, page),
            nav_item(cx, Page::Profile, "方案配置", IconName::FolderOpen, page),
        ]))
    }
}

fn nav_item(
    cx: &mut Context<WeaselApp>,
    target: Page,
    label: &'static str,
    icon: IconName,
    active: Page,
) -> SidebarMenuItem {
    SidebarMenuItem::new(label)
        .icon(icon)
        .active(active == target)
        .on_click(cx.listener(move |this, _, _, cx| {
            this.page = target;
            cx.notify();
        }))
}

impl Render for WeaselApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let pending = self.pending_deletes(cx);
        let page = self.page;
        let deploying = self.deploying;
        let sidebar_w = cx.global::<SidebarResizeState>().width;
        let resizing = cx.global::<SidebarResizeState>().resizing;

        // 拖拽中：canvas 元素在 paint 阶段注册本帧全局鼠标监听（Capture 优先处理）
        let resizer_listeners = resizing.then(|| {
            canvas(
                |_, _, _| (),
                |_, _, window, _cx| {
                    window.on_mouse_event(|ev: &MouseMoveEvent, phase, window, cx| {
                        if phase != DispatchPhase::Capture {
                            return;
                        }
                        let viewport = f32::from(window.viewport_size().width);
                        let max_w = viewport / 3.0;
                        let changed = cx.update_global::<SidebarResizeState, _>(|s, _cx| {
                            if !s.resizing {
                                return false;
                            }
                            let dx = f32::from(ev.position.x) - s.start_x;
                            let next = (s.start_w + dx).clamp(SIDEBAR_MIN_W, max_w);
                            let changed = (next - s.width).abs() > f32::EPSILON;
                            s.width = next;
                            changed
                        });
                        if changed {
                            cx.refresh_windows();
                        }
                    });
                    window.on_mouse_event(|_ev: &MouseUpEvent, phase, window, cx| {
                        if phase != DispatchPhase::Capture {
                            return;
                        }
                        let _ = window;
                        let changed = cx.update_global::<SidebarResizeState, _>(|s, _cx| {
                            let was = s.resizing;
                            s.resizing = false;
                            was
                        });
                        if changed {
                            cx.refresh_windows();
                        }
                    });
                },
            )
        });

        h_flex()
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            .child(
                Sidebar::new("nav")
                    .w(px(sidebar_w))
                    .header(SidebarHeader::new().child("WeaselVision"))
                    .child(self.nav_global_group(cx))
                    .child(self.nav_schema_group(cx)),
            )
            .children(resizer_listeners)
            .child(
                // 拖拽分隔条
                div()
                    .id("sidebar-resizer")
                    .w(px(5.))
                    .h_full()
                    .cursor(CursorStyle::ResizeLeftRight)
                    .hover(|s| s.bg(cx.theme().border))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|_, ev: &MouseDownEvent, _, cx| {
                            cx.update_global::<SidebarResizeState, _>(|s, _| {
                                s.resizing = true;
                                s.start_x = f32::from(ev.position.x);
                                s.start_w = s.width;
                            });
                            cx.stop_propagation();
                        }),
                    ),
            )
            .child(
                v_flex()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .overflow_hidden()
                    .child(
                        // 顶部工具栏
                        h_flex()
                            .h(px(56.))
                            .items_center()
                            .justify_between()
                            .px_4()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .child(div().font_weight(FontWeight::MEDIUM).text_lg().child(page.title()))
                            .child(
                                h_flex()
                                    .gap_3()
                                    .items_center()
                                    .when(!pending.is_empty(), |bar| {
                                        let labels: Vec<String> = pending
                                            .iter()
                                            .map(|p| p.identifier.clone())
                                            .collect();
                                        bar.child(
                                            div()
                                                .text_sm()
                                                .text_color(cx.theme().danger)
                                                .child(format!(
                                                    "{} 项待删除：{}",
                                                    pending.len(),
                                                    labels.join("、")
                                                )),
                                        )
                                    })
                                    .child(
                                        Button::new("deploy")
                                            .primary()
                                            .disabled(deploying)
                                            .label(if deploying { "部署中…" } else { "重新部署" })
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.deploy(window, cx);
                                            })),
                                    ),
                            ),
                    )
                    .child(
                        // 定高容器：页面根的 size_full / flex_1 以此为参照。
                        // 不能用 overflow_y_scrollbar（它会把 child 强制为 h_auto，
                        // 页面内部 flex_1 就失效）；滚动由各页面自己管理。
                        v_flex()
                            .flex_1()
                            .min_h_0()
                            .overflow_hidden()
                            .p_6()
                            .child(self.current_page_view(window, cx)),
                    ),
            )
    }
}
