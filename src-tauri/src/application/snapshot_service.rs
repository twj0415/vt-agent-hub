use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::application::app_container::AppContainer;
use crate::application::project_workflow_service::ProjectWorkflowService;
use crate::application::service_context::ServiceContext;
use crate::application::skill_runtime_service::SkillRuntimeService;
use crate::core::library_layout::{LIBRARY_AREAS, PROJECT_OUTPUT_NOTE};
use crate::core::paths::STORAGE_ROOT_OVERRIDE_ENV;
use crate::core::status_codes::{HEALTH_ATTENTION, HEALTH_NORMAL, TARGET_STATE_ERROR};
use crate::core::truth_source::{TruthSourceKind, TRUTH_SOURCE_BOUNDARIES};
use crate::dto::{
    CatalogSnapshotDto, HistoryEntryDto, HistoryFilterDto, HistorySnapshotDto, ProjectDetailDto,
    ProjectRulePackBindingDto, ProjectRulePackItemDto, RuleSummaryDto, SettingItemDto,
    SettingsPathDto, SettingsSnapshotDto, SettingsTruthSourceDto, SkillSummaryDto,
    ToolRulePackBindingDto, ToolSkillInstallDto, ToolSnapshotDto, ToolsSnapshotDto,
    WorkspaceProjectDto, WorkspaceSnapshotDto,
};
use crate::infrastructure::database::Database;
use crate::infrastructure::history_repo::HistoryRepo;
use crate::infrastructure::project_repo::ProjectRepo;
use crate::infrastructure::resource_repo::ResourceRepo;
use crate::infrastructure::settings_repo::SettingsRepo;
use crate::infrastructure::tool_repo::ToolRepo;

pub struct SnapshotService {
    context: ServiceContext,
    db: Option<Arc<Mutex<Database>>>,
}

impl SnapshotService {
    pub fn new() -> Result<Self, String> {
        Ok(Self::with_context(ServiceContext::default()?))
    }

    pub fn with_context(context: ServiceContext) -> Self {
        Self { context, db: None }
    }

    pub fn with_container(container: &AppContainer) -> Self {
        Self {
            context: container.context().clone(),
            db: Some(container.db()),
        }
    }

    fn open_db(&self) -> Result<Database, String> {
        self.context.open_db()
    }

    pub fn get_workspace_snapshot(&self) -> Result<WorkspaceSnapshotDto, String> {
        struct RowSeed {
            project: crate::infrastructure::project_repo::ProjectRecord,
            rule_bindings: Vec<ProjectRulePackBindingDto>,
            last_operation: String,
            latest_backup: String,
        }

        let (active_tool_id, seeds) = {
            let db_guard;
            let db_owned;
            let db = if let Some(shared) = &self.db {
                db_guard = shared.lock().expect("db poisoned");
                &*db_guard
            } else {
                db_owned = self.open_db()?;
                &db_owned
            };
            let project_repo = ProjectRepo::new(db);
            let resource_repo = ResourceRepo::new(db);
            let history_repo = HistoryRepo::new(db);
            let active_tool_id = ToolRepo::new(db)
                .list()?
                .into_iter()
                .find(|tool| tool.enabled)
                .map(|tool| tool.id)
                .unwrap_or_default();
            let seeds = project_repo
                .list()?
                .into_iter()
                .map(|project| {
                    let last_operation = history_repo
                        .latest_for_project(project.id, "any")?
                        .map(|entry| entry.detail)
                        .unwrap_or_else(|| "No operation yet".to_string());
                    let latest_backup = history_repo
                        .latest_for_project(project.id, "backup")?
                        .map(|entry| entry.title)
                        .unwrap_or_else(|| "No backup yet".to_string());
                    let rule_bindings = resource_repo
                        .project_rule_bindings(project.id)?
                        .into_iter()
                        .map(|binding| {
                            Ok(ProjectRulePackBindingDto {
                                tool_id: binding.tool_id,
                                pack_id: binding.pack_id,
                                pack_name: binding.pack_name,
                                pack_type: binding.pack_type,
                                pack_version_id: binding.pack_version_id,
                                pack_version_no: binding.pack_version_no,
                                update_policy: binding.update_policy,
                                enabled: binding.enabled,
                                items: binding
                                    .items
                                    .into_iter()
                                    .map(|item| ProjectRulePackItemDto {
                                        item_type: item.item_type,
                                        asset_id: item.asset_id,
                                        asset_version_id: item.asset_version_id,
                                        asset_version_no: item.asset_version_no,
                                        sort_order: item.sort_order,
                                        required: item.required,
                                    })
                                    .collect(),
                            })
                        })
                        .collect::<Result<Vec<_>, String>>()?;
                    Ok(RowSeed {
                        project,
                        rule_bindings,
                        last_operation,
                        latest_backup,
                    })
                })
                .collect::<Result<Vec<_>, String>>()?;
            (active_tool_id, seeds)
        };

        let workflow_service = if let Some(shared) = &self.db {
            ProjectWorkflowService::with_db_arc(shared.clone())
        } else {
            ProjectWorkflowService::with_context(self.context.clone())?
        };
        let rows = seeds
            .into_iter()
            .map(|seed| {
                let rule_count = seed
                    .rule_bindings
                    .iter()
                    .flat_map(|binding| binding.items.iter())
                    .filter(|item| item.item_type == "rule")
                    .count();
                let output_scan = workflow_service
                    .scan(seed.project.id, active_tool_id)
                    .unwrap_or_else(|error| crate::dto::ProjectOutputScanDto {
                        project_id: seed.project.id,
                        tool_id: active_tool_id,
                        project_name: seed.project.name.clone(),
                        target_path: String::new(),
                        target_exists: false,
                        managed: false,
                        rule_count,
                        status: "error".to_string(),
                        status_code: TARGET_STATE_ERROR,
                        issues: vec![error],
                    });
                WorkspaceProjectDto {
                    id: seed.project.id,
                    name: seed.project.name,
                    path: seed.project.path,
                    project_type: seed.project.project_type,
                    rule_bindings: seed.rule_bindings,
                    last_operation: seed.last_operation,
                    latest_backup: seed.latest_backup,
                    output_scan: Some(output_scan),
                }
            })
            .collect::<Vec<_>>();

        Ok(WorkspaceSnapshotDto {
            active_project_id: rows.first().map(|project| project.id),
            active_tool_id,
            projects: rows,
        })
    }

    pub fn get_catalog_snapshot(&self) -> Result<CatalogSnapshotDto, String> {
        let db_guard;
        let db_owned;
        let db = if let Some(shared) = &self.db {
            db_guard = shared.lock().expect("db poisoned");
            &*db_guard
        } else {
            db_owned = self.open_db()?;
            &db_owned
        };
        let resource_repo = ResourceRepo::new(db);
        let skill_runtime = if let Some(shared) = &self.db {
            SkillRuntimeService::with_db_arc(shared.clone(), self.context.clone())
        } else {
            SkillRuntimeService::with_context(self.context.clone())?
        };

        Ok(CatalogSnapshotDto {
            rules: resource_repo
                .list_latest_rule_versions()?
                .into_iter()
                .map(|rule| RuleSummaryDto {
                    asset_id: rule.asset_id,
                    version_id: rule.version_id,
                    version_no: rule.version_no,
                    key: rule.asset_key,
                    code: rule.code,
                    name: rule.name,
                    category_code: rule.category_code,
                    state: rule.state,
                    sort_order: rule.sort_order,
                    summary: rule.summary,
                    body: rule.body,
                })
                .collect(),
            skills: {
                let mut skill_tool_ids = HashMap::<i32, Vec<i32>>::new();
                for (skill_asset_id, tool_id) in resource_repo.skill_tool_bindings()? {
                    skill_tool_ids
                        .entry(skill_asset_id)
                        .or_default()
                        .push(tool_id);
                }

                resource_repo
                    .list_latest_skill_versions()?
                    .into_iter()
                    .map(|skill| {
                        let runtime = skill_runtime.inspect_skill_by_name(&skill.name, &skill.body);
                        let tool_ids = skill_tool_ids.remove(&skill.asset_id).unwrap_or_default();
                        SkillSummaryDto {
                            asset_id: skill.asset_id,
                            version_id: skill.version_id,
                            version_no: skill.version_no,
                            key: skill.asset_key,
                            code: skill.code,
                            name: skill.name,
                            category_code: skill.category_code,
                            state: skill.state,
                            summary: skill.summary,
                            body: skill.body,
                            runtime,
                            tool_ids,
                        }
                    })
                    .collect()
            },
        })
    }

    pub fn get_tools_snapshot(&self, tool_id: Option<i32>) -> Result<ToolsSnapshotDto, String> {
        let db_guard;
        let db_owned;
        let db = if let Some(shared) = &self.db {
            db_guard = shared.lock().expect("db poisoned");
            &*db_guard
        } else {
            db_owned = self.open_db()?;
            &db_owned
        };
        let resource_repo = ResourceRepo::new(db);
        let rows = ToolRepo::new(db)
            .list()?
            .into_iter()
            .map(|tool| ToolSnapshotDto {
                id: tool.id,
                name: tool.name,
                enabled: tool.enabled,
            })
            .collect::<Vec<_>>();
        let selected_tool_id = tool_id
            .filter(|id| rows.iter().any(|tool| tool.id == *id && tool.enabled))
            .or_else(|| rows.iter().find(|tool| tool.enabled).map(|tool| tool.id));
        let global_rule_binding = if let Some(selected_tool_id) = selected_tool_id {
            resource_repo
                .tool_global_rule_binding(selected_tool_id)?
                .map(|binding| {
                    Ok::<ToolRulePackBindingDto, String>(ToolRulePackBindingDto {
                        pack_id: binding.pack_id,
                        pack_name: binding.pack_name,
                        pack_type: binding.pack_type,
                        pack_version_id: binding.pack_version_id,
                        pack_version_no: binding.pack_version_no,
                        update_policy: binding.update_policy,
                        enabled: binding.enabled,
                        items: binding
                            .items
                            .into_iter()
                            .map(|item| ProjectRulePackItemDto {
                                item_type: item.item_type,
                                asset_id: item.asset_id,
                                asset_version_id: item.asset_version_id,
                                asset_version_no: item.asset_version_no,
                                sort_order: item.sort_order,
                                required: item.required,
                            })
                            .collect(),
                    })
                })
                .transpose()?
        } else {
            None
        };
        let skill_pack_binding = if let Some(selected_tool_id) = selected_tool_id {
            resource_repo
                .tool_skill_binding(selected_tool_id)?
                .map(|binding| {
                    Ok::<ToolRulePackBindingDto, String>(ToolRulePackBindingDto {
                        pack_id: binding.pack_id,
                        pack_name: binding.pack_name,
                        pack_type: binding.pack_type,
                        pack_version_id: binding.pack_version_id,
                        pack_version_no: binding.pack_version_no,
                        update_policy: binding.update_policy,
                        enabled: binding.enabled,
                        items: binding
                            .items
                            .into_iter()
                            .map(|item| ProjectRulePackItemDto {
                                item_type: item.item_type,
                                asset_id: item.asset_id,
                                asset_version_id: item.asset_version_id,
                                asset_version_no: item.asset_version_no,
                                sort_order: item.sort_order,
                                required: item.required,
                            })
                            .collect(),
                    })
                })
                .transpose()?
        } else {
            None
        };
        let skill_installs = if let Some(selected_tool_id) = selected_tool_id {
            resource_repo
                .list_tool_skill_installs(selected_tool_id)?
                .into_iter()
                .map(|install| ToolSkillInstallDto {
                    skill_asset_id: install.skill_asset_id,
                    required_version_id: install.required_version_id,
                    installed_version_id: install.installed_version_id,
                    state: install.state,
                    updated_at: install.updated_at,
                })
                .collect()
        } else {
            Vec::new()
        };

        Ok(ToolsSnapshotDto {
            tools: rows,
            global_rule_binding,
            skill_pack_binding,
            skill_installs,
        })
    }

    pub fn get_project_detail(&self, project_id: i32) -> Result<ProjectDetailDto, String> {
        let db_guard;
        let db_owned;
        let db = if let Some(shared) = &self.db {
            db_guard = shared.lock().expect("db poisoned");
            &*db_guard
        } else {
            db_owned = self.open_db()?;
            &db_owned
        };
        let project_repo = ProjectRepo::new(db);
        let resource_repo = ResourceRepo::new(db);
        let history_repo = HistoryRepo::new(db);
        let project = project_repo.find(project_id)?;
        let rule_bindings = resource_repo
            .project_rule_bindings(project_id)?
            .into_iter()
            .map(|binding| {
                Ok(ProjectRulePackBindingDto {
                    tool_id: binding.tool_id,
                    pack_id: binding.pack_id,
                    pack_name: binding.pack_name,
                    pack_type: binding.pack_type,
                    pack_version_id: binding.pack_version_id,
                    pack_version_no: binding.pack_version_no,
                    update_policy: binding.update_policy,
                    enabled: binding.enabled,
                    items: binding
                        .items
                        .into_iter()
                        .map(|item| ProjectRulePackItemDto {
                            item_type: item.item_type,
                            asset_id: item.asset_id,
                            asset_version_id: item.asset_version_id,
                            asset_version_no: item.asset_version_no,
                            sort_order: item.sort_order,
                            required: item.required,
                        })
                        .collect(),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        let last_operation = history_repo
            .latest_for_project(project_id, "any")?
            .map(|entry| entry.detail)
            .unwrap_or_else(|| "No operation yet".to_string());
        let latest_backup = history_repo
            .latest_for_project(project_id, "backup")?
            .map(|entry| entry.title)
            .unwrap_or_else(|| "No backup yet".to_string());

        Ok(ProjectDetailDto {
            id: project.id,
            name: project.name,
            path: project.path,
            project_type: project.project_type,
            rule_bindings,
            last_operation,
            latest_backup,
        })
    }

    pub fn get_history_snapshot(&self) -> Result<HistorySnapshotDto, String> {
        let db_guard;
        let db_owned;
        let db = if let Some(shared) = &self.db {
            db_guard = shared.lock().expect("db poisoned");
            &*db_guard
        } else {
            db_owned = self.open_db()?;
            &db_owned
        };
        let rows = HistoryRepo::new(db)
            .list()?
            .into_iter()
            .map(|entry| {
                let kind = entry.kind;
                let created_at = entry.created_at;
                let detail = entry.detail;
                let result = entry.result;
                let result_code = entry.result_code;

                HistoryEntryDto {
                    id: entry.id,
                    project_id: entry.project_id,
                    tool_id: entry.tool_id,
                    related_rule_id: entry.related_rule_id,
                    kind: kind.clone(),
                    title: entry.title,
                    created_at: created_at.clone(),
                    action: entry.action,
                    result: result.clone(),
                    level: if result_code == HEALTH_NORMAL {
                        "healthy".to_string()
                    } else if result_code == HEALTH_ATTENTION {
                        "attention".to_string()
                    } else {
                        "warning".to_string()
                    },
                    level_code: result_code,
                    detail: if detail.is_empty() {
                        format!("{kind} @ {created_at}")
                    } else {
                        detail
                    },
                    related_path: entry.related_path,
                    navigation_target: entry.navigation_target,
                }
            })
            .collect::<Vec<_>>();

        let mut project_ids = rows
            .iter()
            .filter_map(|entry| entry.project_id)
            .collect::<Vec<_>>();
        project_ids.sort_unstable();
        project_ids.dedup();

        let mut tool_ids = rows
            .iter()
            .filter_map(|entry| entry.tool_id)
            .collect::<Vec<_>>();
        tool_ids.sort_unstable();
        tool_ids.dedup();

        let mut kinds = rows
            .iter()
            .map(|entry| entry.kind.clone())
            .collect::<Vec<_>>();
        kinds.sort();
        kinds.dedup();

        let mut results = rows
            .iter()
            .map(|entry| entry.result.clone())
            .collect::<Vec<_>>();
        results.sort();
        results.dedup();

        Ok(HistorySnapshotDto {
            entries: rows,
            filters: HistoryFilterDto {
                project_ids,
                tool_ids,
                kinds,
                results,
            },
        })
    }

    pub fn get_settings_snapshot(&self) -> Result<SettingsSnapshotDto, String> {
        let db_guard;
        let db_owned;
        let db = if let Some(shared) = &self.db {
            db_guard = shared.lock().expect("db poisoned");
            &*db_guard
        } else {
            db_owned = self.open_db()?;
            &db_owned
        };
        let rows = SettingsRepo::new(db)
            .list()?
            .into_iter()
            .map(|setting| SettingItemDto {
                id: setting.id,
                name: setting.key,
                value: setting.value,
            })
            .collect();

        let storage_root = self.context.storage_root()?;
        let library_root = self.context.library_root()?;
        let paths = vec![
            SettingsPathDto {
                key: "storage_root".to_string(),
                path: storage_root.display().to_string(),
                note: format!(
                    "App data root for the current user. Override with {STORAGE_ROOT_OVERRIDE_ENV} when a custom data location is required."
                )
                    .to_string(),
            },
            SettingsPathDto {
                key: "app_db".to_string(),
                path: self.context.db_path().display().to_string(),
                note: "SQLite truth source for entities, bindings, settings, and operation history."
                    .to_string(),
            },
            SettingsPathDto {
                key: "library_root".to_string(),
                path: library_root.display().to_string(),
                note: "Skill library root used for local Skill source files.".to_string(),
            },
            SettingsPathDto {
                key: "backups".to_string(),
                path: storage_root.join("backups").display().to_string(),
                note: "All managed backups are written here instead of scattering into project directories."
                    .to_string(),
            },
            SettingsPathDto {
                key: "logs".to_string(),
                path: storage_root.join("logs").display().to_string(),
                note: "Diagnostics and runtime logs are exported here.".to_string(),
            },
            SettingsPathDto {
                key: "snapshots".to_string(),
                path: storage_root.join("snapshots").display().to_string(),
                note: "Exported diagnostics snapshots and reports are stored here.".to_string(),
            },
            SettingsPathDto {
                key: "runtime".to_string(),
                path: storage_root.join("runtime").display().to_string(),
                note: "Reserved runtime working area for tool-specific operations.".to_string(),
            },
        ];

        let library_paths = LIBRARY_AREAS.iter().map(|area| SettingsPathDto {
            key: format!("library_{}", area.key),
            path: library_root.join(area.relative_path).display().to_string(),
            note: area.responsibility.to_string(),
        });

        let truth_sources = TRUTH_SOURCE_BOUNDARIES
            .iter()
            .map(|boundary| SettingsTruthSourceDto {
                key: boundary.key.to_string(),
                canonical: truth_source_kind_label(boundary.canonical).to_string(),
                mirrors: boundary
                    .mirrors
                    .iter()
                    .map(|kind| truth_source_kind_label(*kind).to_string())
                    .collect(),
                note: boundary.note.to_string(),
            })
            .collect();

        let mut all_paths = paths;
        all_paths.extend(library_paths);
        all_paths.push(SettingsPathDto {
            key: "project_output".to_string(),
            path: "Target project directory".to_string(),
            note: PROJECT_OUTPUT_NOTE.to_string(),
        });

        Ok(SettingsSnapshotDto {
            items: rows,
            paths: all_paths,
            truth_sources,
        })
    }
}

fn truth_source_kind_label(kind: TruthSourceKind) -> &'static str {
    match kind {
        TruthSourceKind::Sqlite => "sqlite",
        TruthSourceKind::FileSystem => "filesystem",
        TruthSourceKind::SecureStorage => "secure_storage",
    }
}
