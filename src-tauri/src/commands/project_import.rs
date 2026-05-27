use crate::application::app_container::AppContainer;
use crate::application::project_import_service::ProjectImportService;
use crate::commands::history_log::{record_command_failure, CommandFailure};
use crate::core::routes::ROUTE_PROJECTS;
use crate::dto::{AppResponse, ProjectDetailDto};

#[tauri::command]
pub fn import_project_from_git(
    state: tauri::State<'_, AppContainer>,
    repo_url: String,
    target_path: String,
    name: Option<String>,
    branch: Option<String>,
    project_type: i32,
) -> AppResponse<ProjectDetailDto> {
    let service = ProjectImportService::with_container(state.inner());

    match service.import_from_git(
        &repo_url,
        &target_path,
        name.as_deref(),
        branch.as_deref(),
        project_type,
    ) {
        Ok(project) => AppResponse::success(project),
        Err(error) => {
            record_history_failure(
                "operation",
                "Project import failed",
                "project-import-git",
                &error,
                Some(&repo_url),
            );
            AppResponse::error(
                "project_import_git_failed",
                &error,
                "errors.projectSaveFailed",
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
        navigation_target: ROUTE_PROJECTS,
    });
}
