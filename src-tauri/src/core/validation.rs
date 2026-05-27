use std::path::Path;

use crate::core::status_codes::{
    SKILL_INSTALL_INSTALLED, SKILL_INSTALL_NOT_INSTALLED, SKILL_INSTALL_SOURCE_MISSING,
    SKILL_INSTALL_STALE, TARGET_STATE_ERROR, TARGET_STATE_MISSING, TARGET_STATE_PLANNED,
    TARGET_STATE_READY,
};
use crate::core::taxonomy;
use crate::core::tool_registry::get_tool;

pub fn validate_project(
    name: &str,
    path: &str,
    project_type: i32,
    import_mode: bool,
) -> Result<(), String> {
    validate_required("project.name", name)?;
    validate_required("project.path", path)?;
    validate_taxonomy_code(
        "project.project_type",
        project_type,
        taxonomy::is_project_type_code,
    )?;

    let path_ref = Path::new(path);
    if import_mode {
        if !path_ref.exists() {
            return Err("project.path must exist when importing a project.".to_string());
        }
        if !path_ref.is_dir() {
            return Err("project.path must be a directory when importing a project.".to_string());
        }
    }

    Ok(())
}

pub fn validate_rule(
    code: i32,
    name: &str,
    category_code: i32,
    state: i32,
    body: &str,
) -> Result<(), String> {
    validate_required("rule.name", name)?;
    validate_required("rule.body", body)?;
    validate_taxonomy_code("rule.code", code, taxonomy::is_rule_category_code)?;
    validate_taxonomy_code(
        "rule.category_code",
        category_code,
        taxonomy::is_rule_category_code,
    )?;
    validate_code(
        "rule.state",
        state,
        &[
            TARGET_STATE_MISSING,
            TARGET_STATE_READY,
            TARGET_STATE_ERROR,
            TARGET_STATE_PLANNED,
        ],
    )
}

pub fn validate_rule_import(
    source_path: &str,
    category_code: i32,
    conflict_strategy: &str,
) -> Result<(), String> {
    validate_required("rule_import.source_path", source_path)?;
    validate_taxonomy_code(
        "rule_import.category_code",
        category_code,
        taxonomy::is_rule_category_code,
    )?;
    if !["skip", "rename", "overwrite"].contains(&conflict_strategy) {
        return Err("rule_import.conflict_strategy is not supported.".to_string());
    }

    let path_ref = Path::new(source_path);
    if !path_ref.exists() {
        return Err("rule_import.source_path does not exist.".to_string());
    }
    if !path_ref.is_file() {
        return Err("rule_import.source_path must be a file.".to_string());
    }
    if !path_ref
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
    {
        return Err("rule_import.source_path must be a markdown file.".to_string());
    }
    Ok(())
}

pub fn validate_rule_position(category_code: i32, sort_order: i32) -> Result<(), String> {
    validate_taxonomy_code(
        "rule.category_code",
        category_code,
        taxonomy::is_rule_category_code,
    )?;
    if sort_order < 0 {
        return Err("rule.sort_order must be zero or greater.".to_string());
    }
    Ok(())
}

pub fn validate_skill(
    code: i32,
    name: &str,
    category_code: i32,
    state: i32,
    install_state: i32,
    body: &str,
) -> Result<(), String> {
    validate_required("skill.name", name)?;
    validate_required("skill.body", body)?;
    validate_taxonomy_code("skill.code", code, taxonomy::is_skill_category_code)?;
    validate_taxonomy_code(
        "skill.category_code",
        category_code,
        taxonomy::is_skill_category_code,
    )?;
    validate_code(
        "skill.state",
        state,
        &[
            TARGET_STATE_MISSING,
            TARGET_STATE_READY,
            TARGET_STATE_ERROR,
            TARGET_STATE_PLANNED,
        ],
    )?;
    validate_code(
        "skill.install_state",
        install_state,
        &[
            SKILL_INSTALL_NOT_INSTALLED,
            SKILL_INSTALL_INSTALLED,
            SKILL_INSTALL_STALE,
            SKILL_INSTALL_SOURCE_MISSING,
        ],
    )
}

pub fn validate_binding_input(
    project_id: i32,
    tool_id: i32,
    rule_ids: &[i32],
) -> Result<(), String> {
    if project_id <= 0 {
        return Err("binding.project_id must be positive.".to_string());
    }
    validate_tool_enabled(tool_id)?;
    if rule_ids.iter().any(|id| *id <= 0) {
        return Err("binding.rule_ids must contain positive IDs.".to_string());
    }
    Ok(())
}

pub fn validate_tool_skill_binding_input(tool_id: i32, skill_ids: &[i32]) -> Result<(), String> {
    if tool_id <= 0 {
        return Err("binding.tool_id must be positive.".to_string());
    }
    validate_tool_enabled(tool_id)?;
    if skill_ids.iter().any(|id| *id <= 0) {
        return Err("binding.skill_ids must contain positive IDs.".to_string());
    }
    Ok(())
}

pub fn validate_project_rule_binding_input(
    project_id: i32,
    tool_id: Option<i32>,
    rule_ids: &[i32],
) -> Result<(), String> {
    if project_id <= 0 {
        return Err("binding.project_id must be positive.".to_string());
    }
    if let Some(tool_id) = tool_id {
        validate_tool_enabled(tool_id)?;
    }
    if rule_ids.iter().any(|id| *id <= 0) {
        return Err("binding.rule_ids must contain positive IDs.".to_string());
    }
    Ok(())
}

pub fn validate_credential(tool_id: i32, token: &str) -> Result<(), String> {
    validate_tool_enabled(tool_id)?;
    validate_required("credential.token", token)?;
    if token.trim().len() < 8 {
        return Err("credential.token is too short for local storage.".to_string());
    }
    Ok(())
}

fn validate_tool_enabled(tool_id: i32) -> Result<(), String> {
    let tool =
        get_tool(tool_id).ok_or_else(|| format!("tool_id {} is not registered.", tool_id))?;
    if !tool.enabled {
        return Err(format!("tool_id {} is not enabled.", tool_id));
    }
    Ok(())
}

fn validate_required(field: &str, value: &str) -> Result<(), String> {
    validate_required_field(field, value)
}

pub fn validate_required_field(field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        return Err(format!("{field} is required."));
    }
    Ok(())
}

fn validate_code(field: &str, value: i32, allowed: &[i32]) -> Result<(), String> {
    if !allowed.contains(&value) {
        return Err(format!("{field} has unsupported code {}.", value));
    }
    Ok(())
}

fn validate_taxonomy_code(
    field: &str,
    value: i32,
    contains: fn(i32) -> bool,
) -> Result<(), String> {
    if !contains(value) {
        return Err(format!("{field} has unsupported code {}.", value));
    }
    Ok(())
}
