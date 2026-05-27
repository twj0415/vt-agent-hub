use std::fs;
use std::path::Path;

use crate::application::operation_service::OperationService;
use crate::core::validation;
use crate::dto::{RuleImpactDto, RuleImportPreviewDto, RuleImportResultDto, RuleSummaryDto};
use crate::infrastructure::resource_repo::{ResourceRepo, RuleVersionRecord};

use super::{parse_markdown_rule, WriteService};

impl WriteService {
    pub fn save_rule(
        &self,
        id: Option<i32>,
        code: i32,
        name: &str,
        category_code: i32,
        state: i32,
        summary: &str,
        body: &str,
    ) -> Result<i32, String> {
        validation::validate_rule(code, name, category_code, state, body)?;
        let db = self.db.lock().expect("db poisoned");
        let repo = ResourceRepo::new(&db);
        let previous = if let Some(existing_id) = id {
            Some(repo.find_latest_rule_version_by_asset(existing_id)?)
        } else {
            None
        };
        let saved = repo.save_rule_version(
            id,
            &Self::asset_key(name),
            code,
            name,
            category_code,
            previous.as_ref().map(|rule| rule.sort_order).unwrap_or(0),
            state,
            summary,
            body,
        )?;
        OperationService::record(
            &db,
            None,
            "operation",
            if id.is_some() {
                "Rule update"
            } else {
                "Rule create"
            },
            "rule-write",
            if id.is_some() {
                "Updated rule asset in SQLite."
            } else {
                "Created rule asset in SQLite."
            },
        )?;
        Ok(saved.asset_id)
    }

    pub fn preview_rule_impact(&self, id: i32) -> Result<RuleImpactDto, String> {
        let db = self.db.lock().expect("db poisoned");
        let repo = ResourceRepo::new(&db);
        let impact = repo.rule_impact(id)?;
        let requires_project_regeneration = !impact.project_names.is_empty();
        Ok(RuleImpactDto {
            rule_asset_id: impact.rule_asset_id,
            rule_name: impact.rule_name,
            bound_project_count: impact.project_names.len(),
            bound_tool_count: impact.tool_ids.len(),
            project_names: impact.project_names,
            tool_ids: impact.tool_ids,
            project_tool_ids: impact.project_tool_ids,
            global_tool_ids: impact.global_tool_ids,
            requires_project_regeneration,
        })
    }

    pub fn preview_rule_import(&self, source_path: &str) -> Result<RuleImportPreviewDto, String> {
        validation::validate_required_field("rule_import.source_path", source_path)?;
        let source = Path::new(source_path);
        let source_body = fs::read_to_string(source)
            .map_err(|error| format!("Failed to read rule source {}: {error}", source.display()))?;
        let parts = parse_markdown_rule(&source_body);
        Ok(RuleImportPreviewDto {
            source_path: source.display().to_string(),
            name: parts.name,
            summary: parts.description,
            body: parts.body,
        })
    }

    pub fn import_rule(
        &self,
        source_path: &str,
        name: &str,
        category_code: i32,
        summary: &str,
        conflict_strategy: &str,
    ) -> Result<RuleImportResultDto, String> {
        validation::validate_rule_import(source_path, category_code, conflict_strategy)?;

        let source = Path::new(source_path);
        let source_body = fs::read_to_string(source)
            .map_err(|error| format!("Failed to read rule source {}: {error}", source.display()))?;
        let parts = parse_markdown_rule(&source_body);
        let body = parts.body;
        let base_name = name.trim().to_string();
        validation::validate_required_field("rule_import.name", &base_name)?;
        let summary = summary.trim().to_string();
        validation::validate_required_field("rule_import.summary", &summary)?;
        validation::validate_required_field("rule_import.body", &body)?;

        let db = self.db.lock().expect("db poisoned");
        let repo = ResourceRepo::new(&db);
        let existing = repo.find_latest_rule_version_by_name(&base_name)?;

        let (saved_id, operation, warnings) = match (existing, conflict_strategy) {
            (Some(rule), "skip") => (
                rule.asset_id,
                "skipped".to_string(),
                vec![format!(
                    "Rule '{}' already exists; import skipped.",
                    base_name
                )],
            ),
            (Some(rule), "overwrite") => (
                repo.save_rule_version(
                    Some(rule.asset_id),
                    &rule.asset_key,
                    category_code,
                    &base_name,
                    category_code,
                    rule.sort_order,
                    502,
                    &summary,
                    &body,
                )?
                .asset_id,
                "overwritten".to_string(),
                Vec::new(),
            ),
            (Some(_), "rename") => {
                let name = Self::next_import_rule_name_inner(&repo, &base_name)?;
                (
                    repo.save_rule_version(
                        None,
                        &Self::asset_key(&name),
                        category_code,
                        &name,
                        category_code,
                        0,
                        502,
                        &summary,
                        &body,
                    )?
                    .asset_id,
                    "renamed".to_string(),
                    vec![format!(
                        "Rule '{}' already exists; imported as '{}'.",
                        base_name, name
                    )],
                )
            }
            (None, _) => (
                repo.save_rule_version(
                    None,
                    &Self::asset_key(&base_name),
                    category_code,
                    &base_name,
                    category_code,
                    0,
                    502,
                    &summary,
                    &body,
                )?
                .asset_id,
                "created".to_string(),
                Vec::new(),
            ),
            (Some(_), other) => {
                return Err(format!(
                    "Unsupported rule import conflict strategy: {other}"
                ));
            }
        };

        OperationService::record(
            &db,
            None,
            "operation",
            "Rule import",
            "rule-import",
            &format!("Rule import {} from {}.", operation, source.display()),
        )?;

        Ok(RuleImportResultDto {
            rule: Self::rule_to_dto(repo.find_latest_rule_version_by_asset(saved_id)?),
            source_path: source.display().to_string(),
            imported_name: base_name,
            operation,
            warnings,
        })
    }

    pub fn move_rule(
        &self,
        id: i32,
        category_code: i32,
        sort_order: i32,
    ) -> Result<RuleSummaryDto, String> {
        validation::validate_rule_position(category_code, sort_order)?;
        let db = self.db.lock().expect("db poisoned");
        let repo = ResourceRepo::new(&db);
        let before = repo.find_latest_rule_version_by_asset(id)?;
        let after = repo.save_rule_version(
            Some(id),
            &before.asset_key,
            before.code,
            &before.name,
            category_code,
            sort_order,
            before.state,
            &before.summary,
            &before.body,
        )?;
        OperationService::record(
            &db,
            None,
            "operation",
            "Rule move",
            "rule-move",
            &format!(
                "Moved rule '{}' from category {} to {} with order {}.",
                before.name, before.category_code, category_code, sort_order
            ),
        )?;

        Ok(Self::rule_to_dto(after))
    }

    pub fn delete_rule(&self, id: i32) -> Result<(), String> {
        let db = self.db.lock().expect("db poisoned");
        let repo = ResourceRepo::new(&db);
        let impact = repo.rule_impact(id)?;
        if !impact.project_names.is_empty() || !impact.tool_ids.is_empty() {
            return Err(format!(
                "Rule is still bound to {} project(s) and {} tool(s). Unbind it before deleting.",
                impact.project_names.len(),
                impact.tool_ids.len()
            ));
        }
        repo.delete_rule_asset(id)?;
        OperationService::record(
            &db,
            None,
            "operation",
            "Rule delete",
            "rule-delete",
            &format!(
                "Deleted rule asset from SQLite and removed bindings for {} project(s) and {} tool(s).",
                impact.project_names.len(),
                impact.tool_ids.len()
            ),
        )?;
        Ok(())
    }

    fn next_import_rule_name_inner(repo: &ResourceRepo, base_name: &str) -> Result<String, String> {
        for index in 2..1000 {
            let candidate = format!("{base_name} ({index})");
            if repo.find_latest_rule_version_by_name(&candidate)?.is_none() {
                return Ok(candidate);
            }
        }
        Err(format!(
            "No available import name for rule '{}'.",
            base_name
        ))
    }

    fn rule_to_dto(rule: RuleVersionRecord) -> RuleSummaryDto {
        RuleSummaryDto {
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
        }
    }
}
