use crate::application::app_container::AppContainer;
use crate::application::skill_runtime_service::SkillRuntimeService;
use crate::application::write_service::WriteService;
use crate::core::routes::ROUTE_SKILLS;
use crate::dto::{AppResponse, SkillRuntimeDto, SkillSummaryDto};

#[tauri::command]
pub fn save_skill_asset(
    app_state: tauri::State<'_, AppContainer>,
    id: Option<i32>,
    code: i32,
    name: String,
    category_code: i32,
    state: i32,
    install_state: i32,
    summary: String,
    body: String,
) -> AppResponse<SkillSummaryDto> {
    let service = WriteService::with_container(app_state.inner());
    match service.save_skill(
        id,
        code,
        &name,
        category_code,
        state,
        install_state,
        &summary,
        &body,
    ) {
        Ok(saved_id) => {
            let runtime_service = SkillRuntimeService::with_container(app_state.inner());
            let runtime = match runtime_service.inspect_skill(saved_id) {
                Ok(runtime) => runtime,
                Err(error) => {
                    return AppResponse::error(
                        "skill_save_failed",
                        &error,
                        "errors.skillSaveFailed",
                    )
                }
            };
            AppResponse::success(SkillSummaryDto {
                asset_id: saved_id,
                version_id: 0,
                version_no: 0,
                key: String::new(),
                code,
                name,
                category_code,
                state,
                summary,
                body,
                runtime,
                tool_ids: Vec::new(),
            })
        }
        Err(error) => {
            crate::commands::history_log::record_command_failure(
                crate::commands::history_log::CommandFailure {
                    project_id: None,
                    tool_id: None,
                    related_rule_id: None,
                    kind: "operation",
                    title: "Skill save failed",
                    action: "skill-write",
                    detail: &error,
                    related_path: None,
                    navigation_target: ROUTE_SKILLS,
                },
            );
            AppResponse::error("skill_save_failed", &error, "errors.skillSaveFailed")
        }
    }
}

#[tauri::command]
pub fn delete_skill_asset(
    app_state: tauri::State<'_, AppContainer>,
    skill_id: i32,
) -> AppResponse<bool> {
    wrap_command!(
        service: WriteService::with_container(app_state.inner()),
        call: |service: WriteService| service.delete_skill(skill_id).map(|_| true),
        error_code: "skill_delete_failed",
        i18n: "errors.skillDeleteFailed",
        history: { kind: "operation", title: "Skill delete failed", action: "skill-delete", route: ROUTE_SKILLS },
    )
}

#[tauri::command]
pub fn install_skill_asset(
    app_state: tauri::State<'_, AppContainer>,
    tool_id: i32,
    skill_id: i32,
) -> AppResponse<SkillRuntimeDto> {
    wrap_command!(
        service: SkillRuntimeService::with_container(app_state.inner()),
        call: |service: SkillRuntimeService| service.install_skill_for_tool(tool_id, skill_id),
        error_code: "skill_install_failed",
        i18n: "errors.skillInstallFailed",
    )
}

#[tauri::command]
pub fn uninstall_skill_asset(
    app_state: tauri::State<'_, AppContainer>,
    tool_id: i32,
    skill_id: i32,
) -> AppResponse<SkillRuntimeDto> {
    wrap_command!(
        service: SkillRuntimeService::with_container(app_state.inner()),
        call: |service: SkillRuntimeService| service.uninstall_skill_for_tool(tool_id, skill_id),
        error_code: "skill_uninstall_failed",
        i18n: "errors.skillUninstallFailed",
    )
}

#[tauri::command]
pub fn repair_skill_asset(
    app_state: tauri::State<'_, AppContainer>,
    tool_id: i32,
    skill_id: i32,
) -> AppResponse<SkillRuntimeDto> {
    wrap_command!(
        service: SkillRuntimeService::with_container(app_state.inner()),
        call: |service: SkillRuntimeService| service.repair_skill_for_tool(tool_id, skill_id),
        error_code: "skill_repair_failed",
        i18n: "errors.skillRepairFailed",
    )
}

#[tauri::command]
pub fn mark_skill_asset_stale(
    app_state: tauri::State<'_, AppContainer>,
    tool_id: i32,
    skill_id: i32,
) -> AppResponse<SkillRuntimeDto> {
    wrap_command!(
        service: SkillRuntimeService::with_container(app_state.inner()),
        call: |service: SkillRuntimeService| service.mark_skill_stale_for_tool(tool_id, skill_id),
        error_code: "skill_mark_stale_failed",
        i18n: "errors.skillMarkStaleFailed",
    )
}
