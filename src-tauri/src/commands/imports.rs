use crate::application::app_container::AppContainer;
use crate::application::repository_import_service::RepositoryImportService;
use crate::commands::history_log::{record_command_failure, CommandFailure};
use crate::core::routes::ROUTE_RULES;
use crate::dto::{AppResponse, RepositoryImportReportDto};

#[tauri::command]
pub fn preview_repository_import(
    state: tauri::State<'_, AppContainer>,
    source: String,
    branch: String,
    conflict_strategy: String,
) -> AppResponse<RepositoryImportReportDto> {
    let service = RepositoryImportService::with_container(state.inner());

    match service.preview_repository(&source, &branch, &conflict_strategy) {
        Ok(report) => AppResponse::success(report),
        Err(error) => {
            record_history_failure(
                "operation",
                "Repository import preview failed",
                "repository-import-preview",
                &error,
                Some(&source),
            );
            AppResponse::error(
                "repository_import_preview_failed",
                &error,
                "errors.repositoryImportPreviewFailed",
            )
        }
    }
}

#[tauri::command]
pub fn apply_repository_import(
    state: tauri::State<'_, AppContainer>,
    source: String,
    branch: String,
    conflict_strategy: String,
) -> AppResponse<RepositoryImportReportDto> {
    let service = RepositoryImportService::with_container(state.inner());

    match service.apply_repository(&source, &branch, &conflict_strategy) {
        Ok(report) => AppResponse::success(report),
        Err(error) => {
            record_history_failure(
                "operation",
                "Repository import failed",
                "repository-import",
                &error,
                Some(&source),
            );
            AppResponse::error(
                "repository_import_failed",
                &error,
                "errors.repositoryImportFailed",
            )
        }
    }
}

fn record_history_failure(
    kind: &str,
    title: &str,
    action: &str,
    detail: &str,
    related_path: Option<&str>,
) {
    record_command_failure(CommandFailure {
        project_id: None,
        tool_id: None,
        related_rule_id: None,
        kind,
        title,
        action,
        detail,
        related_path,
        navigation_target: ROUTE_RULES,
    });
}
