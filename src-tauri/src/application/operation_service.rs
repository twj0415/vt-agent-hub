use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::routes::{
    ROUTE_HISTORY, ROUTE_PRESETS, ROUTE_PROJECTS, ROUTE_RULES, ROUTE_SETTINGS, ROUTE_SKILLS,
};
use crate::core::status_codes::{HEALTH_NORMAL, HEALTH_WARNING};
use crate::infrastructure::database::Database;
use crate::infrastructure::history_repo::{HistoryRepo, NewHistoryRecord};

pub struct OperationContext<'a> {
    pub project_id: Option<i32>,
    pub tool_id: Option<i32>,
    pub related_rule_id: Option<i32>,
    pub kind: &'a str,
    pub title: &'a str,
    pub action: &'a str,
    pub detail: &'a str,
    pub result: &'a str,
    pub result_code: i32,
    pub related_path: Option<&'a str>,
    pub navigation_target: &'a str,
}

/// 历史/审计日志写入器。无状态:所有方法接收 `db: &Database` 由调用方提供,
/// 让 caller(WriteService 等)在持有同一连接的 `MutexGuard` 范围内调用 record,
/// 避免重入 `Arc<Mutex<Database>>` 死锁,也保证业务写与审计写在同一连接上。
pub struct OperationService;

impl OperationService {
    pub fn record(
        db: &Database,
        project_id: Option<i32>,
        kind: &str,
        title: &str,
        action: &str,
        detail: &str,
    ) -> Result<(), String> {
        Self::record_with_context(
            db,
            OperationContext {
                project_id,
                tool_id: None,
                related_rule_id: None,
                kind,
                title,
                action,
                detail,
                result: "success",
                result_code: HEALTH_NORMAL,
                related_path: None,
                navigation_target: "",
            },
        )
    }

    pub fn record_simple(
        db: &Database,
        project_id: Option<i32>,
        tool_id: Option<i32>,
        related_rule_id: Option<i32>,
        kind: &str,
        title: &str,
        action: &str,
        detail: &str,
        result: &str,
        result_code: i32,
        related_path: Option<&str>,
        navigation_target: &str,
    ) -> Result<(), String> {
        Self::record_with_context(
            db,
            OperationContext {
                project_id,
                tool_id,
                related_rule_id,
                kind,
                title,
                action,
                detail,
                result,
                result_code,
                related_path,
                navigation_target,
            },
        )
    }

    pub fn record_failure(
        db: &Database,
        project_id: Option<i32>,
        tool_id: Option<i32>,
        related_rule_id: Option<i32>,
        kind: &str,
        title: &str,
        action: &str,
        detail: &str,
        related_path: Option<&str>,
        navigation_target: &str,
    ) -> Result<(), String> {
        Self::record_with_context(
            db,
            OperationContext {
                project_id,
                tool_id,
                related_rule_id,
                kind,
                title,
                action,
                detail,
                result: "failed",
                result_code: HEALTH_WARNING,
                related_path,
                navigation_target,
            },
        )
    }

    pub fn record_with_context(db: &Database, context: OperationContext<'_>) -> Result<(), String> {
        let navigation_target = if context.navigation_target.trim().is_empty() {
            default_navigation_target(context.project_id, context.kind, context.action)
        } else {
            context.navigation_target
        };

        HistoryRepo::new(db).insert(NewHistoryRecord {
            project_id: context.project_id,
            tool_id: context.tool_id,
            related_rule_id: context.related_rule_id,
            kind: context.kind,
            title: context.title,
            created_at: current_timestamp(),
            action: context.action,
            result: context.result,
            result_code: context.result_code,
            detail: context.detail,
            related_path: context.related_path.unwrap_or("").to_string(),
            navigation_target,
        })
    }
}

fn default_navigation_target(project_id: Option<i32>, kind: &str, action: &str) -> &'static str {
    if project_id.is_some() {
        return ROUTE_PROJECTS;
    }

    let normalized = action.replace('_', "-").to_ascii_lowercase();
    if normalized.contains("rule") {
        ROUTE_RULES
    } else if normalized.contains("skill") {
        ROUTE_SKILLS
    } else if normalized.contains("preset") || normalized.contains("global") {
        ROUTE_PRESETS
    } else if normalized.contains("credential") {
        ROUTE_SETTINGS
    } else if kind == "backup" || kind == "diagnostic" {
        ROUTE_HISTORY
    } else {
        ROUTE_HISTORY
    }
}

fn current_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}
