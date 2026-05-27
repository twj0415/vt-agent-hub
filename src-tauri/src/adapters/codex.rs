use crate::adapters::tool_adapter::ToolActionResult;
use crate::adapters::tool_adapter::{
    render_managed_rules_markdown, PresetConfigBuildInput, ProjectOutputBuildInput, ToolAdapter,
};
use crate::core::product::{MANAGED_MARKER, MANAGED_PRESET_END, MANAGED_PRESET_START};
use crate::core::tool_registry::CODEX_TOOL_ID;
use crate::domain::tool::Tool;
use crate::infrastructure::codex_runtime_repo::CodexRuntimeRepo;
use serde_json::Value;
use std::path::PathBuf;

const DEFAULT_MODEL_CONTEXT_WINDOW: i64 = 1_000_000;
const DEFAULT_MODEL_AUTO_COMPACT_TOKEN_LIMIT: i64 = 900_000;

#[derive(Debug, Default, Clone, Copy)]
pub struct CodexAdapter;

impl ToolAdapter for CodexAdapter {
    fn tool(&self) -> Tool {
        Tool {
            id: CODEX_TOOL_ID,
            key: "codex",
            enabled: true,
        }
    }

    fn detect_installation(&self) -> bool {
        CodexRuntimeRepo::detect_installation()
    }

    fn version(&self) -> String {
        CodexRuntimeRepo::version()
    }

    fn live_config_path(&self) -> String {
        CodexRuntimeRepo::config_path().display().to_string()
    }

    fn credential_state(&self) -> String {
        CodexRuntimeRepo::credential_state()
    }

    fn credential_state_code(&self) -> i32 {
        CodexRuntimeRepo::credential_state_code()
    }

    fn skill_state(&self) -> String {
        CodexRuntimeRepo::skill_state()
    }

    fn skill_state_code(&self) -> i32 {
        CodexRuntimeRepo::skill_state_code()
    }

    fn project_output_state(&self) -> String {
        CodexRuntimeRepo::project_output_state()
    }

    fn project_output_state_code(&self) -> i32 {
        CodexRuntimeRepo::project_output_state_code()
    }

    fn repair_state(&self) -> String {
        CodexRuntimeRepo::repair_state()
    }

    fn repair_state_code(&self) -> i32 {
        CodexRuntimeRepo::repair_state_code()
    }

    fn repair_hint(&self) -> String {
        CodexRuntimeRepo::repair_hint()
    }

    fn verify_credential(&self, token: &str) -> ToolActionResult {
        CodexRuntimeRepo::verify_credential(token)
    }

    fn repair(&self) -> ToolActionResult {
        CodexRuntimeRepo::repair()
    }

    fn project_output_target_path(&self, project_root: &str) -> PathBuf {
        PathBuf::from(project_root).join("AGENTS.md")
    }

    fn global_output_target_path(&self) -> Option<PathBuf> {
        Some(CodexRuntimeRepo::global_agents_path())
    }

    fn skill_runtime_root(&self) -> Option<PathBuf> {
        Some(CodexRuntimeRepo::skills_path())
    }

    fn preset_config_path(&self) -> Option<PathBuf> {
        Some(CodexRuntimeRepo::config_path())
    }

    fn project_output_managed_marker(&self) -> &'static str {
        MANAGED_MARKER
    }

    fn render_project_output(&self, input: &ProjectOutputBuildInput) -> String {
        render_managed_rules_markdown(input)
    }

    fn render_preset_config(
        &self,
        input: &PresetConfigBuildInput,
        existing: &str,
    ) -> Result<String, String> {
        Ok(merge_managed_block(existing, input))
    }

    fn import_live_preset(&self, content: &str) -> Result<PresetConfigBuildInput, String> {
        Ok(PresetConfigBuildInput {
            name: "Imported Codex Live Config".to_string(),
            provider: extract_config_value(content, "provider")
                .or_else(|| extract_config_value(content, "model_provider"))
                .unwrap_or_else(|| "OpenAI".to_string()),
            model: extract_config_value(content, "model").unwrap_or_else(|| "gpt-5.5".to_string()),
            reasoning: extract_config_value(content, "reasoning")
                .or_else(|| extract_config_value(content, "model_reasoning_effort"))
                .unwrap_or_else(|| "medium".to_string()),
            base_url: extract_config_value(content, "base_url")
                .or_else(|| extract_config_value(content, "baseUrl"))
                .unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            config_json: Value::Null,
        })
    }
}

fn build_managed_config(input: &PresetConfigBuildInput) -> ManagedCodexConfig {
    let provider = input.provider.trim();
    let provider = if provider.is_empty() {
        "OpenAI"
    } else {
        provider
    };
    let review_model = string_config(&input.config_json, "reviewModel", &input.model);
    let wire_api = string_config(&input.config_json, "wireApi", "responses");
    let disable_response_storage = bool_config(&input.config_json, "disableResponseStorage", true);
    let network_access = string_config(&input.config_json, "networkAccess", "enabled");
    let windows_wsl_setup_acknowledged =
        bool_config(&input.config_json, "windowsWslSetupAcknowledged", true);
    let context_window = i64_config(
        &input.config_json,
        "modelContextWindow",
        DEFAULT_MODEL_CONTEXT_WINDOW,
    );
    let auto_compact = i64_config(
        &input.config_json,
        "modelAutoCompactTokenLimit",
        DEFAULT_MODEL_AUTO_COMPACT_TOKEN_LIMIT,
    );
    let requires_openai_auth = bool_config(&input.config_json, "requiresOpenaiAuth", true);

    ManagedCodexConfig {
        provider: provider.to_string(),
        root_lines: vec![
            format!("model_provider = \"{}\"", escape_toml_string(provider)),
            format!("model = \"{}\"", escape_toml_string(&input.model)),
            format!("review_model = \"{}\"", escape_toml_string(&review_model)),
            format!(
                "model_reasoning_effort = \"{}\"",
                escape_toml_string(&input.reasoning)
            ),
            format!("disable_response_storage = {disable_response_storage}"),
            format!(
                "network_access = \"{}\"",
                escape_toml_string(&network_access)
            ),
            format!("windows_wsl_setup_acknowledged = {windows_wsl_setup_acknowledged}"),
            format!("model_context_window = {context_window}"),
            format!("model_auto_compact_token_limit = {auto_compact}"),
        ],
        provider_table_lines: vec![
            format!("[model_providers.{}]", toml_table_key(provider)),
            format!("name = \"{}\"", escape_toml_string(provider)),
            format!("base_url = \"{}\"", escape_toml_string(&input.base_url)),
            format!("wire_api = \"{}\"", escape_toml_string(&wire_api)),
            format!("requires_openai_auth = {requires_openai_auth}"),
        ],
    }
}

fn string_config(config: &Value, key: &str, fallback: &str) -> String {
    config
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn bool_config(config: &Value, key: &str, fallback: bool) -> bool {
    config.get(key).and_then(Value::as_bool).unwrap_or(fallback)
}

fn i64_config(config: &Value, key: &str, fallback: i64) -> i64 {
    config.get(key).and_then(Value::as_i64).unwrap_or(fallback)
}

fn toml_table_key(value: &str) -> String {
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')
    {
        value.to_string()
    } else {
        format!("\"{}\"", escape_toml_string(value))
    }
}

const MANAGED_ROOT_KEYS: [&str; 9] = [
    "model_provider",
    "model",
    "review_model",
    "model_reasoning_effort",
    "disable_response_storage",
    "network_access",
    "windows_wsl_setup_acknowledged",
    "model_context_window",
    "model_auto_compact_token_limit",
];

#[derive(Debug, Clone)]
struct ManagedCodexConfig {
    provider: String,
    root_lines: Vec<String>,
    provider_table_lines: Vec<String>,
}

fn render_managed_config_block(managed: &ManagedCodexConfig) -> String {
    [
        vec![MANAGED_PRESET_START.to_string()],
        managed.root_lines.clone(),
        vec![String::new()],
        managed.provider_table_lines.clone(),
        vec![MANAGED_PRESET_END.to_string()],
    ]
    .concat()
    .join("\n")
}

fn merge_managed_block(existing: &str, input: &PresetConfigBuildInput) -> String {
    let managed = build_managed_config(input);
    let stripped = strip_managed_block(existing);
    let sections = split_toml_sections(&stripped);
    let root_lines = trim_blank_edge_lines(
        sections
            .root_lines
            .into_iter()
            .filter(|line| !is_managed_root_key_line(line))
            .collect(),
    );
    let table_sections: Vec<Vec<String>> = sections
        .table_sections
        .into_iter()
        .filter(|section| !is_provider_table_section(section, &managed.provider))
        .map(trim_blank_edge_lines)
        .filter(|section| !section.is_empty())
        .collect();

    let mut rendered_sections = Vec::new();
    let root_section = lines_to_section(&root_lines);
    if !root_section.is_empty() {
        rendered_sections.push(root_section);
    }
    rendered_sections.push(render_managed_config_block(&managed));
    rendered_sections.extend(
        table_sections
            .iter()
            .map(|section| lines_to_section(section)),
    );

    rendered_sections.join("\n\n").trim_end().to_string() + "\n"
}

fn strip_managed_block(existing: &str) -> String {
    let Some(start) = existing.find(MANAGED_PRESET_START) else {
        return existing.to_string();
    };

    let Some(relative_end) = existing[start..].find(MANAGED_PRESET_END) else {
        return existing[..start].trim_end().to_string();
    };

    let end = start + relative_end + MANAGED_PRESET_END.len();
    let mut next = String::new();
    next.push_str(existing[..start].trim_end());
    let trailing = existing[end..].trim_start();
    if !next.is_empty() && !trailing.is_empty() {
        next.push_str("\n\n");
    }
    next.push_str(trailing);
    next
}

#[derive(Debug, Default)]
struct TomlSections {
    root_lines: Vec<String>,
    table_sections: Vec<Vec<String>>,
}

fn split_toml_sections(content: &str) -> TomlSections {
    let normalized = content.replace("\r\n", "\n");
    let mut sections = TomlSections::default();
    let mut current_table = Vec::<String>::new();
    let mut in_table = false;

    for line in normalized.lines() {
        if table_header_name(line).is_some() {
            if in_table && !current_table.is_empty() {
                sections
                    .table_sections
                    .push(std::mem::take(&mut current_table));
            }
            in_table = true;
            current_table.push(line.to_string());
            continue;
        }

        if in_table {
            current_table.push(line.to_string());
        } else {
            sections.root_lines.push(line.to_string());
        }
    }

    if !current_table.is_empty() {
        sections.table_sections.push(current_table);
    }

    sections
}

fn is_managed_root_key_line(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return false;
    }

    let Some((left, _)) = trimmed.split_once('=') else {
        return false;
    };
    MANAGED_ROOT_KEYS.contains(&left.trim())
}

fn is_provider_table_section(lines: &[String], provider: &str) -> bool {
    lines
        .first()
        .and_then(|line| provider_table_name(line))
        .map(|name| name == provider)
        .unwrap_or(false)
}

fn provider_table_name(line: &str) -> Option<String> {
    let header = table_header_name(line)?;
    let rest = header.strip_prefix("model_providers.")?;
    Some(unquote_toml_string(rest.trim()))
}

fn table_header_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') || !trimmed.starts_with('[') {
        return None;
    }
    let end = trimmed.rfind(']')?;
    let inner = trimmed.get(1..end)?.trim();
    if inner.is_empty() || inner.starts_with('[') {
        return None;
    }
    Some(inner.to_string())
}

fn trim_blank_edge_lines(mut lines: Vec<String>) -> Vec<String> {
    while lines.first().is_some_and(|line| line.trim().is_empty()) {
        lines.remove(0);
    }
    while lines.last().is_some_and(|line| line.trim().is_empty()) {
        lines.pop();
    }
    lines
}

fn lines_to_section(lines: &[String]) -> String {
    lines.join("\n").trim_end().to_string()
}

fn extract_config_value(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        let (left, right) = trimmed.split_once('=')?;
        if left.trim() != key {
            continue;
        }
        let value = right.trim().trim_matches('"').trim_matches('\'').trim();
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

fn unquote_toml_string(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.len() >= 2 && trimmed.starts_with('"') && trimmed.ends_with('"') {
        trimmed[1..trimmed.len() - 1]
            .replace("\\\"", "\"")
            .replace("\\\\", "\\")
    } else if trimmed.len() >= 2 && trimmed.starts_with('\'') && trimmed.ends_with('\'') {
        trimmed[1..trimmed.len() - 1].to_string()
    } else {
        trimmed.to_string()
    }
}

fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(model: &str) -> PresetConfigBuildInput {
        PresetConfigBuildInput {
            name: "Demo".to_string(),
            provider: "OpenAI".to_string(),
            model: model.to_string(),
            reasoning: "xhigh".to_string(),
            base_url: "https://api.freemodel.dev".to_string(),
            config_json: serde_json::json!({
                "reviewModel": "gpt-5.4",
                "wireApi": "responses",
                "disableResponseStorage": true,
                "networkAccess": "enabled",
                "windowsWslSetupAcknowledged": true,
                "modelContextWindow": 1_000_000,
                "modelAutoCompactTokenLimit": 900_000,
                "requiresOpenaiAuth": true
            }),
        }
    }

    fn count_occurrences(content: &str, needle: &str) -> usize {
        content.matches(needle).count()
    }

    #[test]
    fn merges_into_existing_live_config_without_duplicate_root_keys_or_provider_table() {
        let existing = r#"model_provider = "OpenAI"
model = "gpt-5.5"
review_model = "gpt-5.4"
model_reasoning_effort = "xhigh"
disable_response_storage = true
network_access = "enabled"
windows_wsl_setup_acknowledged = true
model_context_window = 1000000
model_auto_compact_token_limit = 900000

[model_providers.OpenAI]
name = "OpenAI"
base_url = "https://api.freemodel.dev"
wire_api = "responses"
requires_openai_auth = true

[projects.'c:\users\twj']
trust_level = "trusted"

[tui.model_availability_nux]
"gpt-5.5" = 4

[windows]
sandbox = "elevated"
"#;
        let rendered = merge_managed_block(existing, &input("gpt-5.4-mini"));

        assert_eq!(count_occurrences(&rendered, "model_provider = "), 1);
        assert_eq!(count_occurrences(&rendered, "[model_providers.OpenAI]"), 1);
        assert!(rendered.contains("model = \"gpt-5.4-mini\""));
        assert!(rendered.contains("[projects.'c:\\users\\twj']"));
        assert!(rendered.contains("\"gpt-5.5\" = 4"));
        assert!(rendered.contains("[windows]\nsandbox = \"elevated\""));
        assert!(
            rendered.find("model_provider = \"OpenAI\"").unwrap()
                < rendered.find("[projects.'c:\\users\\twj']").unwrap()
        );
    }

    #[test]
    fn repairs_previously_appended_managed_block_into_a_single_valid_section() {
        let existing = r#"model_provider = "OpenAI"
model = "gpt-5.5"
review_model = "gpt-5.4"
model_reasoning_effort = "xhigh"
disable_response_storage = true
network_access = "enabled"
windows_wsl_setup_acknowledged = true
model_context_window = 1000000
model_auto_compact_token_limit = 900000

[model_providers.OpenAI]
name = "OpenAI"
base_url = "https://api.freemodel.dev"
wire_api = "responses"
requires_openai_auth = true

[projects.'c:\users\twj']
trust_level = "trusted"

[tui.model_availability_nux]
"gpt-5.5" = 4

[windows]
sandbox = "elevated"

# VT Hub Manager managed preset start
model_provider = "OpenAI"
model = "gpt-5.4-mini"
review_model = "gpt-5.4"
model_reasoning_effort = "xhigh"
disable_response_storage = true
network_access = "enabled"
windows_wsl_setup_acknowledged = true
model_context_window = 1000000
model_auto_compact_token_limit = 900000

[model_providers.OpenAI]
name = "OpenAI"
base_url = "https://api.freemodel.dev"
wire_api = "responses"
requires_openai_auth = true
# VT Hub Manager managed preset end
"#;
        let rendered = merge_managed_block(existing, &input("gpt-5.4-mini"));

        assert_eq!(count_occurrences(&rendered, MANAGED_PRESET_START), 1);
        assert_eq!(count_occurrences(&rendered, MANAGED_PRESET_END), 1);
        assert_eq!(count_occurrences(&rendered, "[model_providers.OpenAI]"), 1);
        assert_eq!(count_occurrences(&rendered, "model_provider = "), 1);
        assert!(
            rendered.find("model_provider = \"OpenAI\"").unwrap()
                < rendered.find("[projects.'c:\\users\\twj']").unwrap()
        );
    }
}
