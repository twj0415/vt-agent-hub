use crate::application::app_container::AppContainer;
use crate::application::provider_runtime_service::ProviderRuntimeService;
use crate::commands::history_log::{record_command_failure, CommandFailure};
use crate::core::routes::ROUTE_PRESETS;
use crate::dto::{
    AppResponse, ProviderApplyPreviewDto, ProviderApplyResultDto, ProviderImportDraftDto,
    ProviderImportInputDto, ProviderSaveInputDto, ProviderSummaryDto,
};

#[tauri::command]
pub fn list_providers(
    state: tauri::State<'_, AppContainer>,
    tool_id: Option<i32>,
) -> AppResponse<Vec<ProviderSummaryDto>> {
    let service = ProviderRuntimeService::with_container(state.inner());

    match service.list(tool_id) {
        Ok(items) => AppResponse::success(items),
        Err(error) => {
            AppResponse::error("provider_list_failed", &error, "errors.providerListFailed")
        }
    }
}

#[tauri::command]
pub fn save_provider(
    state: tauri::State<'_, AppContainer>,
    payload: ProviderSaveInputDto,
) -> AppResponse<ProviderSummaryDto> {
    let service = ProviderRuntimeService::with_container(state.inner());

    match service.save(payload) {
        Ok(item) => AppResponse::success(item),
        Err(error) => {
            record_failure("Provider save failed", "provider-write", &error);
            AppResponse::error("provider_save_failed", &error, "errors.providerSaveFailed")
        }
    }
}

#[tauri::command]
pub fn import_provider_config(
    state: tauri::State<'_, AppContainer>,
    payload: ProviderImportInputDto,
) -> AppResponse<ProviderImportDraftDto> {
    let service = ProviderRuntimeService::with_container(state.inner());

    match service.import_config(payload) {
        Ok(item) => AppResponse::success(item),
        Err(error) => {
            record_failure("Provider import failed", "provider-import-config", &error);
            AppResponse::error(
                "provider_import_failed",
                &error,
                "errors.providerImportFailed",
            )
        }
    }
}

#[tauri::command]
pub fn delete_provider(
    state: tauri::State<'_, AppContainer>,
    provider_id: i32,
) -> AppResponse<bool> {
    let service = ProviderRuntimeService::with_container(state.inner());

    match service.delete(provider_id) {
        Ok(()) => AppResponse::success(true),
        Err(error) => {
            record_failure("Provider delete failed", "provider-delete", &error);
            AppResponse::error(
                "provider_delete_failed",
                &error,
                "errors.providerDeleteFailed",
            )
        }
    }
}

#[tauri::command]
pub fn duplicate_provider(
    state: tauri::State<'_, AppContainer>,
    provider_id: i32,
) -> AppResponse<ProviderSummaryDto> {
    let service = ProviderRuntimeService::with_container(state.inner());

    match service.duplicate(provider_id) {
        Ok(item) => AppResponse::success(item),
        Err(error) => {
            record_failure("Provider duplicate failed", "provider-duplicate", &error);
            AppResponse::error(
                "provider_duplicate_failed",
                &error,
                "errors.providerDuplicateFailed",
            )
        }
    }
}

#[tauri::command]
pub fn preview_provider_apply(
    state: tauri::State<'_, AppContainer>,
    config_id: i32,
) -> AppResponse<ProviderApplyPreviewDto> {
    let service = ProviderRuntimeService::with_container(state.inner());

    match service.preview_apply(config_id) {
        Ok(preview) => AppResponse::success(preview),
        Err(error) => {
            record_failure("Provider apply preview failed", "provider-preview", &error);
            AppResponse::error(
                "provider_preview_apply_failed",
                &error,
                "errors.providerPreviewApplyFailed",
            )
        }
    }
}

#[tauri::command]
pub fn apply_provider_to_live_config(
    state: tauri::State<'_, AppContainer>,
    config_id: i32,
    confirm_risk: bool,
) -> AppResponse<ProviderApplyResultDto> {
    let service = ProviderRuntimeService::with_container(state.inner());

    match service.apply(config_id, confirm_risk) {
        Ok(result) => AppResponse::success(result),
        Err(error) => {
            record_failure("Provider apply failed", "provider-apply", &error);
            AppResponse::error(
                "provider_apply_failed",
                &error,
                "errors.providerApplyFailed",
            )
        }
    }
}

fn record_failure(title: &str, action: &str, detail: &str) {
    record_command_failure(CommandFailure {
        project_id: None,
        tool_id: None,
        related_rule_id: None,
        kind: "operation",
        title,
        action,
        detail,
        related_path: None,
        navigation_target: ROUTE_PRESETS,
    });
}
