use std::path::PathBuf;

use crate::domain::tool::Tool;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolDiagnostics {
    pub installation_detected: bool,
    pub version: String,
    pub live_config_path: String,
    pub credential_state: String,
    pub credential_state_code: i32,
    pub skill_state: String,
    pub skill_state_code: i32,
    pub project_output_state: String,
    pub project_output_state_code: i32,
    pub repair_state: String,
    pub repair_state_code: i32,
    pub repair_hint: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolActionResult {
    pub ok: bool,
    pub state: String,
    pub detail: String,
    pub manual_steps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectOutputRule {
    pub id: i32,
    pub version_no: i32,
    pub code: i32,
    pub category_code: i32,
    pub sort_order: i32,
    pub name: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectOutputBuildInput {
    pub project_name: String,
    pub scope: ProjectOutputScope,
    pub rules: Vec<ProjectOutputRule>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PresetConfigBuildInput {
    pub name: String,
    pub provider: String,
    pub model: String,
    pub reasoning: String,
    pub base_url: String,
    pub config_json: serde_json::Value,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectOutputScope {
    Project,
    Tool,
}

impl ProjectOutputScope {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Project => "project",
            Self::Tool => "tool",
        }
    }
}

pub fn render_managed_rules_markdown(input: &ProjectOutputBuildInput) -> String {
    let rules = dedupe_project_output_rules(&input.rules);
    let mut content = String::new();
    content.push_str("---\n");
    content.push_str(&format!("name: {}\n", yaml_scalar(&input.project_name)));
    content.push_str(&format!("scope: {}\n", input.scope.as_str()));
    content.push_str("managedBy: vt-hub-manager\n");
    content.push_str("---\n\n");

    if rules.is_empty() {
        content.push_str("# VT Hub Manager Managed Rules\n\n");
        content.push_str("> 暂无绑定规则\n\n");
    }

    for (index, rule) in rules.iter().enumerate() {
        if index > 0 {
            content.push_str("---\n\n");
        }

        content.push_str(&format!(
            "## {}. {} `v{}`\n\n",
            index + 1,
            display_rule_name(rule),
            rule.version_no
        ));
        content.push_str(rule.body.trim());
        content.push_str("\n\n");
    }

    content.trim_end().to_string() + "\n"
}

fn dedupe_project_output_rules(rules: &[ProjectOutputRule]) -> Vec<&ProjectOutputRule> {
    let mut deduped = Vec::<&ProjectOutputRule>::new();
    for rule in rules {
        if let Some(existing_index) = deduped.iter().position(|existing| existing.id == rule.id) {
            if rule.sort_order < deduped[existing_index].sort_order {
                deduped[existing_index] = rule;
            }
            continue;
        }
        deduped.push(rule);
    }
    deduped
}

fn display_rule_name(rule: &ProjectOutputRule) -> &str {
    let rule_name = rule.name.trim();
    if rule_name.is_empty() {
        "Untitled rule"
    } else {
        rule_name
    }
}

fn yaml_scalar(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{}\"", escaped)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn output_rule(id: i32, sort_order: i32, name: &str, body: &str) -> ProjectOutputRule {
        ProjectOutputRule {
            id,
            version_no: 1,
            code: 301,
            category_code: 301,
            sort_order,
            name: name.to_string(),
            body: body.to_string(),
        }
    }

    #[test]
    fn render_dedupes_rules_bound_both_project_and_tool_global() {
        let rendered = render_managed_rules_markdown(&ProjectOutputBuildInput {
            project_name: "Demo".to_string(),
            scope: ProjectOutputScope::Project,
            rules: vec![
                output_rule(1, 1, "Shared", "Project body"),
                output_rule(1, 0, "Shared", "Tool body"),
                output_rule(2, 2, "Project only", "Project only body"),
            ],
        });

        assert_eq!(rendered.matches("## ").count(), 2);
        assert!(rendered.contains("Tool body"));
        assert!(!rendered.contains("Project body"));
        assert!(rendered.contains("Project only body"));
    }
}

pub trait ToolAdapter {
    fn tool(&self) -> Tool;
    fn detect_installation(&self) -> bool;
    fn version(&self) -> String;
    fn live_config_path(&self) -> String;
    fn credential_state(&self) -> String;
    fn skill_state(&self) -> String;
    fn project_output_state(&self) -> String;
    fn repair_state(&self) -> String;
    fn repair_hint(&self) -> String;
    fn verify_credential(&self, token: &str) -> ToolActionResult;
    fn repair(&self) -> ToolActionResult;
    fn project_output_target_path(&self, project_root: &str) -> PathBuf;
    fn global_output_target_path(&self) -> Option<PathBuf>;
    fn skill_runtime_root(&self) -> Option<PathBuf>;
    fn preset_config_path(&self) -> Option<PathBuf>;
    fn project_output_managed_marker(&self) -> &'static str;
    fn render_project_output(&self, input: &ProjectOutputBuildInput) -> String;
    fn render_preset_config(
        &self,
        _input: &PresetConfigBuildInput,
        _existing: &str,
    ) -> Result<String, String> {
        Err(format!(
            "{} does not support managed preset config.",
            self.tool().key
        ))
    }
    fn import_live_preset(&self, _content: &str) -> Result<PresetConfigBuildInput, String> {
        Err(format!(
            "{} does not support live preset import.",
            self.tool().key
        ))
    }
    fn credential_state_code(&self) -> i32;
    fn skill_state_code(&self) -> i32;
    fn project_output_state_code(&self) -> i32;
    fn repair_state_code(&self) -> i32;

    fn diagnostics(&self) -> ToolDiagnostics {
        ToolDiagnostics {
            installation_detected: self.detect_installation(),
            version: self.version(),
            live_config_path: self.live_config_path(),
            credential_state: self.credential_state(),
            credential_state_code: self.credential_state_code(),
            skill_state: self.skill_state(),
            skill_state_code: self.skill_state_code(),
            project_output_state: self.project_output_state(),
            project_output_state_code: self.project_output_state_code(),
            repair_state: self.repair_state(),
            repair_state_code: self.repair_state_code(),
            repair_hint: self.repair_hint(),
        }
    }
}
