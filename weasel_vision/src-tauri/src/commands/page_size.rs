use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use ts_rs::TS;

use crate::rime::config::{self, RimeConfig};
use crate::rime::error::RimeError;
use crate::rime::patch::get_patch_path_value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, TS)]
pub struct PageSizeDetail {
    pub page_size: i64,
    pub is_custom: bool,
}

/// 判断 custom 文件中是否显式定义了给定 patch 路径（兼容 slash-form
/// 「menu/page_size」与 nested-form 「menu: { page_size: ... }」）。
fn is_patch_path_custom(custom: &Value, path: &str) -> bool {
    custom
        .get(Value::String("patch".into()))
        .and_then(|v| v.as_mapping())
        .map(|m| get_patch_path_value(m, path).is_some())
        .unwrap_or(false)
}

#[tauri::command]
pub fn get_page_size_detail(scope: Option<String>) -> Result<PageSizeDetail, RimeError> {
    let cfg = RimeConfig::detect();

    // 计算生效值
    let dict_value = match scope.as_deref() {
        None => cfg.load_effective(&cfg.default_path(), &cfg.default_custom_path())?,
        Some(id) => cfg.load_schema_effective(id)?,
    };
    let dict = config::value_as_mapping(&dict_value);
    let menu = config::get_mapping(dict, "menu");
    let page_size = config::get_i64(menu, "page_size").unwrap_or(6);

    // 判断是否自定义（custom 文件中是否存在该 patch 键）
    let custom_path = match scope.as_deref() {
        None => cfg.default_custom_path(),
        Some(id) => cfg
            .schema_custom_path(id)
            .ok_or_else(|| RimeError::InvalidPath(format!("Invalid schema_id: {}", id)))?,
    };
    let custom = cfg.load_yaml(&custom_path)?;
    let is_custom = is_patch_path_custom(&custom, "menu/page_size");

    Ok(PageSizeDetail { page_size, is_custom })
}

#[tauri::command]
pub fn save_page_size(scope: Option<String>, page_size: i64) -> Result<(), RimeError> {
    let cfg = RimeConfig::detect();
    let target_path = match scope.as_deref() {
        None => cfg.default_custom_path(),
        Some(id) => cfg
            .schema_custom_path(id)
            .ok_or_else(|| RimeError::InvalidPath(format!("Invalid schema_id: {}", id)))?,
    };
    cfg.save_patch(&target_path, |patch| {
        patch.insert(
            Value::String("menu/page_size".into()),
            Value::Number(page_size.into()),
        );
        Ok(())
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rime::config::RimeConfig;
    use tempfile::tempdir;

    fn test_cfg() -> (tempfile::TempDir, RimeConfig) {
        let tmp = tempdir().unwrap();
        let cfg = RimeConfig {
            user_dir: tmp.path().to_path_buf(),
            build_dir: None,
            style_file: "squirrel.yaml".into(),
            style_custom: "squirrel.custom.yaml".into(),
            default_yaml: "default.yaml".into(),
            default_custom: "default.custom.yaml".into(),
        };
        (tmp, cfg)
    }

    #[test]
    fn test_page_size_scope_schema() {
        let (_tmp, cfg) = test_cfg();
        std::fs::write(
            cfg.user_dir.join("ps_schema.schema.yaml"),
            "menu:\n  page_size: 6\n",
        )
        .unwrap();
        // custom patch 覆盖 page_size=9
        std::fs::write(
            cfg.user_dir.join("ps_schema.custom.yaml"),
            "patch:\n  menu/page_size: 9\n",
        )
        .unwrap();
        let eff = cfg.load_schema_effective("ps_schema").unwrap();
        let dict = config::value_as_mapping(&eff);
        let menu = config::get_mapping(dict, "menu");
        let v = config::get_i64(menu, "page_size");
        assert_eq!(v, Some(9));
    }

    #[test]
    fn test_get_page_size_detail_marks_custom() {
        let (_tmp, cfg) = test_cfg();
        // 方案 custom 显式定义 page_size
        std::fs::write(
            cfg.user_dir.join("ps_d.schema.yaml"),
            "menu:\n  page_size: 6\n",
        )
        .unwrap();
        std::fs::write(
            cfg.user_dir.join("ps_d.custom.yaml"),
            "patch:\n  menu/page_size: 9\n",
        )
        .unwrap();
        let detail = get_page_size_detail_inner(&cfg, Some("ps_d".to_string())).unwrap();
        assert_eq!(detail.page_size, 9);
        assert!(detail.is_custom);
    }

    #[test]
    fn test_get_page_size_detail_not_custom() {
        let (_tmp, cfg) = test_cfg();
        // 方案 custom 无 page_size
        std::fs::write(
            cfg.user_dir.join("ps_n.schema.yaml"),
            "menu:\n  page_size: 6\n",
        )
        .unwrap();
        let detail = get_page_size_detail_inner(&cfg, Some("ps_n".to_string())).unwrap();
        assert_eq!(detail.page_size, 6);
        assert!(!detail.is_custom);
    }

    #[test]
    fn test_get_page_size_detail_custom_nested_form() {
        let (_tmp, cfg) = test_cfg();
        // 手写 nested-form：menu: { page_size: ... } 也应识别为自定义
        std::fs::write(
            cfg.user_dir.join("ps_nd.schema.yaml"),
            "menu:\n  page_size: 6\n",
        )
        .unwrap();
        std::fs::write(
            cfg.user_dir.join("ps_nd.custom.yaml"),
            "patch:\n  menu:\n    page_size: 9\n",
        )
        .unwrap();
        let detail = get_page_size_detail_inner(&cfg, Some("ps_nd".to_string())).unwrap();
        assert_eq!(detail.page_size, 9);
        assert!(detail.is_custom, "nested-form menu/page_size 应识别为自定义");
    }

    /// 测试用内部封装（避免 Tauri command 的注入依赖）
    fn get_page_size_detail_inner(
        cfg: &RimeConfig,
        scope: Option<String>,
    ) -> Result<PageSizeDetail, RimeError> {
        let dict_value = match scope.as_deref() {
            None => cfg.load_effective(&cfg.default_path(), &cfg.default_custom_path())?,
            Some(id) => cfg.load_schema_effective(id)?,
        };
        let dict = config::value_as_mapping(&dict_value);
        let menu = config::get_mapping(dict, "menu");
        let page_size = config::get_i64(menu, "page_size").unwrap_or(6);

        let custom_path = match scope.as_deref() {
            None => cfg.default_custom_path(),
            Some(id) => cfg
                .schema_custom_path(id)
                .ok_or_else(|| RimeError::InvalidPath(format!("Invalid schema_id: {}", id)))?,
        };
        let custom = cfg.load_yaml(&custom_path)?;
        let is_custom = is_patch_path_custom(&custom, "menu/page_size");

        Ok(PageSizeDetail { page_size, is_custom })
    }
}
