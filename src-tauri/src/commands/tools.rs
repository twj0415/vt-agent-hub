use crate::application::app_container::AppContainer;
use crate::application::tool_management_service::ToolManagementService;
use crate::application::tool_service::ToolService;
use crate::dto::{AppResponse, ToolActionResultDto, ToolDiagnosticsDto};

#[tauri::command]
pub fn set_tool_enabled(
    state: tauri::State<'_, AppContainer>,
    tool_id: i32,
    enabled: bool,
) -> AppResponse<bool> {
    let service = ToolManagementService::with_container(state.inner());

    match service.set_enabled(tool_id, enabled) {
        Ok(()) => AppResponse::success(true),
        Err(error) => AppResponse::error(
            "tool_set_enabled_failed",
            &error,
            "errors.toolSetEnabledFailed",
        ),
    }
}

#[tauri::command]
pub fn get_tool_diagnostics(tool_id: i32) -> AppResponse<ToolDiagnosticsDto> {
    let service = ToolService::new();

    match service.get_diagnostics(tool_id) {
        Ok(diagnostics) => AppResponse::success(ToolDiagnosticsDto {
            installation_detected: diagnostics.installation_detected,
            version: diagnostics.version,
            live_config_path: diagnostics.live_config_path,
            credential_state: diagnostics.credential_state,
            credential_state_code: diagnostics.credential_state_code,
            skill_state: diagnostics.skill_state,
            skill_state_code: diagnostics.skill_state_code,
            project_output_state: diagnostics.project_output_state,
            project_output_state_code: diagnostics.project_output_state_code,
            repair_state: diagnostics.repair_state,
            repair_state_code: diagnostics.repair_state_code,
            repair_hint: diagnostics.repair_hint,
        }),
        Err(error) => AppResponse::error(
            "tool_diagnostics_failed",
            &error,
            "errors.toolDiagnosticsFailed",
        ),
    }
}

#[tauri::command]
pub fn verify_tool_credential(tool_id: i32, token: String) -> AppResponse<ToolActionResultDto> {
    let service = ToolService::new();

    match service.verify_credential(tool_id, &token) {
        Ok(result) => AppResponse::success(ToolActionResultDto {
            ok: result.ok,
            state: result.state,
            detail: result.detail,
            manual_steps: result.manual_steps,
        }),
        Err(error) => AppResponse::error(
            "tool_verify_credential_failed",
            &error,
            "errors.toolVerifyCredentialFailed",
        ),
    }
}

#[tauri::command]
pub fn repair_tool(tool_id: i32) -> AppResponse<ToolActionResultDto> {
    let service = ToolService::new();

    match service.repair(tool_id) {
        Ok(result) => AppResponse::success(ToolActionResultDto {
            ok: result.ok,
            state: result.state,
            detail: result.detail,
            manual_steps: result.manual_steps,
        }),
        Err(error) => AppResponse::error("tool_repair_failed", &error, "errors.toolRepairFailed"),
    }
}
