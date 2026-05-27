use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::application::app_container::AppContainer;
use crate::application::service_context::ServiceContext;
use crate::application::tool_service::ToolService;
use crate::core::status_codes::{HEALTH_ATTENTION, HEALTH_NORMAL, HEALTH_WARNING};
use crate::core::tool_registry::TOOL_REGISTRY;
use crate::dto::{LibraryDiagnosticIssueDto, LibraryDiagnosticsDto};
use crate::infrastructure::database::Database;
use crate::infrastructure::library_repo::LibraryRepo;
use crate::infrastructure::project_output_repo::ProjectOutputRepo;
use crate::infrastructure::project_repo::ProjectRepo;
use crate::infrastructure::resource_repo::ResourceRepo;
use crate::infrastructure::settings_repo::SettingsRepo;

pub struct LibraryDiagnosticsService {
    db: Arc<Mutex<Database>>,
    library_root: PathBuf,
}

impl LibraryDiagnosticsService {
    pub fn new() -> Result<Self, String> {
        Self::with_context(ServiceContext::default()?)
    }

    pub fn with_context(context: ServiceContext) -> Result<Self, String> {
        let db = context.open_db()?;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            library_root: context.library_root()?,
        })
    }

    pub fn with_container(container: &AppContainer) -> Result<Self, String> {
        Ok(Self {
            db: container.db(),
            library_root: container.context().library_root()?,
        })
    }

    pub fn scan(&self) -> Result<LibraryDiagnosticsDto, String> {
        let db = self.db.lock().expect("db poisoned");
        let project_repo = ProjectRepo::new(&db);
        let resource_repo = ResourceRepo::new(&db);
        let settings_repo = SettingsRepo::new(&db);
        let tool_service = ToolService::new();
        let projects = project_repo.list()?;
        let rules = resource_repo.list_latest_rule_versions()?;
        let skills = resource_repo.list_latest_skill_versions()?;
        let settings = settings_repo.list()?;

        let mut issues = Vec::new();
        let ensure_result = LibraryRepo::ensure(&self.library_root)?;
        let structure_items = LibraryRepo::inspect(&self.library_root);

        let library_root = settings
            .iter()
            .find(|item| item.key == "library_root")
            .map(|item| item.value.clone());
        if library_root.is_none() {
            issues.push(LibraryDiagnosticIssueDto {
                scope: "settings".to_string(),
                key: "library_root_missing".to_string(),
                level: "warning".to_string(),
                level_code: HEALTH_WARNING,
                detail: "Library root setting is missing from SQLite.".to_string(),
                related_path: None,
            });
        } else if library_root.as_deref() != Some(&self.library_root.display().to_string()) {
            issues.push(LibraryDiagnosticIssueDto {
                scope: "settings".to_string(),
                key: "library_root_mismatch".to_string(),
                level: "warning".to_string(),
                level_code: HEALTH_WARNING,
                detail: "Library root setting does not match the active service context."
                    .to_string(),
                related_path: library_root,
            });
        }

        for item in structure_items {
            if item.status == "ok" {
                continue;
            }

            let level_code = if item.status == "invalid" {
                HEALTH_WARNING
            } else {
                HEALTH_ATTENTION
            };
            issues.push(LibraryDiagnosticIssueDto {
                scope: "library".to_string(),
                key: format!("{}_{}", item.key, item.status),
                level: if item.status == "invalid" {
                    "warning".to_string()
                } else {
                    "attention".to_string()
                },
                level_code,
                detail: item.repair_hint,
                related_path: Some(item.path),
            });
        }

        for project in &projects {
            for tool in TOOL_REGISTRY.iter().filter(|tool| tool.enabled) {
                let tool_id = tool.id;
                let target_path =
                    tool_service.project_output_target_path(tool_id, &project.path)?;
                let managed_marker = tool_service.project_output_managed_marker(tool_id)?;
                let scan = ProjectOutputRepo::inspect(&target_path, managed_marker)?;

                if !scan.target_exists {
                    issues.push(LibraryDiagnosticIssueDto {
                        scope: "project_output".to_string(),
                        key: format!("missing_target_tool_{tool_id}"),
                        level: "warning".to_string(),
                        level_code: HEALTH_WARNING,
                        detail: format!(
                            "Project {} is missing managed output for tool {}.",
                            project.name, tool_id
                        ),
                        related_path: Some(scan.target_path.display().to_string()),
                    });
                } else if !scan.managed {
                    issues.push(LibraryDiagnosticIssueDto {
                        scope: "project_output".to_string(),
                        key: format!("unmanaged_target_tool_{tool_id}"),
                        level: "warning".to_string(),
                        level_code: HEALTH_WARNING,
                        detail: format!(
                            "Project {} has an unmanaged output target for tool {}.",
                            project.name, tool_id
                        ),
                        related_path: Some(scan.target_path.display().to_string()),
                    });
                }
            }

            let bindings = resource_repo.project_rule_bindings(project.id)?;
            if bindings.is_empty() {
                issues.push(LibraryDiagnosticIssueDto {
                    scope: "project_binding".to_string(),
                    key: "missing_rule_bindings".to_string(),
                    level: "attention".to_string(),
                    level_code: HEALTH_ATTENTION,
                    detail: format!("Project {} has no bound project rule pack.", project.name),
                    related_path: Some(project.path.clone()),
                });
            }
        }

        if rules.is_empty() {
            issues.push(LibraryDiagnosticIssueDto {
                scope: "catalog".to_string(),
                key: "rules_empty".to_string(),
                level: "warning".to_string(),
                level_code: HEALTH_WARNING,
                detail: "No rules are currently stored in SQLite.".to_string(),
                related_path: None,
            });
        }

        if skills.is_empty() {
            issues.push(LibraryDiagnosticIssueDto {
                scope: "catalog".to_string(),
                key: "skills_empty".to_string(),
                level: "warning".to_string(),
                level_code: HEALTH_WARNING,
                detail: "No skills are currently stored in SQLite.".to_string(),
                related_path: None,
            });
        }

        let issue_count = issues.len();
        let health_state = if issue_count == 0 {
            "normal".to_string()
        } else if issues.iter().any(|item| item.level == "warning") {
            "warning".to_string()
        } else {
            "attention".to_string()
        };

        Ok(LibraryDiagnosticsDto {
            project_count: projects.len(),
            rule_count: rules.len(),
            skill_count: skills.len(),
            library_root: self.library_root.display().to_string(),
            created_paths: ensure_result.created_paths,
            existing_paths: ensure_result.existing_paths,
            issue_count,
            health_state,
            health_state_code: if issue_count == 0 {
                HEALTH_NORMAL
            } else if issues.iter().any(|item| item.level == "warning") {
                HEALTH_WARNING
            } else {
                HEALTH_ATTENTION
            },
            issues,
        })
    }
}
