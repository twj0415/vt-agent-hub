use crate::application::app_container::AppContainer;
use crate::application::snapshot_service::SnapshotService;
use crate::dto::{
    AppResponse, CatalogSnapshotDto, HistorySnapshotDto, ProjectDetailDto, SettingsSnapshotDto,
    ToolsSnapshotDto, WorkspaceSnapshotDto,
};

#[tauri::command]
pub fn get_workspace_snapshot(
    state: tauri::State<'_, AppContainer>,
) -> AppResponse<WorkspaceSnapshotDto> {
    let service = SnapshotService::with_container(state.inner());
    match service.get_workspace_snapshot() {
        Ok(data) => AppResponse::success(data),
        Err(error) => AppResponse::error(
            "workspace_snapshot_failed",
            &error,
            "errors.workspaceSnapshotFailed",
        ),
    }
}

#[tauri::command]
pub fn get_project_context_snapshot(
    state: tauri::State<'_, AppContainer>,
) -> AppResponse<WorkspaceSnapshotDto> {
    get_workspace_snapshot(state)
}

#[tauri::command]
pub fn get_catalog_snapshot(
    state: tauri::State<'_, AppContainer>,
) -> AppResponse<CatalogSnapshotDto> {
    let service = SnapshotService::with_container(state.inner());
    match service.get_catalog_snapshot() {
        Ok(data) => AppResponse::success(data),
        Err(error) => AppResponse::error(
            "catalog_snapshot_failed",
            &error,
            "errors.catalogSnapshotFailed",
        ),
    }
}

#[tauri::command]
pub fn get_library_snapshot(
    state: tauri::State<'_, AppContainer>,
) -> AppResponse<CatalogSnapshotDto> {
    get_catalog_snapshot(state)
}

#[tauri::command]
pub fn get_tools_snapshot(
    state: tauri::State<'_, AppContainer>,
    tool_id: Option<i32>,
) -> AppResponse<ToolsSnapshotDto> {
    let service = SnapshotService::with_container(state.inner());
    match service.get_tools_snapshot(tool_id) {
        Ok(data) => AppResponse::success(data),
        Err(error) => AppResponse::error(
            "tools_snapshot_failed",
            &error,
            "errors.toolsSnapshotFailed",
        ),
    }
}

#[tauri::command]
pub fn get_project_detail(
    state: tauri::State<'_, AppContainer>,
    project_id: i32,
) -> AppResponse<ProjectDetailDto> {
    let service = SnapshotService::with_container(state.inner());
    match service.get_project_detail(project_id) {
        Ok(data) => AppResponse::success(data),
        Err(error) => AppResponse::error(
            "project_detail_failed",
            &error,
            "errors.projectDetailFailed",
        ),
    }
}

#[tauri::command]
pub fn get_history_snapshot(
    state: tauri::State<'_, AppContainer>,
) -> AppResponse<HistorySnapshotDto> {
    let service = SnapshotService::with_container(state.inner());
    match service.get_history_snapshot() {
        Ok(data) => AppResponse::success(data),
        Err(error) => AppResponse::error(
            "history_snapshot_failed",
            &error,
            "errors.historySnapshotFailed",
        ),
    }
}

#[tauri::command]
pub fn get_settings_snapshot(
    state: tauri::State<'_, AppContainer>,
) -> AppResponse<SettingsSnapshotDto> {
    let service = SnapshotService::with_container(state.inner());
    match service.get_settings_snapshot() {
        Ok(data) => AppResponse::success(data),
        Err(error) => AppResponse::error(
            "settings_snapshot_failed",
            &error,
            "errors.settingsSnapshotFailed",
        ),
    }
}
