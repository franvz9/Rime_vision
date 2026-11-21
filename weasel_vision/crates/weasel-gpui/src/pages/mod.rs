//! 页面模块：每页一个独立 View（Entity 惰性创建 + 缓存）。

pub mod advanced;
pub mod backup;
pub mod dict;
pub mod general;
pub mod grammar;
pub mod keybinding;
pub mod page_size;
pub mod profile;
pub mod punctuation;
pub mod schema;
pub mod sync;
pub mod theme_editor;
pub mod translator;

use gpui_kit::*;
use rime_core::service::settings::PendingDelete;

/// 侧边栏页面（顺序 = Vue 版导航顺序）。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Page {
    General,
    Theme,
    Grammar,
    Backup,
    Dict,
    Sync,
    Advanced,
    Schema,
    Profile,
}

impl Page {
    pub fn title(self) -> &'static str {
        match self {
            Page::General => "通用设置",
            Page::Theme => "主题外观",
            Page::Grammar => "语言模型",
            Page::Backup => "备份管理",
            Page::Dict => "词典管理",
            Page::Sync => "同步管理",
            Page::Advanced => "高级设置",
            Page::Schema => "方案列表",
            Page::Profile => "方案配置",
        }
    }
}

/// 页面实体缓存：首次进入时创建，之后复用（保留未保存的编辑状态，与 Vue 版一致）。
#[derive(Default)]
pub struct PageCache {
    pub general: Option<Entity<general::GeneralPage>>,
    pub theme: Option<Entity<theme_editor::ThemePage>>,
    pub grammar: Option<Entity<grammar::GrammarPage>>,
    pub backup: Option<Entity<backup::BackupPage>>,
    pub dict: Option<Entity<dict::DictPage>>,
    pub sync: Option<Entity<sync::SyncPage>>,
    pub advanced: Option<Entity<advanced::AdvancedPage>>,
    pub schema: Option<Entity<schema::SchemaPage>>,
    pub profile: Option<Entity<profile::ProfilePage>>,
}

impl PageCache {
    /// 汇总三个管理页（schema/grammar/theme）的「待删除」项，供顶栏显示与部署。
    pub fn collect_pending(&self, cx: &App, out: &mut Vec<PendingDelete>) {
        if let Some(e) = &self.schema {
            out.extend(e.read(cx).pending_deletes());
        }
        if let Some(e) = &self.grammar {
            out.extend(e.read(cx).pending_deletes());
        }
        if let Some(e) = &self.theme {
            out.extend(e.read(cx).pending_deletes());
        }
    }

    /// 部署完成后清空各页待删除标记并通知刷新。
    pub fn on_deploy_complete(&self, window: &mut Window, cx: &mut App) {
        if let Some(e) = &self.schema {
            e.update(cx, |p, cx| p.on_deploy_complete(cx));
        }
        if let Some(e) = &self.grammar {
            e.update(cx, |p, cx| p.on_deploy_complete(cx));
        }
        if let Some(e) = &self.theme {
            // 部署会重写 selected_light/selected_dark，主题页需整体重载（M4）
            e.update(cx, |p, cx| p.on_deploy_complete(window, cx));
        }
    }
}
