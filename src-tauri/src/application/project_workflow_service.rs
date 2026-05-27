use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::adapters::tool_adapter::{
    ProjectOutputBuildInput, ProjectOutputRule, ProjectOutputScope,
};
use crate::application::app_container::AppContainer;
use crate::application::operation_service::OperationService;
use crate::application::service_context::ServiceContext;
use crate::application::tool_service::ToolService;
use crate::core::routes::ROUTE_PROJECTS;
use crate::core::status_codes::{TARGET_STATE_ERROR, TARGET_STATE_PLANNED, TARGET_STATE_READY};
use crate::dto::{ProjectOutputPreviewDto, ProjectOutputScanDto, ProjectOutputWriteDto};
use crate::infrastructure::database::Database;
use crate::infrastructure::project_output_repo::{ProjectOutputRepo, ProjectOutputState};
use crate::infrastructure::project_repo::ProjectRepo;
use crate::infrastructure::resource_repo::ResourceRepo;

pub struct ProjectWorkflowService {
    db: Arc<Mutex<Database>>,
    tool_service: ToolService,
}

impl ProjectWorkflowService {
    pub fn new() -> Result<Self, String> {
        Self::with_context(ServiceContext::default()?)
    }

    pub fn with_context(context: ServiceContext) -> Result<Self, String> {
        let db = context.open_db()?;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            tool_service: ToolService::new(),
        })
    }

    pub fn with_container(container: &AppContainer) -> Self {
        Self {
            db: container.db(),
            tool_service: ToolService::new(),
        }
    }

    pub fn with_db(db: Database) -> Self {
        Self {
            db: Arc::new(Mutex::new(db)),
            tool_service: ToolService::new(),
        }
    }

    pub fn with_db_arc(db: Arc<Mutex<Database>>) -> Self {
        Self {
            db,
            tool_service: ToolService::new(),
        }
    }

    pub fn scan(&self, project_id: i32, tool_id: i32) -> Result<ProjectOutputScanDto, String> {
        let project = {
            let db = self.db.lock().expect("db poisoned");
            ProjectRepo::new(&db).find(project_id)?
        };
        let project_path = PathBuf::from(&project.path);
        let project_path_exists = project_path.is_dir();
        let target_path = self
            .tool_service
            .project_output_target_path(tool_id, &project.path)?;
        let managed_marker = self.tool_service.project_output_managed_marker(tool_id)?;
        let output_state = ProjectOutputRepo::inspect(&target_path, managed_marker)?;
        let rules = self.resolve_project_rules(project_id, tool_id)?;
        let mut issues = Vec::new();

        if rules.is_empty() {
            issues.push("no_bound_rules".to_string());
        }

        if !project_path_exists {
            issues.push("project_path_missing".to_string());
        }

        if output_state.target_exists && !output_state.managed {
            issues.push("unmanaged_existing".to_string());
        }

        if !output_state.target_exists {
            issues.push("missing_target".to_string());
        }

        let status = if !project_path_exists {
            "missing"
        } else if rules.is_empty() {
            "blocked"
        } else if output_state.target_exists && !output_state.managed {
            "attention"
        } else if output_state.target_exists {
            "ready"
        } else {
            "planned"
        };

        Ok(ProjectOutputScanDto {
            project_id,
            tool_id,
            project_name: project.name,
            target_path: output_state.target_path.display().to_string(),
            target_exists: output_state.target_exists,
            managed: output_state.managed,
            rule_count: rules.len(),
            status: status.to_string(),
            status_code: if !project_path_exists
                || rules.is_empty()
                || (output_state.target_exists && !output_state.managed)
            {
                TARGET_STATE_ERROR
            } else if output_state.target_exists {
                TARGET_STATE_READY
            } else {
                TARGET_STATE_PLANNED
            },
            issues,
        })
    }

    pub fn preview(
        &self,
        project_id: i32,
        tool_id: i32,
    ) -> Result<ProjectOutputPreviewDto, String> {
        let project = {
            let db = self.db.lock().expect("db poisoned");
            ProjectRepo::new(&db).find(project_id)?
        };
        let target_path = self
            .tool_service
            .project_output_target_path(tool_id, &project.path)?;
        let managed_marker = self.tool_service.project_output_managed_marker(tool_id)?;
        let output_state = ProjectOutputRepo::inspect(&target_path, managed_marker)?;
        let rules = self.resolve_project_rules(project_id, tool_id)?;
        let can_sync_empty_rules = output_state.target_exists && output_state.managed;

        if rules.is_empty() && !can_sync_empty_rules {
            return Err("Project has no bound rules for the current tool view.".to_string());
        }

        let after_content = self.build_agents_content(tool_id, &project.name, &rules)?;
        let issues = if output_state.target_exists && !output_state.managed {
            vec!["unmanaged_existing".to_string()]
        } else {
            Vec::new()
        };
        let warning = if output_state.target_exists && !output_state.managed {
            Some(
                "Target AGENTS.md is not managed and requires repair flow confirmation."
                    .to_string(),
            )
        } else {
            Some("Applying will overwrite the live AGENTS.md after confirmation and optional backup.".to_string())
        };

        let db = self.db.lock().expect("db poisoned");
        OperationService::record_simple(
            &db,
            Some(project_id),
            Some(tool_id),
            None,
            "operation",
            "Workspace preview",
            "project-preview",
            "Previewed AGENTS.md diff for the current project workflow.",
            "success",
            crate::core::status_codes::HEALTH_NORMAL,
            Some(&output_state.target_path.display().to_string()),
            ROUTE_PROJECTS,
        )?;
        drop(db);

        Ok(ProjectOutputPreviewDto {
            project_id,
            tool_id,
            project_name: project.name,
            target_path: output_state.target_path.display().to_string(),
            target_exists: output_state.target_exists,
            managed: output_state.managed,
            rule_count: rules.len(),
            backup_required: output_state.target_exists,
            can_apply: !output_state.target_exists || output_state.managed,
            warning,
            before_content: output_state.content.clone(),
            after_content: after_content.clone(),
            diff: Self::build_diff(&output_state.content, &after_content),
            issues,
        })
    }

    pub fn apply(
        &self,
        project_id: i32,
        tool_id: i32,
        confirm_risk: bool,
    ) -> Result<ProjectOutputWriteDto, String> {
        let preview = self.preview(project_id, tool_id)?;

        if preview.target_exists && !preview.managed {
            return Err(
                "Target AGENTS.md is not a VT Hub Manager managed file. Use repair instead."
                    .to_string(),
            );
        }

        self.write_preview(preview, "project.apply_agents", "operation", confirm_risk)
    }

    pub fn repair(
        &self,
        project_id: i32,
        tool_id: i32,
        confirm_risk: bool,
    ) -> Result<ProjectOutputWriteDto, String> {
        let preview = self.preview(project_id, tool_id)?;
        self.write_preview(preview, "project.repair_agents", "repair", confirm_risk)
    }

    pub fn cleanup(
        &self,
        project_id: i32,
        tool_id: i32,
        confirm_risk: bool,
    ) -> Result<ProjectOutputWriteDto, String> {
        self.remove_output(project_id, tool_id, confirm_risk, "project.cleanup_agents")
    }

    pub fn reset(
        &self,
        project_id: i32,
        tool_id: i32,
        confirm_risk: bool,
    ) -> Result<ProjectOutputWriteDto, String> {
        self.remove_output(project_id, tool_id, confirm_risk, "project.reset_agents")
    }

    fn build_agents_content(
        &self,
        tool_id: i32,
        project_name: &str,
        rules: &[ProjectOutputRule],
    ) -> Result<String, String> {
        self.tool_service.render_project_output(
            tool_id,
            &ProjectOutputBuildInput {
                project_name: project_name.to_string(),
                scope: ProjectOutputScope::Project,
                rules: rules.to_vec(),
            },
        )
    }

    fn resolve_project_rules(
        &self,
        project_id: i32,
        tool_id: i32,
    ) -> Result<Vec<ProjectOutputRule>, String> {
        let db = self.db.lock().expect("db poisoned");
        let resource_repo = ResourceRepo::new(&db);
        let bindings = resource_repo.project_rule_bindings(project_id)?;
        let mut rules = Vec::new();
        let mut seen_rule_asset_ids = Vec::new();

        for binding in bindings {
            if !binding.enabled {
                continue;
            }
            if binding.tool_id.is_some() && binding.tool_id != Some(tool_id) {
                continue;
            }

            for item in binding.items {
                if item.item_type != "rule" {
                    continue;
                }
                let rule = resource_repo.find_latest_rule_version_by_asset(item.asset_id)?;
                if seen_rule_asset_ids.contains(&rule.asset_id) {
                    continue;
                }
                seen_rule_asset_ids.push(rule.asset_id);
                rules.push(ProjectOutputRule {
                    id: rule.asset_id,
                    version_no: rule.version_no,
                    code: rule.code,
                    category_code: rule.category_code,
                    sort_order: item.sort_order,
                    name: rule.name,
                    body: rule.body,
                });
            }
        }

        Ok(rules)
    }

    fn write_preview(
        &self,
        preview: ProjectOutputPreviewDto,
        operation: &str,
        history_kind: &str,
        confirm_risk: bool,
    ) -> Result<ProjectOutputWriteDto, String> {
        if !confirm_risk {
            return Err("Risk confirmation is required before writing AGENTS.md.".to_string());
        }

        let target_path = PathBuf::from(&preview.target_path);

        let backup_path = if preview.target_exists {
            Some(ProjectOutputRepo::backup(preview.project_id, &target_path)?)
        } else {
            None
        };

        ProjectOutputRepo::write(&target_path, &preview.after_content)?;

        let db = self.db.lock().expect("db poisoned");
        if let Some(path) = &backup_path {
            OperationService::record_simple(
                &db,
                Some(preview.project_id),
                Some(preview.tool_id),
                None,
                "backup",
                "AGENTS.md backup",
                "project-backup",
                &path,
                "success",
                crate::core::status_codes::HEALTH_NORMAL,
                Some(path),
                ROUTE_PROJECTS,
            )?;
        }

        OperationService::record_simple(
            &db,
            Some(preview.project_id),
            Some(preview.tool_id),
            None,
            history_kind,
            if history_kind == "repair" {
                "Workspace repair"
            } else {
                "Workspace apply"
            },
            operation,
            if history_kind == "repair" {
                "Repaired project AGENTS.md from current rule bindings."
            } else {
                "Applied project AGENTS.md from current rule bindings."
            },
            "success",
            crate::core::status_codes::HEALTH_NORMAL,
            Some(&preview.target_path),
            ROUTE_PROJECTS,
        )?;

        Ok(ProjectOutputWriteDto {
            project_id: preview.project_id,
            tool_id: preview.tool_id,
            operation: operation.to_string(),
            target_path: preview.target_path,
            backup_path,
            managed: true,
            created: !preview.target_exists,
            message: if history_kind == "repair" {
                "Project AGENTS.md repaired."
            } else {
                "Project AGENTS.md applied."
            }
            .to_string(),
        })
    }

    fn remove_output(
        &self,
        project_id: i32,
        tool_id: i32,
        confirm_risk: bool,
        operation: &str,
    ) -> Result<ProjectOutputWriteDto, String> {
        if !confirm_risk {
            return Err("Risk confirmation is required before deleting AGENTS.md.".to_string());
        }

        let project = {
            let db = self.db.lock().expect("db poisoned");
            ProjectRepo::new(&db).find(project_id)?
        };
        let output_state = self.inspect_output(project_id, tool_id, &project.path)?;

        if !output_state.target_exists {
            return Err(
                "Project AGENTS.md does not exist. Scan or apply before cleanup/reset.".to_string(),
            );
        }

        if !output_state.managed {
            return Err("Target AGENTS.md is not a VT Hub Manager managed file. Use repair instead of cleanup/reset.".to_string());
        }

        let backup_path = Some(ProjectOutputRepo::backup(
            project_id,
            &output_state.target_path,
        )?);
        ProjectOutputRepo::delete(&output_state.target_path)?;

        let db = self.db.lock().expect("db poisoned");
        if let Some(path) = &backup_path {
            OperationService::record_simple(
                &db,
                Some(project_id),
                Some(tool_id),
                None,
                "backup",
                "AGENTS.md backup",
                "project-backup",
                path,
                "success",
                crate::core::status_codes::HEALTH_NORMAL,
                Some(path),
                ROUTE_PROJECTS,
            )?;
        }

        let detail = if operation == "project.reset_agents" {
            "Removed managed project AGENTS.md and returned the project output to an unmanaged state."
        } else {
            "Removed managed project AGENTS.md from the current project output."
        };

        OperationService::record_simple(
            &db,
            Some(project_id),
            Some(tool_id),
            None,
            "operation",
            if operation == "project.reset_agents" {
                "Project AGENTS reset"
            } else {
                "Project AGENTS cleanup"
            },
            operation,
            detail,
            "success",
            crate::core::status_codes::HEALTH_NORMAL,
            Some(&output_state.target_path.display().to_string()),
            ROUTE_PROJECTS,
        )?;

        Ok(ProjectOutputWriteDto {
            project_id,
            tool_id,
            operation: operation.to_string(),
            target_path: output_state.target_path.display().to_string(),
            backup_path,
            managed: false,
            created: false,
            message: if operation == "project.reset_agents" {
                "Project AGENTS.md reset to unmanaged state."
            } else {
                "Project AGENTS.md removed."
            }
            .to_string(),
        })
    }

    fn inspect_output(
        &self,
        project_id: i32,
        tool_id: i32,
        project_path: &str,
    ) -> Result<ProjectOutputState, String> {
        let _ = project_id;
        let target_path = self
            .tool_service
            .project_output_target_path(tool_id, project_path)?;
        let managed_marker = self.tool_service.project_output_managed_marker(tool_id)?;
        ProjectOutputRepo::inspect(&target_path, managed_marker)
    }

    fn build_diff(before: &str, after: &str) -> String {
        format!(
            "--- existing output\n{}\n+++ generated output\n{}",
            before, after
        )
    }
}
