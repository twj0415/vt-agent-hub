use crate::application::app_container::AppContainer;
use crate::application::first_run_import_service::FirstRunImportService;
use crate::core::routes::ROUTE_SETTINGS;
use crate::dto::{
    AppResponse, FirstRunImportApplyInputDto, FirstRunImportApplyResultDto,
    FirstRunImportPreviewDto, FirstRunImportStatusDto,
};

#[tauri::command]
pub fn get_first_run_import_status(
    state: tauri::State<'_, AppContainer>,
) -> AppResponse<FirstRunImportStatusDto> {
    wrap_command!(
        service: FirstRunImportService::with_container(state.inner()),
        call: |service: FirstRunImportService| service.status(),
        error_code: "first_run_import_status_failed",
        i18n: "errors.firstRunImportStatusFailed",
    )
}

#[tauri::command]
pub fn preview_first_run_import(
    state: tauri::State<'_, AppContainer>,
) -> AppResponse<FirstRunImportPreviewDto> {
    wrap_command!(
        service: FirstRunImportService::with_container(state.inner()),
        call: |service: FirstRunImportService| service.preview(),
        error_code: "first_run_import_preview_failed",
        i18n: "errors.firstRunImportPreviewFailed",
        history: { kind: "operation", title: "First-run import preview failed", action: "first-run-import-preview", route: ROUTE_SETTINGS },
    )
}

#[tauri::command]
pub fn apply_first_run_import(
    state: tauri::State<'_, AppContainer>,
    payload: FirstRunImportApplyInputDto,
) -> AppResponse<FirstRunImportApplyResultDto> {
    wrap_command!(
        service: FirstRunImportService::with_container(state.inner()),
        call: |service: FirstRunImportService| service.apply(payload),
        error_code: "first_run_import_apply_failed",
        i18n: "errors.firstRunImportApplyFailed",
        history: { kind: "operation", title: "First-run import apply failed", action: "first-run-import-apply", route: ROUTE_SETTINGS },
    )
}

#[tauri::command]
pub fn dismiss_first_run_import(
    state: tauri::State<'_, AppContainer>,
    status: String,
    reason: Option<String>,
) -> AppResponse<FirstRunImportStatusDto> {
    wrap_command!(
        service: FirstRunImportService::with_container(state.inner()),
        call: |service: FirstRunImportService| service.dismiss(&status, reason.as_deref()),
        error_code: "first_run_import_dismiss_failed",
        i18n: "errors.firstRunImportDismissFailed",
    )
}

#[tauri::command]
pub fn reset_first_run_import_status(
    state: tauri::State<'_, AppContainer>,
) -> AppResponse<FirstRunImportStatusDto> {
    wrap_command!(
        service: FirstRunImportService::with_container(state.inner()),
        call: |service: FirstRunImportService| service.reset_status(),
        error_code: "first_run_import_reset_failed",
        i18n: "errors.firstRunImportResetFailed",
    )
}
