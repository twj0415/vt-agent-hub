use std::path::PathBuf;

use crate::adapters::tool_adapter::{
    render_managed_rules_markdown, ProjectOutputBuildInput, ProviderConfigImport, ToolActionResult,
    ToolAdapter,
};
use crate::core::product::MANAGED_MARKER;
use crate::core::status_codes::{TARGET_STATE_MISSING, TARGET_STATE_PLANNED, TARGET_STATE_READY};
use crate::core::tool_registry::CLAUDE_TOOL_ID;
use crate::domain::tool::Tool;
use crate::dto::ProviderImportInputDto;
use crate::infrastructure::claude_runtime_repo::ClaudeRuntimeRepo;
use serde_json::{json, Value};

#[derive(Debug, Default, Clone, Copy)]
pub struct ClaudeAdapter;

impl ToolAdapter for ClaudeAdapter {
    fn tool(&self) -> Tool {
        Tool {
            id: CLAUDE_TOOL_ID,
            key: "claude",
            enabled: true,
        }
    }

    fn detect_installation(&self) -> bool {
        ClaudeRuntimeRepo::root().exists()
    }

    fn version(&self) -> String {
        "-".to_string()
    }

    fn live_config_path(&self) -> String {
        ClaudeRuntimeRepo::global_claude_path()
            .display()
            .to_string()
    }

    fn credential_state(&self) -> String {
        "managed_elsewhere".to_string()
    }

    fn credential_state_code(&self) -> i32 {
        TARGET_STATE_PLANNED
    }

    fn skill_state(&self) -> String {
        "tool_local".to_string()
    }

    fn skill_state_code(&self) -> i32 {
        TARGET_STATE_PLANNED
    }

    fn project_output_state(&self) -> String {
        if self.detect_installation() {
            "preview_ready".to_string()
        } else {
            "tool_missing".to_string()
        }
    }

    fn project_output_state_code(&self) -> i32 {
        if self.detect_installation() {
            TARGET_STATE_READY
        } else {
            TARGET_STATE_MISSING
        }
    }

    fn repair_state(&self) -> String {
        "manual_required".to_string()
    }

    fn repair_state_code(&self) -> i32 {
        TARGET_STATE_PLANNED
    }

    fn repair_hint(&self) -> String {
        "Claude Code global and project memory should be reviewed before overwrite.".to_string()
    }

    fn verify_credential(&self, _token: &str) -> ToolActionResult {
        ToolActionResult {
            ok: false,
            state: "unsupported".to_string(),
            detail: "Credential verification is not implemented for Claude Code here.".to_string(),
            manual_steps: vec![
                "Claude Code is hidden in V1 and has no supported credential flow.".to_string(),
            ],
        }
    }

    fn repair(&self) -> ToolActionResult {
        ToolActionResult {
            ok: false,
            state: "manual_required".to_string(),
            detail: "Claude Code repair requires manual review of the target memory file."
                .to_string(),
            manual_steps: vec![
                "Claude Code is hidden in V1 and has no supported repair flow.".to_string(),
            ],
        }
    }

    fn project_output_target_path(&self, project_root: &str) -> PathBuf {
        PathBuf::from(project_root).join("CLAUDE.md")
    }

    fn global_output_target_path(&self) -> Option<PathBuf> {
        Some(ClaudeRuntimeRepo::global_claude_path())
    }

    fn skill_runtime_root(&self) -> Option<PathBuf> {
        Some(ClaudeRuntimeRepo::root().join("skills"))
    }

    fn preset_config_path(&self) -> Option<PathBuf> {
        None
    }

    fn project_output_managed_marker(&self) -> &'static str {
        MANAGED_MARKER
    }

    fn render_project_output(&self, input: &ProjectOutputBuildInput) -> String {
        render_managed_rules_markdown(input)
    }

    fn import_provider_config(&self, input: &ProviderImportInputDto) -> Result<ProviderConfigImport, String> {
        let config_content = required_import_part(input, "config")?;
        parse_claude_provider_config(config_content)
    }
}

fn required_import_part<'a>(input: &'a ProviderImportInputDto, role: &str) -> Result<&'a str, String> {
    input
        .parts
        .iter()
        .find(|part| part.role.trim().eq_ignore_ascii_case(role))
        .map(|part| part.content.trim())
        .filter(|content| !content.is_empty())
        .ok_or_else(|| format!("Provider import requires pasted {role} content."))
}

fn parse_claude_provider_config(content: &str) -> Result<ProviderConfigImport, String> {
    let root: Value = serde_json::from_str(content.trim_start_matches('\u{feff}'))
        .map_err(|error| format!("Failed to parse Claude settings.json: {error}"))?;
    let env = root.get("env").and_then(Value::as_object);
    let model = json_env_string(&root, "ANTHROPIC_MODEL")
        .or_else(|| json_string(&root, "model"))
        .unwrap_or_else(|| "claude-opus-4-7".to_string());
    let small_fast_model = json_env_string(&root, "ANTHROPIC_SMALL_FAST_MODEL");
    let use_bedrock = json_env_bool(&root, "CLAUDE_CODE_USE_BEDROCK");
    let use_vertex = json_env_bool(&root, "CLAUDE_CODE_USE_VERTEX");
    let aws_region = json_env_string(&root, "AWS_REGION");
    let aws_profile = json_env_string(&root, "AWS_PROFILE");
    let google_project = json_env_string(&root, "GOOGLE_CLOUD_PROJECT");
    let vertex_region = json_env_string(&root, "VERTEX_REGION");
    let base_url = json_env_string(&root, "ANTHROPIC_BASE_URL");

    let provider_kind = if use_bedrock || aws_region.is_some() || aws_profile.is_some() {
        "bedrock"
    } else if use_vertex || google_project.is_some() || vertex_region.is_some() {
        "vertex"
    } else if base_url.as_deref().is_some_and(|url| !is_official_anthropic_url(url)) {
        "anthropic-compatible"
    } else {
        "anthropic"
    };

    let resolved_base_url = match provider_kind {
        "bedrock" => format!("bedrock://{}", aws_region.as_deref().unwrap_or("default")),
        "vertex" => format!(
            "vertex://{}/{}",
            google_project.as_deref().unwrap_or("default"),
            vertex_region.as_deref().unwrap_or("global")
        ),
        _ => base_url.unwrap_or_else(|| "https://api.anthropic.com".to_string()),
    };

    let provider_name = match provider_kind {
        "anthropic" => "Anthropic".to_string(),
        "anthropic-compatible" => "Claude Compatible".to_string(),
        "bedrock" => "Claude Bedrock".to_string(),
        "vertex" => "Claude Vertex".to_string(),
        _ => "Claude".to_string(),
    };
    let category = match provider_kind {
        "anthropic" => "official",
        "bedrock" | "vertex" => "cloud_provider",
        _ => "third_party",
    }
    .to_string();
    let api_key = json_env_string(&root, "ANTHROPIC_API_KEY");
    let auth_token = json_env_string(&root, "ANTHROPIC_AUTH_TOKEN");
    let (credential_key, credential_token) = if let Some(token) = api_key {
        ("ANTHROPIC_API_KEY", Some(token))
    } else if let Some(token) = auth_token {
        ("ANTHROPIC_AUTH_TOKEN", Some(token))
    } else {
        ("", None)
    };
    let credential_detected = credential_token.is_some();

    Ok(ProviderConfigImport {
        provider_name,
        category: category.clone(),
        model,
        reasoning: "medium".to_string(),
        base_url: resolved_base_url,
        credential_token,
        config_json: json!({
            "providerKind": provider_kind,
            "category": category,
            "smallFastModel": small_fast_model,
            "awsRegion": aws_region,
            "awsProfile": aws_profile,
            "awsProfileDetected": aws_profile.is_some(),
            "googleCloudProject": google_project,
            "vertexRegion": vertex_region,
            "credentialDetected": credential_detected,
            "credentialSource": if credential_detected { "settings.env" } else { "" },
            "credentialKey": credential_key,
            "proxyDetected": env.is_some_and(|values| values.contains_key("HTTPS_PROXY") || values.contains_key("HTTP_PROXY")),
            "liveApplySupported": true,
        }),
    })
}

fn json_string(root: &Value, key: &str) -> Option<String> {
    root.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn json_env_string(root: &Value, key: &str) -> Option<String> {
    root
        .get("env")
        .and_then(Value::as_object)
        .and_then(|env| env.get(key))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn json_env_bool(root: &Value, key: &str) -> bool {
    json_env_string(root, key)
        .map(|value| matches!(value.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

fn is_official_anthropic_url(value: &str) -> bool {
    let normalized = value.trim().trim_end_matches('/').to_ascii_lowercase();
    normalized == "https://api.anthropic.com" || normalized == "https://api.anthropic.com/v1"
}
