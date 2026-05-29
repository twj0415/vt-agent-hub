use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::adapters::tool_adapter::PresetConfigBuildInput;
use crate::core::paths;

use super::ProviderRuntimeService;

const CLAUDE_MANAGED_ENV_KEYS: &[&str] = &[
    "ANTHROPIC_MODEL",
    "ANTHROPIC_SMALL_FAST_MODEL",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL",
    "ANTHROPIC_DEFAULT_SONNET_MODEL",
    "ANTHROPIC_DEFAULT_OPUS_MODEL",
    "ANTHROPIC_BASE_URL",
    "ANTHROPIC_API_KEY",
    "ANTHROPIC_AUTH_TOKEN",
    "CLAUDE_CODE_USE_BEDROCK",
    "CLAUDE_CODE_USE_VERTEX",
    "AWS_REGION",
    "AWS_PROFILE",
    "GOOGLE_CLOUD_PROJECT",
    "VERTEX_REGION",
];

impl ProviderRuntimeService {
    pub(super) fn read_config(path: &Path) -> Result<String, String> {
        if !path.exists() {
            return Ok(String::new());
        }
        fs::read_to_string(path)
            .map_err(|error| format!("Failed to read {}: {error}", path.display()))
    }

    pub(super) fn read_auth_preview(path: &Path) -> Result<String, String> {
        if !path.exists() {
            return Ok(String::new());
        }
        let content = fs::read_to_string(path)
            .map_err(|error| format!("Failed to read {}: {error}", path.display()))?;
        let Ok(mut value) = serde_json::from_str::<Value>(&content) else {
            return Ok("<existing auth.json hidden>\n".to_string());
        };
        if let Some(object) = value.as_object_mut() {
            if object.contains_key("OPENAI_API_KEY") {
                object.insert(
                    "OPENAI_API_KEY".to_string(),
                    Value::String("<existing credential>".to_string()),
                );
            }
        }
        serde_json::to_string_pretty(&value)
            .map(|value| value + "\n")
            .map_err(|error| error.to_string())
    }

    pub(super) fn claude_settings_path() -> PathBuf {
        paths::claude_root().join("settings.json")
    }

    pub(super) fn read_claude_preview(path: &Path) -> Result<String, String> {
        if !path.exists() {
            return Ok(String::new());
        }
        let content = fs::read_to_string(path)
            .map_err(|error| format!("Failed to read {}: {error}", path.display()))?;
        let Ok(value) = serde_json::from_str::<Value>(&content) else {
            return Ok("<existing Claude settings hidden>\n".to_string());
        };
        let masked = Self::mask_claude_settings(&value);
        serde_json::to_string_pretty(&masked)
            .map(|value| value + "\n")
            .map_err(|error| error.to_string())
    }

    pub(super) fn mask_claude_preview(content: &str) -> Result<String, String> {
        let value: Value = serde_json::from_str(content.trim_start_matches('\u{feff}'))
            .map_err(|error| format!("Failed to parse Claude settings.json: {error}"))?;
        let masked = Self::mask_claude_settings(&value);
        serde_json::to_string_pretty(&masked)
            .map(|value| value + "\n")
            .map_err(|error| error.to_string())
    }

    pub(super) fn render_claude_settings(
        existing: &str,
        input: &PresetConfigBuildInput,
    ) -> Result<String, String> {
        let mut root: Value = if existing.trim().is_empty() {
            json!({})
        } else {
            serde_json::from_str(existing.trim_start_matches('\u{feff}'))
                .map_err(|error| format!("Failed to parse Claude settings.json: {error}"))?
        };

        if !root.is_object() {
            root = json!({});
        }

        let Some(root_obj) = root.as_object_mut() else {
            return Err("Claude settings.json must be a JSON object.".to_string());
        };

        let env_value = root_obj
            .entry("env".to_string())
            .or_insert_with(|| json!({}));
        let Some(env) = env_value.as_object_mut() else {
            return Err("Claude settings.json env must be a JSON object.".to_string());
        };

        let existing_api_key = env
            .get("ANTHROPIC_API_KEY")
            .and_then(Value::as_str)
            .map(str::to_string);
        let existing_auth_token = env
            .get("ANTHROPIC_AUTH_TOKEN")
            .and_then(Value::as_str)
            .map(str::to_string);

        for key in CLAUDE_MANAGED_ENV_KEYS {
            env.remove(*key);
        }

        let model = input.model.trim();
        let base_url = input.base_url.trim();
        let provider_kind = input
            .config_json
            .get("providerKind")
            .and_then(Value::as_str)
            .unwrap_or_else(|| {
                if base_url.starts_with("bedrock://") {
                    "bedrock"
                } else if base_url.starts_with("vertex://") {
                    "vertex"
                } else if base_url.trim_end_matches('/').eq_ignore_ascii_case("https://api.anthropic.com")
                    || base_url.trim_end_matches('/').eq_ignore_ascii_case("https://api.anthropic.com/v1")
                {
                    "anthropic"
                } else {
                    "anthropic-compatible"
                }
            });
        let small_fast_model = config_string(&input.config_json, "smallFastModel");
        let default_small_fast = small_fast_model
            .clone()
            .unwrap_or_else(|| model.to_string());

        env.insert("ANTHROPIC_MODEL".to_string(), Value::String(model.to_string()));
        env.insert(
            "ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(),
            Value::String(default_small_fast.clone()),
        );
        env.insert(
            "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
            Value::String(model.to_string()),
        );
        env.insert(
            "ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(),
            Value::String(model.to_string()),
        );

        if let Some(small_fast_model) = small_fast_model {
            env.insert(
                "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
                Value::String(small_fast_model),
            );
        }

        let credential_key = input
            .config_json
            .get("credentialKey")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("ANTHROPIC_API_KEY");

        if let Some(token) = input
            .credential_token
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            env.insert(credential_key.to_string(), Value::String(token.to_string()));
            if credential_key != "ANTHROPIC_API_KEY" {
                env.remove("ANTHROPIC_API_KEY");
            }
            if credential_key != "ANTHROPIC_AUTH_TOKEN" {
                env.remove("ANTHROPIC_AUTH_TOKEN");
            }
        } else {
            if let Some(existing_api_key) = existing_api_key {
                env.insert("ANTHROPIC_API_KEY".to_string(), Value::String(existing_api_key));
            }
            if let Some(existing_auth_token) = existing_auth_token {
                env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), Value::String(existing_auth_token));
            }
        }

        match provider_kind {
            "bedrock" => {
                env.insert(
                    "CLAUDE_CODE_USE_BEDROCK".to_string(),
                    Value::String("true".to_string()),
                );
                env.remove("CLAUDE_CODE_USE_VERTEX");
                env.remove("ANTHROPIC_BASE_URL");

                if let Some(aws_region) = config_string(&input.config_json, "awsRegion")
                    .or_else(|| single_display_url_part(base_url, "bedrock://"))
                {
                    env.insert("AWS_REGION".to_string(), Value::String(aws_region));
                }
                if let Some(aws_profile) = config_string(&input.config_json, "awsProfile") {
                    env.insert("AWS_PROFILE".to_string(), Value::String(aws_profile));
                }
                env.remove("GOOGLE_CLOUD_PROJECT");
                env.remove("VERTEX_REGION");
            }
            "vertex" => {
                env.insert(
                    "CLAUDE_CODE_USE_VERTEX".to_string(),
                    Value::String("true".to_string()),
                );
                env.remove("CLAUDE_CODE_USE_BEDROCK");
                env.remove("ANTHROPIC_BASE_URL");

                let (display_project, display_region) = vertex_display_url_parts(base_url);
                if let Some(project) = config_string(&input.config_json, "googleCloudProject")
                    .or(display_project)
                {
                    env.insert("GOOGLE_CLOUD_PROJECT".to_string(), Value::String(project));
                }
                if let Some(vertex_region) = config_string(&input.config_json, "vertexRegion")
                    .or(display_region)
                {
                    env.insert("VERTEX_REGION".to_string(), Value::String(vertex_region));
                }
                env.remove("AWS_REGION");
                env.remove("AWS_PROFILE");
            }
            _ => {
                env.insert(
                    "ANTHROPIC_BASE_URL".to_string(),
                    Value::String(base_url.to_string()),
                );
                env.remove("CLAUDE_CODE_USE_BEDROCK");
                env.remove("CLAUDE_CODE_USE_VERTEX");
                env.remove("AWS_REGION");
                env.remove("AWS_PROFILE");
                env.remove("GOOGLE_CLOUD_PROJECT");
                env.remove("VERTEX_REGION");
            }
        }

        root_obj.insert("model".to_string(), Value::String("sonnet".to_string()));
        serde_json::to_string_pretty(&root)
            .map(|value| value + "\n")
            .map_err(|error| error.to_string())
    }

    pub(super) fn mask_claude_settings(value: &Value) -> Value {
        let mut masked = value.clone();
        if let Some(root) = masked.as_object_mut() {
            if let Some(env) = root.get_mut("env").and_then(Value::as_object_mut) {
                for key in [
                    "ANTHROPIC_API_KEY",
                    "ANTHROPIC_AUTH_TOKEN",
                    "AWS_ACCESS_KEY_ID",
                    "AWS_SECRET_ACCESS_KEY",
                ] {
                    if env.contains_key(key) {
                        env.insert(
                            key.to_string(),
                            Value::String("<existing credential>".to_string()),
                        );
                    }
                }
            }
        }
        masked
    }

    pub(super) fn auth_path_for_config(config_path: &Path) -> Result<PathBuf, String> {
        let Some(parent) = config_path.parent() else {
            return Err(format!(
                "Cannot resolve auth.json next to {}.",
                config_path.display()
            ));
        };
        Ok(parent.join("auth.json"))
    }

    pub(super) fn write_text(path: &Path, content: &str) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::write(path, content)
            .map_err(|error| format!("Failed to write {}: {error}", path.display()))
    }

    pub(super) fn build_diff(label: &str, before: &str, after: &str) -> String {
        format!("--- existing {label}\n{before}\n+++ generated {label}\n{after}")
    }
}

fn config_string(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn single_display_url_part(value: &str, prefix: &str) -> Option<String> {
    value
        .strip_prefix(prefix)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn vertex_display_url_parts(value: &str) -> (Option<String>, Option<String>) {
    let Some(rest) = value.strip_prefix("vertex://") else {
        return (None, None);
    };
    let mut parts = rest.split('/').map(str::trim).filter(|part| !part.is_empty());
    let project = parts.next().map(str::to_string);
    let region = parts.next().map(str::to_string);
    (project, region)
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::adapters::tool_adapter::PresetConfigBuildInput;
    use crate::application::provider_runtime_service::ProviderRuntimeService;

    #[test]
    fn claude_settings_apply_preserves_existing_credential_when_token_is_not_managed() {
        let rendered = ProviderRuntimeService::render_claude_settings(
            r#"{
  "env": {
    "ANTHROPIC_API_KEY": "existing-secret",
    "UNRELATED": "keep"
  }
}"#,
            &PresetConfigBuildInput {
                name: "Anthropic".to_string(),
                provider: "Anthropic".to_string(),
                model: "claude-opus-4-7".to_string(),
                reasoning: "medium".to_string(),
                base_url: "https://api.anthropic.com".to_string(),
                credential_token: None,
                config_json: json!({
                    "providerKind": "anthropic",
                    "credentialKey": "ANTHROPIC_API_KEY"
                }),
            },
        )
        .expect("claude settings should render");
        let parsed: serde_json::Value = serde_json::from_str(&rendered).expect("valid json");
        let env = parsed.get("env").and_then(serde_json::Value::as_object).expect("env object");

        assert_eq!(
            env.get("ANTHROPIC_API_KEY").and_then(serde_json::Value::as_str),
            Some("existing-secret")
        );
        assert_eq!(
            env.get("UNRELATED").and_then(serde_json::Value::as_str),
            Some("keep")
        );
        assert_eq!(
            env.get("ANTHROPIC_MODEL").and_then(serde_json::Value::as_str),
            Some("claude-opus-4-7")
        );
    }

    #[test]
    fn claude_settings_apply_keeps_top_level_model_as_sonnet_and_updates_env_models() {
        let rendered = ProviderRuntimeService::render_claude_settings(
            r#"{
  "model": "sonnet",
  "env": {
    "ANTHROPIC_MODEL": "old-model",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL": "old-haiku",
    "ANTHROPIC_DEFAULT_SONNET_MODEL": "old-sonnet",
    "ANTHROPIC_DEFAULT_OPUS_MODEL": "old-opus"
  }
}"#,
            &PresetConfigBuildInput {
                name: "Claude Compatible".to_string(),
                provider: "Claude Compatible".to_string(),
                model: "gpt-5.5".to_string(),
                reasoning: "xhigh".to_string(),
                base_url: "http://43.173.89.135:8080".to_string(),
                credential_token: None,
                config_json: json!({
                    "providerKind": "anthropic-compatible",
                    "credentialKey": "ANTHROPIC_API_KEY"
                }),
            },
        )
        .expect("claude settings should render");
        let parsed: serde_json::Value = serde_json::from_str(&rendered).expect("valid json");
        let env = parsed.get("env").and_then(serde_json::Value::as_object).expect("env object");

        assert_eq!(
            parsed.get("model").and_then(serde_json::Value::as_str),
            Some("sonnet")
        );
        assert_eq!(
            env.get("ANTHROPIC_MODEL").and_then(serde_json::Value::as_str),
            Some("gpt-5.5")
        );
        assert_eq!(
            env.get("ANTHROPIC_DEFAULT_HAIKU_MODEL").and_then(serde_json::Value::as_str),
            Some("gpt-5.5")
        );
        assert_eq!(
            env.get("ANTHROPIC_DEFAULT_SONNET_MODEL").and_then(serde_json::Value::as_str),
            Some("gpt-5.5")
        );
        assert_eq!(
            env.get("ANTHROPIC_DEFAULT_OPUS_MODEL").and_then(serde_json::Value::as_str),
            Some("gpt-5.5")
        );
    }
}
