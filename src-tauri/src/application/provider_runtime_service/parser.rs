use serde_json::{json, Value};
use toml::Value as TomlValue;

use super::constants::DEFAULT_BASE_URL;
use super::ProviderRuntimeService;

#[derive(Debug, Clone)]
pub(crate) struct ParsedCodexConfig {
    pub(crate) provider_name: String,
    pub(crate) model: String,
    pub(crate) reasoning: String,
    pub(crate) base_url: String,
    pub(crate) config_json: Value,
}

#[derive(Debug, Clone)]
pub(crate) struct ParsedClaudeSettings {
    pub(crate) provider_name: String,
    pub(crate) model: String,
    pub(crate) reasoning: String,
    pub(crate) base_url: String,
    pub(crate) category: String,
    pub(crate) credential_detected: bool,
    pub(crate) credential_source: String,
    pub(crate) credential_token: Option<String>,
    pub(crate) config_json: Value,
}

impl ProviderRuntimeService {
    pub(crate) fn parse_codex_config(content: &str) -> Result<ParsedCodexConfig, String> {
        let content = content.trim_start_matches('\u{feff}');
        let parsed = content
            .parse::<TomlValue>()
            .map_err(|error| format!("Failed to parse provider config.toml: {error}"))?;
        let root = parsed
            .as_table()
            .ok_or_else(|| "Provider config.toml must be a TOML table.".to_string())?;
        let provider_tables = root
            .get("model_providers")
            .and_then(TomlValue::as_table)
            .cloned()
            .unwrap_or_default();

        let provider_key = Self::table_string(root, "model_provider")
            .or_else(|| provider_tables.keys().next().cloned())
            .unwrap_or_else(|| "OpenAI".to_string());
        let provider_table = provider_tables
            .get(&provider_key)
            .and_then(TomlValue::as_table);
        let provider_name = provider_table
            .and_then(|table| Self::value_string(table, "name"))
            .unwrap_or_else(|| provider_key.clone());
        let model = Self::table_string(root, "model").unwrap_or_else(|| "gpt-5.5".to_string());
        let review_model =
            Self::table_string(root, "review_model").unwrap_or_else(|| model.clone());
        let reasoning = Self::table_string(root, "model_reasoning_effort")
            .or_else(|| Self::table_string(root, "reasoning"))
            .unwrap_or_else(|| "medium".to_string());
        let base_url = provider_table
            .and_then(|table| Self::value_string(table, "base_url"))
            .or_else(|| Self::table_string(root, "base_url"))
            .unwrap_or_else(|| DEFAULT_BASE_URL.to_string());
        let wire_api = provider_table
            .and_then(|table| Self::value_string(table, "wire_api"))
            .unwrap_or_else(|| "responses".to_string());
        let requires_openai_auth = provider_table
            .and_then(|table| Self::value_bool(table, "requires_openai_auth"))
            .unwrap_or(true);

        Ok(ParsedCodexConfig {
            provider_name,
            model,
            reasoning,
            base_url,
            config_json: json!({
                "reviewModel": review_model,
                "wireApi": wire_api,
                "requiresOpenaiAuth": requires_openai_auth,
                "disableResponseStorage": Self::table_bool(root, "disable_response_storage").unwrap_or(true),
                "networkAccess": Self::table_string(root, "network_access").unwrap_or_else(|| "enabled".to_string()),
                "windowsWslSetupAcknowledged": Self::table_bool(root, "windows_wsl_setup_acknowledged").unwrap_or(true),
                "modelContextWindow": Self::table_i64(root, "model_context_window").unwrap_or(1_000_000),
                "modelAutoCompactTokenLimit": Self::table_i64(root, "model_auto_compact_token_limit").unwrap_or(900_000),
            }),
        })
    }

    pub(crate) fn parse_claude_settings(content: &str) -> Result<ParsedClaudeSettings, String> {
        let root: Value = serde_json::from_str(content.trim_start_matches('\u{feff}'))
            .map_err(|error| format!("Failed to parse Claude settings.json: {error}"))?;
        let env = root.get("env").and_then(Value::as_object);
        let model = Self::json_env_string(&root, "ANTHROPIC_MODEL")
            .or_else(|| Self::json_string(&root, "model"))
            .unwrap_or_else(|| "claude-opus-4-7".to_string());
        let small_fast_model = Self::json_env_string(&root, "ANTHROPIC_SMALL_FAST_MODEL");
        let use_bedrock = Self::json_env_bool(&root, "CLAUDE_CODE_USE_BEDROCK");
        let use_vertex = Self::json_env_bool(&root, "CLAUDE_CODE_USE_VERTEX");
        let aws_region = Self::json_env_string(&root, "AWS_REGION");
        let aws_profile = Self::json_env_string(&root, "AWS_PROFILE");
        let google_project = Self::json_env_string(&root, "GOOGLE_CLOUD_PROJECT");
        let vertex_region = Self::json_env_string(&root, "VERTEX_REGION");
        let base_url = Self::json_env_string(&root, "ANTHROPIC_BASE_URL");
        let api_key = Self::json_env_string(&root, "ANTHROPIC_API_KEY");
        let auth_token = Self::json_env_string(&root, "ANTHROPIC_AUTH_TOKEN");
        let (credential_key, credential_token) = if let Some(token) = api_key {
            ("ANTHROPIC_API_KEY", Some(token))
        } else if let Some(token) = auth_token {
            ("ANTHROPIC_AUTH_TOKEN", Some(token))
        } else {
            ("", None)
        };
        let credential_detected = credential_token.is_some();
        let credential_source = if credential_detected {
            "settings.env".to_string()
        } else {
            String::new()
        };

        let provider_kind = if use_bedrock || aws_region.is_some() || aws_profile.is_some() {
            "bedrock"
        } else if use_vertex || google_project.is_some() || vertex_region.is_some() {
            "vertex"
        } else if base_url
            .as_deref()
            .is_some_and(|url| !Self::is_official_anthropic_url(url))
        {
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

        // 反向解析模型映射
        let opus_model = Self::json_env_string(&root, "ANTHROPIC_DEFAULT_OPUS_MODEL");
        let sonnet_model = Self::json_env_string(&root, "ANTHROPIC_DEFAULT_SONNET_MODEL");
        let haiku_model = Self::json_env_string(&root, "ANTHROPIC_DEFAULT_HAIKU_MODEL");
        let model_mapping = json!({
            "opus": opus_model,
            "sonnet": sonnet_model,
            "haiku": haiku_model,
            "smallFast": small_fast_model,
        });

        // 反向解析开关字段
        let switches = json!({
            "disableNonessentialTraffic": Self::json_env_truthy(&root, "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC"),
            "attributionHeader": Self::json_env_truthy(&root, "CLAUDE_CODE_ATTRIBUTION_HEADER"),
            "disableTelemetry": Self::json_env_truthy(&root, "DISABLE_TELEMETRY"),
            "disableNonEssentialModelCalls": Self::json_env_truthy(&root, "DISABLE_NON_ESSENTIAL_MODEL_CALLS"),
            "disableAutoupdater": Self::json_env_truthy(&root, "DISABLE_AUTOUPDATER"),
            "disableErrorReporting": Self::json_env_truthy(&root, "DISABLE_ERROR_REPORTING"),
            "claudeCodeBeta": Self::json_env_truthy(&root, "CLAUDE_CODE_BETA"),
        });

        // 反向解析数值字段
        let numbers = json!({
            "apiTimeoutMs": Self::json_env_string(&root, "API_TIMEOUT_MS"),
            "claudeCodeMaxOutputTokens": Self::json_env_string(&root, "CLAUDE_CODE_MAX_OUTPUT_TOKENS"),
            "bashDefaultTimeoutMs": Self::json_env_string(&root, "BASH_DEFAULT_TIMEOUT_MS"),
            "bashMaxTimeoutMs": Self::json_env_string(&root, "BASH_MAX_TIMEOUT_MS"),
            "mcpTimeout": Self::json_env_string(&root, "MCP_TIMEOUT"),
            "mcpToolTimeout": Self::json_env_string(&root, "MCP_TOOL_TIMEOUT"),
        });

        // 反向解析代理
        let proxy = json!({
            "httpsProxy": Self::json_env_string(&root, "HTTPS_PROXY"),
            "httpProxy": Self::json_env_string(&root, "HTTP_PROXY"),
        });

        // 反向解析 extraEnv:env 里所有非受管理字段都进 extraEnv
        let extra_env = Self::collect_unmanaged_env(env);

        Ok(ParsedClaudeSettings {
            provider_name,
            model,
            reasoning: "medium".to_string(),
            base_url: resolved_base_url,
            category,
            credential_detected,
            credential_source,
            credential_token,
            config_json: json!({
                "providerKind": provider_kind,
                "category": category_from_kind(provider_kind),
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
                "modelMapping": model_mapping,
                "switches": switches,
                "numbers": numbers,
                "proxy": proxy,
                "extraEnv": extra_env,
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
        root.get("env")
            .and_then(Value::as_object)
            .and_then(|env| env.get(key))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
    }

    fn json_env_bool(root: &Value, key: &str) -> bool {
        Self::json_env_string(root, key)
            .map(|value| {
                matches!(
                    value.to_ascii_lowercase().as_str(),
                    "1" | "true" | "yes" | "on"
                )
            })
            .unwrap_or(false)
    }

    /// 用于 switches 反向解析:有值即为 true(包括 "1"/"true"/"yes"/"on"),其他都为 false。
    /// 字段不存在时返回 false(对应 UI 上开关默认关闭)。
    fn json_env_truthy(root: &Value, key: &str) -> bool {
        Self::json_env_bool(root, key)
    }

    /// 把 env 里所有非受管理字段收集到 extraEnv 透传区。
    fn collect_unmanaged_env(env: Option<&serde_json::Map<String, Value>>) -> Value {
        const MANAGED: &[&str] = &[
            "ANTHROPIC_API_KEY",
            "ANTHROPIC_AUTH_TOKEN",
            "ANTHROPIC_BASE_URL",
            "ANTHROPIC_MODEL",
            "ANTHROPIC_SMALL_FAST_MODEL",
            "ANTHROPIC_DEFAULT_HAIKU_MODEL",
            "ANTHROPIC_DEFAULT_SONNET_MODEL",
            "ANTHROPIC_DEFAULT_OPUS_MODEL",
            "CLAUDE_CODE_USE_BEDROCK",
            "CLAUDE_CODE_USE_VERTEX",
            "AWS_REGION",
            "AWS_PROFILE",
            "GOOGLE_CLOUD_PROJECT",
            "VERTEX_REGION",
            "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC",
            "CLAUDE_CODE_ATTRIBUTION_HEADER",
            "DISABLE_TELEMETRY",
            "DISABLE_NON_ESSENTIAL_MODEL_CALLS",
            "DISABLE_AUTOUPDATER",
            "DISABLE_ERROR_REPORTING",
            "CLAUDE_CODE_BETA",
            "API_TIMEOUT_MS",
            "CLAUDE_CODE_MAX_OUTPUT_TOKENS",
            "BASH_DEFAULT_TIMEOUT_MS",
            "BASH_MAX_TIMEOUT_MS",
            "MCP_TIMEOUT",
            "MCP_TOOL_TIMEOUT",
            "HTTPS_PROXY",
            "HTTP_PROXY",
        ];
        let Some(env) = env else {
            return json!({});
        };
        let mut extra = serde_json::Map::new();
        for (key, value) in env {
            if MANAGED.contains(&key.as_str()) {
                continue;
            }
            extra.insert(key.clone(), value.clone());
        }
        Value::Object(extra)
    }

    fn is_official_anthropic_url(value: &str) -> bool {
        let normalized = value.trim().trim_end_matches('/').to_ascii_lowercase();
        normalized == "https://api.anthropic.com" || normalized == "https://api.anthropic.com/v1"
    }

    fn value_string(table: &toml::map::Map<String, TomlValue>, key: &str) -> Option<String> {
        table
            .get(key)
            .and_then(TomlValue::as_str)
            .map(str::to_string)
    }

    fn value_bool(table: &toml::map::Map<String, TomlValue>, key: &str) -> Option<bool> {
        table.get(key).and_then(TomlValue::as_bool)
    }

    fn value_i64(table: &toml::map::Map<String, TomlValue>, key: &str) -> Option<i64> {
        table.get(key).and_then(TomlValue::as_integer)
    }

    fn table_string(table: &toml::map::Map<String, TomlValue>, key: &str) -> Option<String> {
        Self::value_string(table, key)
    }

    fn table_bool(table: &toml::map::Map<String, TomlValue>, key: &str) -> Option<bool> {
        Self::value_bool(table, key)
    }

    fn table_i64(table: &toml::map::Map<String, TomlValue>, key: &str) -> Option<i64> {
        Self::value_i64(table, key)
    }
}

/// 从 providerKind 推断 ProviderCategory(用于解析时初始化 config_json.category)。
fn category_from_kind(kind: &str) -> &'static str {
    match kind {
        "anthropic" => "official",
        "bedrock" | "vertex" => "cloud_provider",
        _ => "third_party",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_codex_provider_config_and_preserves_runtime_options() {
        let parsed = ProviderRuntimeService::parse_codex_config(
            r#"
model_provider = "OpenAI"
model = "gpt-5.4"
review_model = "gpt-5.4"
model_reasoning_effort = "xhigh"
disable_response_storage = true
network_access = "enabled"
windows_wsl_setup_acknowledged = true
model_context_window = 1000000
model_auto_compact_token_limit = 900000

[model_providers.OpenAI]
name = "OpenAI"
base_url = "http://43.173.89.135:8080"
wire_api = "responses"
requires_openai_auth = true
"#,
        )
        .expect("reference provider config should parse");

        assert_eq!(parsed.provider_name, "OpenAI");
        assert_eq!(parsed.model, "gpt-5.4");
        assert_eq!(parsed.reasoning, "xhigh");
        assert_eq!(parsed.base_url, "http://43.173.89.135:8080");
        assert_eq!(
            parsed
                .config_json
                .get("modelContextWindow")
                .and_then(Value::as_i64),
            Some(1_000_000)
        );
        assert_eq!(
            parsed.config_json.get("wireApi").and_then(Value::as_str),
            Some("responses")
        );
    }
}
