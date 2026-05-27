use crate::application::app_container::AppContainer;
use crate::application::write_service::WriteService;
use crate::core::routes::ROUTE_RULES;
use crate::dto::{
    AppResponse, RuleImpactDto, RuleImportPreviewDto, RuleImportResultDto, RuleSummaryDto,
};

#[tauri::command]
pub fn save_rule_asset(
    app_state: tauri::State<'_, AppContainer>,
    id: Option<i32>,
    code: i32,
    name: String,
    category_code: i32,
    state: i32,
    summary: String,
    body: String,
) -> AppResponse<RuleSummaryDto> {
    wrap_command!(
        service: WriteService::with_container(app_state.inner()),
        call: |service: WriteService| service
            .save_rule(id, code, &name, category_code, state, &summary, &body)
            .map(|saved_id| RuleSummaryDto {
                asset_id: saved_id,
                version_id: 0,
                version_no: 0,
                key: String::new(),
                code,
                name,
                category_code,
                state,
                sort_order: 0,
                summary,
                body,
            }),
        error_code: "rule_save_failed",
        i18n: "errors.ruleSaveFailed",
        history: { kind: "operation", title: "Rule save failed", action: "rule-write", route: ROUTE_RULES },
    )
}

#[tauri::command]
pub fn preview_rule_impact(
    app_state: tauri::State<'_, AppContainer>,
    rule_id: i32,
) -> AppResponse<RuleImpactDto> {
    wrap_command!(
        service: WriteService::with_container(app_state.inner()),
        call: |service: WriteService| service.preview_rule_impact(rule_id),
        error_code: "rule_impact_failed",
        i18n: "errors.ruleImpactFailed",
        history: { kind: "operation", title: "Rule impact preview failed", action: "rule-impact", route: ROUTE_RULES },
    )
}

#[tauri::command]
pub fn import_rule_asset(
    app_state: tauri::State<'_, AppContainer>,
    source_path: String,
    name: String,
    category_code: i32,
    summary: String,
    conflict_strategy: String,
) -> AppResponse<RuleImportResultDto> {
    wrap_command!(
        service: WriteService::with_container(app_state.inner()),
        call: |service: WriteService| service.import_rule(
            &source_path,
            &name,
            category_code,
            &summary,
            &conflict_strategy,
        ),
        error_code: "rule_import_failed",
        i18n: "errors.ruleImportFailed",
        history: { kind: "operation", title: "Rule import failed", action: "rule-import", route: ROUTE_RULES, related_path: &source_path },
    )
}

#[tauri::command]
pub fn preview_rule_import(
    app_state: tauri::State<'_, AppContainer>,
    source_path: String,
) -> AppResponse<RuleImportPreviewDto> {
    wrap_command!(
        service: WriteService::with_container(app_state.inner()),
        call: |service: WriteService| service.preview_rule_import(&source_path),
        error_code: "rule_import_preview_failed",
        i18n: "errors.ruleImportFailed",
    )
}

#[tauri::command]
pub fn move_rule_asset(
    app_state: tauri::State<'_, AppContainer>,
    rule_id: i32,
    category_code: i32,
    sort_order: i32,
) -> AppResponse<RuleSummaryDto> {
    wrap_command!(
        service: WriteService::with_container(app_state.inner()),
        call: |service: WriteService| service.move_rule(rule_id, category_code, sort_order),
        error_code: "rule_move_failed",
        i18n: "errors.ruleMoveFailed",
        history: { kind: "operation", title: "Rule move failed", action: "rule-move", route: ROUTE_RULES },
    )
}

#[tauri::command]
pub fn delete_rule_asset(
    app_state: tauri::State<'_, AppContainer>,
    rule_id: i32,
) -> AppResponse<bool> {
    let service = WriteService::with_container(app_state.inner());
    match service.delete_rule(rule_id) {
        Ok(()) => AppResponse::success(true),
        Err(error) => {
            crate::commands::history_log::record_command_failure(
                crate::commands::history_log::CommandFailure {
                    project_id: None,
                    tool_id: None,
                    related_rule_id: None,
                    kind: "operation",
                    title: "Rule delete failed",
                    action: "rule-delete",
                    detail: &error,
                    related_path: None,
                    navigation_target: ROUTE_RULES,
                },
            );
            let i18n_key = if error.contains("Unbind it before deleting") {
                "errors.ruleDeleteBound"
            } else {
                "errors.ruleDeleteFailed"
            };
            AppResponse::error("rule_delete_failed", &error, i18n_key)
        }
    }
}
