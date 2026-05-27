use crate::application::app_container::AppContainer;
use crate::application::global_output_service::GlobalOutputService;
use crate::application::project_workflow_service::ProjectWorkflowService;
use crate::application::write_service::WriteService;
use crate::commands::history_log::{record_command_failure, CommandFailure};
use crate::core::routes::{ROUTE_PRESETS, ROUTE_PROJECTS, ROUTE_RULES, ROUTE_SKILLS};
use crate::dto::{
    AppResponse, GlobalOutputPreviewDto, GlobalOutputWriteDto, ProjectDetailDto,
    ProjectOutputPreviewDto, ProjectOutputScanDto, ProjectOutputWriteDto,
};

#[tauri::command]
pub fn scan_project_output(
    state: tauri::State<'_, AppContainer>,
    project_id: i32,
    tool_id: i32,
) -> AppResponse<ProjectOutputScanDto> {
    let service = ProjectWorkflowService::with_container(state.inner());

    match service.scan(project_id, tool_id) {
        Ok(scan) => AppResponse::success(scan),
        Err(error) => AppResponse::error(
            "project_output_scan_failed",
            &error,
            "errors.projectOutputScanFailed",
        ),
    }
}

#[tauri::command]
pub fn preview_project_output(
    state: tauri::State<'_, AppContainer>,
    project_id: i32,
    tool_id: i32,
) -> AppResponse<ProjectOutputPreviewDto> {
    let service = ProjectWorkflowService::with_container(state.inner());

    match service.preview(project_id, tool_id) {
        Ok(preview) => AppResponse::success(preview),
        Err(error) => {
            record_history_failure(
                Some(project_id),
                Some(tool_id),
                "operation",
                "Workspace preview failed",
                "project-preview",
                &error,
                Some(ROUTE_PROJECTS),
            );
            AppResponse::error(
                "project_output_preview_failed",
                &error,
                "errors.projectOutputPreviewFailed",
            )
        }
    }
}

#[tauri::command]
pub fn apply_project_output(
    state: tauri::State<'_, AppContainer>,
    project_id: i32,
    tool_id: i32,
    confirm_risk: bool,
) -> AppResponse<ProjectOutputWriteDto> {
    let service = ProjectWorkflowService::with_container(state.inner());

    match service.apply(project_id, tool_id, confirm_risk) {
        Ok(result) => AppResponse::success(result),
        Err(error) => {
            record_history_failure(
                Some(project_id),
                Some(tool_id),
                "operation",
                "Workspace apply failed",
                "project.apply_agents",
                &error,
                Some(ROUTE_PROJECTS),
            );
            AppResponse::error(
                "project_output_apply_failed",
                &error,
                "errors.projectOutputApplyFailed",
            )
        }
    }
}

#[tauri::command]
pub fn repair_project_output(
    state: tauri::State<'_, AppContainer>,
    project_id: i32,
    tool_id: i32,
    confirm_risk: bool,
) -> AppResponse<ProjectOutputWriteDto> {
    let service = ProjectWorkflowService::with_container(state.inner());

    match service.repair(project_id, tool_id, confirm_risk) {
        Ok(result) => AppResponse::success(result),
        Err(error) => {
            record_history_failure(
                Some(project_id),
                Some(tool_id),
                "repair",
                "Workspace repair failed",
                "project.repair_agents",
                &error,
                Some(ROUTE_PROJECTS),
            );
            AppResponse::error(
                "project_output_repair_failed",
                &error,
                "errors.projectOutputRepairFailed",
            )
        }
    }
}

#[tauri::command]
pub fn cleanup_project_output(
    state: tauri::State<'_, AppContainer>,
    project_id: i32,
    tool_id: i32,
    confirm_risk: bool,
) -> AppResponse<ProjectOutputWriteDto> {
    let service = ProjectWorkflowService::with_container(state.inner());

    match service.cleanup(project_id, tool_id, confirm_risk) {
        Ok(result) => AppResponse::success(result),
        Err(error) => {
            record_history_failure(
                Some(project_id),
                Some(tool_id),
                "operation",
                "Project AGENTS cleanup failed",
                "project.cleanup_agents",
                &error,
                Some(ROUTE_PROJECTS),
            );
            AppResponse::error(
                "project_output_cleanup_failed",
                &error,
                "errors.projectOutputCleanupFailed",
            )
        }
    }
}

#[tauri::command]
pub fn reset_project_output(
    state: tauri::State<'_, AppContainer>,
    project_id: i32,
    tool_id: i32,
    confirm_risk: bool,
) -> AppResponse<ProjectOutputWriteDto> {
    let service = ProjectWorkflowService::with_container(state.inner());

    match service.reset(project_id, tool_id, confirm_risk) {
        Ok(result) => AppResponse::success(result),
        Err(error) => {
            record_history_failure(
                Some(project_id),
                Some(tool_id),
                "operation",
                "Project AGENTS reset failed",
                "project.reset_agents",
                &error,
                Some(ROUTE_PROJECTS),
            );
            AppResponse::error(
                "project_output_reset_failed",
                &error,
                "errors.projectOutputResetFailed",
            )
        }
    }
}

#[tauri::command]
pub fn preview_global_output(
    state: tauri::State<'_, AppContainer>,
    tool_id: i32,
) -> AppResponse<GlobalOutputPreviewDto> {
    let service = GlobalOutputService::with_container(state.inner());

    match service.preview(tool_id) {
        Ok(preview) => AppResponse::success(preview),
        Err(error) => {
            record_history_failure(
                None,
                Some(tool_id),
                "operation",
                "Global AGENTS preview failed",
                "global-preview",
                &error,
                Some(ROUTE_PRESETS),
            );
            AppResponse::error(
                "global_output_preview_failed",
                &error,
                "errors.globalOutputPreviewFailed",
            )
        }
    }
}

#[tauri::command]
pub fn apply_global_output(
    state: tauri::State<'_, AppContainer>,
    tool_id: i32,
    confirm_risk: bool,
) -> AppResponse<GlobalOutputWriteDto> {
    let service = GlobalOutputService::with_container(state.inner());

    match service.apply(tool_id, confirm_risk) {
        Ok(result) => AppResponse::success(result),
        Err(error) => {
            record_history_failure(
                None,
                Some(tool_id),
                "operation",
                "Global AGENTS apply failed",
                "global.apply_agents",
                &error,
                Some(ROUTE_PRESETS),
            );
            AppResponse::error(
                "global_output_apply_failed",
                &error,
                "errors.globalOutputApplyFailed",
            )
        }
    }
}

#[tauri::command]
pub fn repair_global_output(
    state: tauri::State<'_, AppContainer>,
    tool_id: i32,
    confirm_risk: bool,
) -> AppResponse<GlobalOutputWriteDto> {
    let service = GlobalOutputService::with_container(state.inner());

    match service.repair(tool_id, confirm_risk) {
        Ok(result) => AppResponse::success(result),
        Err(error) => {
            record_history_failure(
                None,
                Some(tool_id),
                "repair",
                "Global AGENTS repair failed",
                "global.repair_agents",
                &error,
                Some(ROUTE_PRESETS),
            );
            AppResponse::error(
                "global_output_repair_failed",
                &error,
                "errors.globalOutputRepairFailed",
            )
        }
    }
}

#[tauri::command]
pub fn cleanup_global_output(
    state: tauri::State<'_, AppContainer>,
    tool_id: i32,
    confirm_risk: bool,
) -> AppResponse<GlobalOutputWriteDto> {
    let service = GlobalOutputService::with_container(state.inner());

    match service.cleanup(tool_id, confirm_risk) {
        Ok(result) => AppResponse::success(result),
        Err(error) => {
            record_history_failure(
                None,
                Some(tool_id),
                "operation",
                "Global AGENTS cleanup failed",
                "global.cleanup_agents",
                &error,
                Some(ROUTE_PRESETS),
            );
            AppResponse::error(
                "global_output_cleanup_failed",
                &error,
                "errors.globalOutputCleanupFailed",
            )
        }
    }
}

#[tauri::command]
pub fn save_project_entity(
    app_state: tauri::State<'_, AppContainer>,
    id: Option<i32>,
    name: String,
    path: String,
    project_type: i32,
    import_mode: bool,
) -> AppResponse<ProjectDetailDto> {
    let service = WriteService::with_container(app_state.inner());
    match service.save_project(id, &name, &path, project_type, import_mode) {
        Ok(saved_id) => AppResponse::success(ProjectDetailDto {
            id: saved_id,
            name,
            path,
            project_type,
            rule_bindings: Vec::new(),
            last_operation: String::new(),
            latest_backup: String::new(),
        }),
        Err(error) => {
            record_history_failure(
                None,
                None,
                "operation",
                "Project save failed",
                "project-write",
                &error,
                Some(ROUTE_PROJECTS),
            );
            AppResponse::error("project_save_failed", &error, "errors.projectSaveFailed")
        }
    }
}

#[tauri::command]
pub fn delete_project_entity(
    app_state: tauri::State<'_, AppContainer>,
    project_id: i32,
) -> AppResponse<bool> {
    let service = WriteService::with_container(app_state.inner());
    match service.delete_project(project_id) {
        Ok(()) => AppResponse::success(true),
        Err(error) => {
            record_history_failure(
                Some(project_id),
                None,
                "operation",
                "Project delete failed",
                "project-delete",
                &error,
                Some(ROUTE_PROJECTS),
            );
            AppResponse::error(
                "project_delete_failed",
                &error,
                "errors.projectDeleteFailed",
            )
        }
    }
}

#[tauri::command]
pub fn save_project_rule_bindings(
    app_state: tauri::State<'_, AppContainer>,
    project_id: i32,
    tool_id: Option<i32>,
    rule_ids: Vec<i32>,
) -> AppResponse<bool> {
    let service = WriteService::with_container(app_state.inner());
    match service.replace_project_rule_bindings(project_id, tool_id, &rule_ids) {
        Ok(()) => AppResponse::success(true),
        Err(error) => {
            record_history_failure(
                Some(project_id),
                tool_id,
                "operation",
                "Rule binding update failed",
                "project-rule-bindings",
                &error,
                Some(ROUTE_PROJECTS),
            );
            AppResponse::error(
                "project_rule_bindings_failed",
                &error,
                "errors.projectRuleBindingsFailed",
            )
        }
    }
}

#[tauri::command]
pub fn save_tool_global_rule_bindings(
    app_state: tauri::State<'_, AppContainer>,
    tool_id: i32,
    rule_ids: Vec<i32>,
) -> AppResponse<bool> {
    let service = WriteService::with_container(app_state.inner());
    match service.replace_tool_global_rule_bindings(tool_id, &rule_ids) {
        Ok(()) => AppResponse::success(true),
        Err(error) => {
            record_history_failure(
                None,
                Some(tool_id),
                "operation",
                "Tool global rule binding update failed",
                "tool-global-rule-bindings",
                &error,
                Some(ROUTE_RULES),
            );
            AppResponse::error(
                "tool_global_rule_bindings_failed",
                &error,
                "errors.toolGlobalRuleBindingsFailed",
            )
        }
    }
}

#[tauri::command]
pub fn save_tool_skill_bindings(
    app_state: tauri::State<'_, AppContainer>,
    tool_id: i32,
    skill_ids: Vec<i32>,
) -> AppResponse<bool> {
    let service = WriteService::with_container(app_state.inner());
    match service.replace_tool_skill_bindings(tool_id, &skill_ids) {
        Ok(()) => AppResponse::success(true),
        Err(error) => {
            record_history_failure(
                None,
                Some(tool_id),
                "operation",
                "Tool skill binding update failed",
                "tool-skill-bindings",
                &error,
                Some(ROUTE_SKILLS),
            );
            AppResponse::error(
                "tool_skill_bindings_failed",
                &error,
                "errors.skillSaveFailed",
            )
        }
    }
}

fn record_history_failure(
    project_id: Option<i32>,
    tool_id: Option<i32>,
    kind: &str,
    title: &str,
    action: &str,
    detail: &str,
    navigation_target: Option<&str>,
) {
    record_command_failure(CommandFailure {
        project_id,
        tool_id,
        related_rule_id: None,
        kind,
        title,
        action,
        detail,
        related_path: None,
        navigation_target: navigation_target.unwrap_or(""),
    });
}
