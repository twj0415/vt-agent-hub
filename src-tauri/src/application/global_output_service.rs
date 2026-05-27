use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use crate::adapters::tool_adapter::{
    ProjectOutputBuildInput, ProjectOutputRule, ProjectOutputScope,
};
use crate::application::app_container::AppContainer;
use crate::application::operation_service::OperationService;
use crate::application::service_context::ServiceContext;
use crate::application::tool_service::ToolService;
use crate::core::routes::ROUTE_PRESETS;
use crate::core::tool_registry::{CLAUDE_TOOL_ID, CODEX_TOOL_ID, CURSOR_TOOL_ID};
use crate::dto::{GlobalOutputPreviewDto, GlobalOutputWriteDto};
use crate::infrastructure::database::Database;
use crate::infrastructure::project_output_repo::ProjectOutputRepo;
use crate::infrastructure::resource_repo::ResourceRepo;

pub struct GlobalOutputService {
    db: Arc<Mutex<Database>>,
    tool_service: ToolService,
}

#[derive(Debug, Clone)]
struct GlobalOutputDescriptor {
    tool_name: &'static str,
    title_file_label: String,
    tool_file_label: String,
    diff_file_label: String,
    operation_label: String,
}

impl GlobalOutputDescriptor {
    fn for_tool(tool_id: i32, target_path: &Path) -> Self {
        let tool_name = match tool_id {
            CODEX_TOOL_ID => "Codex",
            CLAUDE_TOOL_ID => "Claude",
            CURSOR_TOOL_ID => "Cursor",
            _ => "Tool",
        };
        let file_name = target_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("managed output");
        let operation_name = file_name.strip_suffix(".md").unwrap_or(file_name);

        Self {
            tool_name,
            title_file_label: format!("Global {file_name}"),
            tool_file_label: format!("{tool_name} global {file_name}"),
            diff_file_label: format!("global {file_name}"),
            operation_label: format!("Global {operation_name}"),
        }
    }
}

impl GlobalOutputService {
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

    pub fn preview(&self, tool_id: i32) -> Result<GlobalOutputPreviewDto, String> {
        let rules = self.resolve_global_rules(tool_id)?;
        if rules.is_empty() {
            return Err("Tool global output has no bound rules.".to_string());
        }

        let target_path = self.global_target_path(tool_id)?;
        let descriptor = GlobalOutputDescriptor::for_tool(tool_id, &target_path);
        let managed_marker = self.tool_service.project_output_managed_marker(tool_id)?;
        let output_state = ProjectOutputRepo::inspect(&target_path, managed_marker)?;
        let after_content = self.build_global_content(tool_id, &descriptor, &rules)?;
        let issues = if output_state.target_exists && !output_state.managed {
            vec!["unmanaged_existing_global".to_string()]
        } else {
            Vec::new()
        };

        let db = self.db.lock().expect("db poisoned");
        OperationService::record_simple(
            &db,
            None,
            Some(tool_id),
            None,
            "operation",
            &format!("{} preview", descriptor.title_file_label),
            "global-preview",
            &format!(
                "Previewed {} from tool-global rule bindings.",
                descriptor.tool_file_label
            ),
            "success",
            crate::core::status_codes::HEALTH_NORMAL,
            Some(&output_state.target_path.display().to_string()),
            ROUTE_PRESETS,
        )?;
        drop(db);

        Ok(GlobalOutputPreviewDto {
            tool_id,
            target_path: output_state.target_path.display().to_string(),
            target_exists: output_state.target_exists,
            managed: output_state.managed,
            rule_count: rules.len(),
            backup_required: output_state.target_exists,
            can_apply: !output_state.target_exists || output_state.managed,
            warning: if output_state.target_exists && !output_state.managed {
                Some(format!(
                    "{} is not managed and requires repair confirmation.",
                    descriptor.title_file_label
                ))
            } else {
                Some(format!(
                    "Applying will write {} after confirmation and backup.",
                    descriptor.tool_file_label
                ))
            },
            before_content: output_state.content.clone(),
            after_content: after_content.clone(),
            diff: Self::build_diff(
                &descriptor.diff_file_label,
                &output_state.content,
                &after_content,
            ),
            issues,
        })
    }

    pub fn apply(&self, tool_id: i32, confirm_risk: bool) -> Result<GlobalOutputWriteDto, String> {
        let preview = self.preview(tool_id)?;
        if preview.target_exists && !preview.managed {
            let descriptor =
                GlobalOutputDescriptor::for_tool(tool_id, Path::new(&preview.target_path));
            return Err(format!(
                "{} is not managed. Use repair instead.",
                descriptor.title_file_label
            ));
        }

        self.write_preview(preview, "global.apply_agents", "operation", confirm_risk)
    }

    pub fn repair(&self, tool_id: i32, confirm_risk: bool) -> Result<GlobalOutputWriteDto, String> {
        let preview = self.preview(tool_id)?;
        self.write_preview(preview, "global.repair_agents", "repair", confirm_risk)
    }

    pub fn cleanup(
        &self,
        tool_id: i32,
        confirm_risk: bool,
    ) -> Result<GlobalOutputWriteDto, String> {
        let target_path = self.global_target_path(tool_id)?;
        let descriptor = GlobalOutputDescriptor::for_tool(tool_id, &target_path);
        if !confirm_risk {
            return Err(format!(
                "Risk confirmation is required before deleting {}.",
                descriptor.title_file_label
            ));
        }
        let managed_marker = self.tool_service.project_output_managed_marker(tool_id)?;
        let output_state = ProjectOutputRepo::inspect(&target_path, managed_marker)?;

        if !output_state.target_exists {
            return Ok(GlobalOutputWriteDto {
                tool_id,
                operation: "global.cleanup_agents".to_string(),
                target_path: output_state.target_path.display().to_string(),
                backup_path: None,
                managed: false,
                created: false,
                message: format!("{} is already absent.", descriptor.title_file_label),
            });
        }

        if !output_state.managed {
            return Err(format!(
                "{} is not a VT Hub Manager managed file. Use repair instead of cleanup.",
                descriptor.title_file_label
            ));
        }

        let backup_path = Some(ProjectOutputRepo::backup(0, &output_state.target_path)?);
        ProjectOutputRepo::delete(&output_state.target_path)?;

        let db = self.db.lock().expect("db poisoned");
        if let Some(path) = &backup_path {
            OperationService::record_simple(
                &db,
                None,
                Some(tool_id),
                None,
                "backup",
                &format!("{} backup", descriptor.title_file_label),
                "global-backup",
                path,
                "success",
                crate::core::status_codes::HEALTH_NORMAL,
                Some(path),
                ROUTE_PRESETS,
            )?;
        }

        OperationService::record_simple(
            &db,
            None,
            Some(tool_id),
            None,
            "operation",
            &format!("{} cleanup", descriptor.operation_label),
            "global.cleanup_agents",
            &format!(
                "Removed managed {} from tool-global rule output.",
                descriptor.tool_file_label
            ),
            "success",
            crate::core::status_codes::HEALTH_NORMAL,
            Some(&output_state.target_path.display().to_string()),
            ROUTE_PRESETS,
        )?;

        Ok(GlobalOutputWriteDto {
            tool_id,
            operation: "global.cleanup_agents".to_string(),
            target_path: output_state.target_path.display().to_string(),
            backup_path,
            managed: false,
            created: false,
            message: format!("{} removed.", descriptor.title_file_label),
        })
    }

    fn global_target_path(&self, tool_id: i32) -> Result<PathBuf, String> {
        self.tool_service.global_output_target_path(tool_id)
    }

    fn build_global_content(
        &self,
        tool_id: i32,
        descriptor: &GlobalOutputDescriptor,
        rules: &[ProjectOutputRule],
    ) -> Result<String, String> {
        self.tool_service.render_project_output(
            tool_id,
            &ProjectOutputBuildInput {
                project_name: descriptor.tool_name.to_string(),
                scope: ProjectOutputScope::Tool,
                rules: rules.to_vec(),
            },
        )
    }

    fn resolve_global_rules(&self, tool_id: i32) -> Result<Vec<ProjectOutputRule>, String> {
        let db = self.db.lock().expect("db poisoned");
        let resource_repo = ResourceRepo::new(&db);
        let Some(binding) = resource_repo.tool_global_rule_binding(tool_id)? else {
            return Ok(Vec::new());
        };
        if !binding.enabled {
            return Ok(Vec::new());
        }

        let mut rules = Vec::new();
        for item in binding.items {
            if item.item_type != "rule" {
                continue;
            }
            let rule = resource_repo.find_latest_rule_version_by_asset(item.asset_id)?;
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
        Ok(rules)
    }

    fn write_preview(
        &self,
        preview: GlobalOutputPreviewDto,
        operation: &str,
        history_kind: &str,
        confirm_risk: bool,
    ) -> Result<GlobalOutputWriteDto, String> {
        let target_path = PathBuf::from(&preview.target_path);
        let descriptor = GlobalOutputDescriptor::for_tool(preview.tool_id, &target_path);
        if !confirm_risk {
            return Err(format!(
                "Risk confirmation is required before writing {}.",
                descriptor.title_file_label
            ));
        }

        let backup_path = if preview.target_exists {
            Some(ProjectOutputRepo::backup(0, &target_path)?)
        } else {
            None
        };

        ProjectOutputRepo::write(&target_path, &preview.after_content)?;

        let db = self.db.lock().expect("db poisoned");
        if let Some(path) = &backup_path {
            OperationService::record_simple(
                &db,
                None,
                Some(preview.tool_id),
                None,
                "backup",
                &format!("{} backup", descriptor.title_file_label),
                "global-backup",
                &path,
                "success",
                crate::core::status_codes::HEALTH_NORMAL,
                Some(path),
                ROUTE_PRESETS,
            )?;
        }

        OperationService::record_simple(
            &db,
            None,
            Some(preview.tool_id),
            None,
            history_kind,
            &format!(
                "{} {}",
                descriptor.operation_label,
                if history_kind == "repair" {
                    "repair"
                } else {
                    "apply"
                }
            ),
            operation,
            &format!(
                "{} {} from tool-global rule bindings.",
                if history_kind == "repair" {
                    "Repaired"
                } else {
                    "Applied"
                },
                descriptor.tool_file_label
            ),
            "success",
            crate::core::status_codes::HEALTH_NORMAL,
            Some(&preview.target_path),
            ROUTE_PRESETS,
        )?;

        Ok(GlobalOutputWriteDto {
            tool_id: preview.tool_id,
            operation: operation.to_string(),
            target_path: preview.target_path,
            backup_path,
            managed: true,
            created: !preview.target_exists,
            message: format!(
                "{} {}.",
                descriptor.title_file_label,
                if history_kind == "repair" {
                    "repaired"
                } else {
                    "applied"
                }
            ),
        })
    }

    fn build_diff(file_label: &str, before: &str, after: &str) -> String {
        format!(
            "--- existing {}\n{}\n+++ generated {}\n{}",
            file_label, before, file_label, after
        )
    }
}
