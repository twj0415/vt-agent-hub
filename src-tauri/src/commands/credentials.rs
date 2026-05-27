use crate::application::app_container::AppContainer;
use crate::application::write_service::WriteService;
use crate::core::routes::ROUTE_SETTINGS;
use crate::dto::AppResponse;

#[tauri::command]
pub fn save_tool_credential_state(
    app_state: tauri::State<'_, AppContainer>,
    tool_id: i32,
    token: String,
) -> AppResponse<bool> {
    wrap_command!(
        service: WriteService::with_container(app_state.inner()),
        call: |service: WriteService| service.save_tool_credential(tool_id, &token).map(|_| true),
        error_code: "credential_save_failed",
        i18n: "errors.credentialSaveFailed",
        history: { kind: "operation", title: "Credential save failed", action: "credential-save", route: ROUTE_SETTINGS },
    )
}

#[tauri::command]
pub fn clear_tool_credential_state(
    app_state: tauri::State<'_, AppContainer>,
    tool_id: i32,
) -> AppResponse<bool> {
    wrap_command!(
        service: WriteService::with_container(app_state.inner()),
        call: |service: WriteService| service.clear_tool_credential(tool_id).map(|_| true),
        error_code: "credential_clear_failed",
        i18n: "errors.credentialClearFailed",
        history: { kind: "operation", title: "Credential clear failed", action: "credential-clear", route: ROUTE_SETTINGS },
    )
}
