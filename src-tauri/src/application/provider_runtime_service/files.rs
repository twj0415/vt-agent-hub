use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{json, Map, Value};

use crate::adapters::tool_adapter::PresetConfigBuildInput;
use crate::core::paths;

use super::ProviderRuntimeService;

/// 受管理的 env 字段:apply 时会清理重写。其余字段(包括用户在 extraEnv 里手填的)按透传保留。
const CLAUDE_MANAGED_ENV_KEYS: &[&str] = &[
    // 凭证
    "ANTHROPIC_API_KEY",
    "ANTHROPIC_AUTH_TOKEN",
    // 端点
    "ANTHROPIC_BASE_URL",
    // 模型映射
    "ANTHROPIC_MODEL",
    "ANTHROPIC_SMALL_FAST_MODEL",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL",
    "ANTHROPIC_DEFAULT_SONNET_MODEL",
    "ANTHROPIC_DEFAULT_OPUS_MODEL",
    // 云服务
    "CLAUDE_CODE_USE_BEDROCK",
    "CLAUDE_CODE_USE_VERTEX",
    "AWS_REGION",
    "AWS_PROFILE",
    "GOOGLE_CLOUD_PROJECT",
    "VERTEX_REGION",
    // 行为开关
    "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC",
    "CLAUDE_CODE_ATTRIBUTION_HEADER",
    "DISABLE_TELEMETRY",
    "DISABLE_NON_ESSENTIAL_MODEL_CALLS",
    "DISABLE_AUTOUPDATER",
    "DISABLE_ERROR_REPORTING",
    "CLAUDE_CODE_BETA",
    // 性能/超时
    "API_TIMEOUT_MS",
    "CLAUDE_CODE_MAX_OUTPUT_TOKENS",
    "BASH_DEFAULT_TIMEOUT_MS",
    "BASH_MAX_TIMEOUT_MS",
    "MCP_TIMEOUT",
    "MCP_TOOL_TIMEOUT",
    // 网络
    "HTTPS_PROXY",
    "HTTP_PROXY",
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

        // 1. 保留现有凭证(后面可能在没有新 token 时复用)
        let existing_api_key = string_from(env.get("ANTHROPIC_API_KEY"));
        let existing_auth_token = string_from(env.get("ANTHROPIC_AUTH_TOKEN"));

        // 2. 清理所有受管理字段(其它字段一律透传)
        for key in CLAUDE_MANAGED_ENV_KEYS {
            env.remove(*key);
        }

        // 3. 写入模型映射
        let model = input.model.trim();
        let mapping = input.config_json.get("modelMapping");
        let opus = mapping_string(mapping, "opus").unwrap_or_else(|| model.to_string());
        let sonnet = mapping_string(mapping, "sonnet").unwrap_or_else(|| model.to_string());
        let haiku = mapping_string(mapping, "haiku").unwrap_or_else(|| model.to_string());
        let small_fast = mapping_string(mapping, "smallFast")
            .or_else(|| config_string(&input.config_json, "smallFastModel"));

        env.insert("ANTHROPIC_MODEL".to_string(), Value::String(model.to_string()));
        env.insert(
            "ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(),
            Value::String(opus),
        );
        env.insert(
            "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
            Value::String(sonnet),
        );
        env.insert(
            "ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(),
            Value::String(haiku.clone()),
        );
        if let Some(small_fast) = small_fast {
            env.insert(
                "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
                Value::String(small_fast),
            );
        }

        // 4. 推断 providerKind(用户没指定时按 base_url 推断)
        let base_url = input.base_url.trim();
        let provider_kind = input
            .config_json
            .get("providerKind")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| infer_provider_kind(base_url));

        // 5. 写入凭证(按优先级:显式 credentialKey > category 推断 > providerKind 兜底)
        let credential_key = resolve_credential_key(
            input.config_json.get("credentialKey").and_then(Value::as_str),
            input.config_json.get("category").and_then(Value::as_str),
            &provider_kind,
        );

        if let Some(token) = input
            .credential_token
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            env.insert(credential_key.to_string(), Value::String(token.to_string()));
        } else {
            // 没传 token 时尝试沿用旧值,但要写到当前 credential_key 字段下
            let preserved = if credential_key == "ANTHROPIC_API_KEY" {
                existing_api_key.or(existing_auth_token)
            } else {
                existing_auth_token.or(existing_api_key)
            };
            if let Some(value) = preserved {
                env.insert(credential_key.to_string(), Value::String(value));
            }
        }

        // 6. 写入云服务/网关端点
        match provider_kind.as_str() {
            "bedrock" => {
                env.insert(
                    "CLAUDE_CODE_USE_BEDROCK".to_string(),
                    Value::String("true".to_string()),
                );
                if let Some(aws_region) = config_string(&input.config_json, "awsRegion")
                    .or_else(|| single_display_url_part(base_url, "bedrock://"))
                {
                    env.insert("AWS_REGION".to_string(), Value::String(aws_region));
                }
                if let Some(aws_profile) = config_string(&input.config_json, "awsProfile") {
                    env.insert("AWS_PROFILE".to_string(), Value::String(aws_profile));
                }
            }
            "vertex" => {
                env.insert(
                    "CLAUDE_CODE_USE_VERTEX".to_string(),
                    Value::String("true".to_string()),
                );
                let (display_project, display_region) = vertex_display_url_parts(base_url);
                if let Some(project) =
                    config_string(&input.config_json, "googleCloudProject").or(display_project)
                {
                    env.insert("GOOGLE_CLOUD_PROJECT".to_string(), Value::String(project));
                }
                if let Some(vertex_region) =
                    config_string(&input.config_json, "vertexRegion").or(display_region)
                {
                    env.insert("VERTEX_REGION".to_string(), Value::String(vertex_region));
                }
            }
            _ => {
                env.insert(
                    "ANTHROPIC_BASE_URL".to_string(),
                    Value::String(base_url.to_string()),
                );
            }
        }

        // 7. 写入行为开关(switches: { camelCaseKey: bool })
        if let Some(switches) = input.config_json.get("switches").and_then(Value::as_object) {
            for (camel_key, env_key) in SWITCH_KEYS {
                if let Some(value) = switches.get(*camel_key) {
                    if let Some(rendered) = render_switch_value(value) {
                        env.insert(env_key.to_string(), Value::String(rendered));
                    }
                }
            }
        }

        // 8. 写入数值字段(numbers: { camelCaseKey: number|string })
        if let Some(numbers) = input.config_json.get("numbers").and_then(Value::as_object) {
            for (camel_key, env_key) in NUMBER_KEYS {
                if let Some(value) = numbers.get(*camel_key) {
                    if let Some(rendered) = render_number_value(value) {
                        env.insert(env_key.to_string(), Value::String(rendered));
                    }
                }
            }
        }

        // 9. 写入代理设置
        if let Some(proxy) = input.config_json.get("proxy").and_then(Value::as_object) {
            if let Some(value) = proxy.get("httpsProxy").and_then(Value::as_str) {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    env.insert("HTTPS_PROXY".to_string(), Value::String(trimmed.to_string()));
                }
            }
            if let Some(value) = proxy.get("httpProxy").and_then(Value::as_str) {
                let trimmed = value.trim();
                if !trimmed.is_empty() {
                    env.insert("HTTP_PROXY".to_string(), Value::String(trimmed.to_string()));
                }
            }
        }

        // 10. 透传 extraEnv(自由 KV,但跳过任何受管理字段以防冲突)
        if let Some(extra) = input.config_json.get("extraEnv").and_then(Value::as_object) {
            for (key, value) in extra {
                if CLAUDE_MANAGED_ENV_KEYS.contains(&key.as_str()) {
                    continue;
                }
                let rendered = match value {
                    Value::String(s) => {
                        let trimmed = s.trim();
                        if trimmed.is_empty() {
                            continue;
                        }
                        Value::String(trimmed.to_string())
                    }
                    Value::Bool(b) => Value::String(b.to_string()),
                    Value::Number(n) => Value::String(n.to_string()),
                    Value::Null => continue,
                    other => other.clone(),
                };
                env.insert(key.clone(), rendered);
            }
        }

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

/// 行为开关:(config_json.switches 的 camelCase key, settings.env 的 ENV 名)
const SWITCH_KEYS: &[(&str, &str)] = &[
    ("disableNonessentialTraffic", "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC"),
    ("attributionHeader", "CLAUDE_CODE_ATTRIBUTION_HEADER"),
    ("disableTelemetry", "DISABLE_TELEMETRY"),
    ("disableNonEssentialModelCalls", "DISABLE_NON_ESSENTIAL_MODEL_CALLS"),
    ("disableAutoupdater", "DISABLE_AUTOUPDATER"),
    ("disableErrorReporting", "DISABLE_ERROR_REPORTING"),
    ("claudeCodeBeta", "CLAUDE_CODE_BETA"),
];

/// 数值字段:(config_json.numbers 的 camelCase key, settings.env 的 ENV 名)
const NUMBER_KEYS: &[(&str, &str)] = &[
    ("apiTimeoutMs", "API_TIMEOUT_MS"),
    ("claudeCodeMaxOutputTokens", "CLAUDE_CODE_MAX_OUTPUT_TOKENS"),
    ("bashDefaultTimeoutMs", "BASH_DEFAULT_TIMEOUT_MS"),
    ("bashMaxTimeoutMs", "BASH_MAX_TIMEOUT_MS"),
    ("mcpTimeout", "MCP_TIMEOUT"),
    ("mcpToolTimeout", "MCP_TOOL_TIMEOUT"),
];

fn string_from(value: Option<&Value>) -> Option<String> {
    value
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn mapping_string(mapping: Option<&Value>, key: &str) -> Option<String> {
    mapping
        .and_then(Value::as_object)
        .and_then(|map| map.get(key))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
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

fn infer_provider_kind(base_url: &str) -> String {
    let normalized = base_url.trim_end_matches('/').to_ascii_lowercase();
    if normalized.starts_with("bedrock://") {
        "bedrock".to_string()
    } else if normalized.starts_with("vertex://") {
        "vertex".to_string()
    } else if normalized == "https://api.anthropic.com" || normalized == "https://api.anthropic.com/v1"
    {
        "anthropic".to_string()
    } else {
        "anthropic-compatible".to_string()
    }
}

/// 凭证字段名解析优先级:
/// 1. config_json.credentialKey 显式指定
/// 2. category 推断(official → API_KEY, 其它 → AUTH_TOKEN)
/// 3. providerKind 兜底(anthropic → API_KEY, 其它 → AUTH_TOKEN)
fn resolve_credential_key(
    explicit: Option<&str>,
    category: Option<&str>,
    provider_kind: &str,
) -> &'static str {
    if let Some(value) = explicit.map(str::trim).filter(|value| !value.is_empty()) {
        if value == "ANTHROPIC_API_KEY" {
            return "ANTHROPIC_API_KEY";
        }
        if value == "ANTHROPIC_AUTH_TOKEN" {
            return "ANTHROPIC_AUTH_TOKEN";
        }
    }
    if let Some(category) = category {
        match category {
            "official" => return "ANTHROPIC_API_KEY",
            "cn_official" | "aggregator" | "third_party" | "custom" => {
                return "ANTHROPIC_AUTH_TOKEN"
            }
            _ => {}
        }
    }
    if provider_kind == "anthropic" {
        "ANTHROPIC_API_KEY"
    } else {
        "ANTHROPIC_AUTH_TOKEN"
    }
}

fn render_switch_value(value: &Value) -> Option<String> {
    match value {
        Value::Bool(b) => Some(if *b { "1".to_string() } else { "0".to_string() }),
        Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

fn render_number_value(value: &Value) -> Option<String> {
    match value {
        Value::Number(n) => Some(n.to_string()),
        Value::String(s) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        _ => None,
    }
}

// 保持 Map 类型 import 以备扩展(目前不直接使用但 SWITCH_KEYS / NUMBER_KEYS 表本质上是 Map)
#[allow(dead_code)]
fn _ensure_map(_: Map<String, Value>) {}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use crate::adapters::tool_adapter::PresetConfigBuildInput;
    use crate::application::provider_runtime_service::ProviderRuntimeService;

    fn build_input(category: &str, base_url: &str, config_extra: serde_json::Value) -> PresetConfigBuildInput {
        let mut config_json = json!({ "category": category });
        if let Some(extra) = config_extra.as_object() {
            for (k, v) in extra {
                config_json[k] = v.clone();
            }
        }
        PresetConfigBuildInput {
            name: "Test".to_string(),
            provider: "Test".to_string(),
            model: "claude-opus-4-7".to_string(),
            reasoning: "medium".to_string(),
            base_url: base_url.to_string(),
            credential_token: Some("sk-test-token".to_string()),
            config_json,
        }
    }

    #[test]
    fn credential_key_defaults_to_api_key_for_official_category() {
        let rendered = ProviderRuntimeService::render_claude_settings(
            "",
            &build_input("official", "https://api.anthropic.com", json!({})),
        )
        .expect("render should succeed");
        let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        let env = parsed.get("env").and_then(serde_json::Value::as_object).unwrap();
        assert_eq!(
            env.get("ANTHROPIC_API_KEY").and_then(serde_json::Value::as_str),
            Some("sk-test-token")
        );
        assert!(env.get("ANTHROPIC_AUTH_TOKEN").is_none());
    }

    #[test]
    fn credential_key_defaults_to_auth_token_for_third_party_category() {
        let rendered = ProviderRuntimeService::render_claude_settings(
            "",
            &build_input("third_party", "https://my-proxy.example.com", json!({})),
        )
        .expect("render should succeed");
        let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        let env = parsed.get("env").and_then(serde_json::Value::as_object).unwrap();
        assert_eq!(
            env.get("ANTHROPIC_AUTH_TOKEN").and_then(serde_json::Value::as_str),
            Some("sk-test-token")
        );
        assert!(env.get("ANTHROPIC_API_KEY").is_none());
    }

    #[test]
    fn credential_key_defaults_to_auth_token_for_cn_official_category() {
        let rendered = ProviderRuntimeService::render_claude_settings(
            "",
            &build_input("cn_official", "https://api.deepseek.com/anthropic", json!({})),
        )
        .expect("render should succeed");
        let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        let env = parsed.get("env").and_then(serde_json::Value::as_object).unwrap();
        assert!(env.get("ANTHROPIC_AUTH_TOKEN").is_some());
        assert!(env.get("ANTHROPIC_API_KEY").is_none());
    }

    #[test]
    fn explicit_credential_key_overrides_category_default() {
        let rendered = ProviderRuntimeService::render_claude_settings(
            "",
            &build_input(
                "official",
                "https://api.anthropic.com",
                json!({ "credentialKey": "ANTHROPIC_AUTH_TOKEN" }),
            ),
        )
        .expect("render should succeed");
        let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        let env = parsed.get("env").and_then(serde_json::Value::as_object).unwrap();
        assert!(env.get("ANTHROPIC_AUTH_TOKEN").is_some());
        assert!(env.get("ANTHROPIC_API_KEY").is_none());
    }

    #[test]
    fn model_mapping_writes_all_anthropic_default_models() {
        let rendered = ProviderRuntimeService::render_claude_settings(
            "",
            &build_input(
                "third_party",
                "https://my-proxy.example.com",
                json!({
                    "modelMapping": {
                        "opus": "claude-opus-4-7",
                        "sonnet": "claude-sonnet-4-6",
                        "haiku": "claude-haiku-4-5-20251001",
                        "smallFast": "claude-haiku-4-5-20251001"
                    }
                }),
            ),
        )
        .expect("render should succeed");
        let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        let env = parsed.get("env").and_then(serde_json::Value::as_object).unwrap();
        assert_eq!(
            env.get("ANTHROPIC_DEFAULT_OPUS_MODEL").and_then(serde_json::Value::as_str),
            Some("claude-opus-4-7")
        );
        assert_eq!(
            env.get("ANTHROPIC_DEFAULT_SONNET_MODEL").and_then(serde_json::Value::as_str),
            Some("claude-sonnet-4-6")
        );
        assert_eq!(
            env.get("ANTHROPIC_DEFAULT_HAIKU_MODEL").and_then(serde_json::Value::as_str),
            Some("claude-haiku-4-5-20251001")
        );
        assert_eq!(
            env.get("ANTHROPIC_SMALL_FAST_MODEL").and_then(serde_json::Value::as_str),
            Some("claude-haiku-4-5-20251001")
        );
    }

    #[test]
    fn switches_render_as_zero_one_strings() {
        let rendered = ProviderRuntimeService::render_claude_settings(
            "",
            &build_input(
                "third_party",
                "https://my-proxy.example.com",
                json!({
                    "switches": {
                        "disableNonessentialTraffic": true,
                        "attributionHeader": false,
                        "disableTelemetry": true
                    }
                }),
            ),
        )
        .expect("render should succeed");
        let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        let env = parsed.get("env").and_then(serde_json::Value::as_object).unwrap();
        assert_eq!(
            env.get("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC").and_then(serde_json::Value::as_str),
            Some("1")
        );
        assert_eq!(
            env.get("CLAUDE_CODE_ATTRIBUTION_HEADER").and_then(serde_json::Value::as_str),
            Some("0")
        );
        assert_eq!(
            env.get("DISABLE_TELEMETRY").and_then(serde_json::Value::as_str),
            Some("1")
        );
    }

    #[test]
    fn numbers_render_as_strings() {
        let rendered = ProviderRuntimeService::render_claude_settings(
            "",
            &build_input(
                "third_party",
                "https://my-proxy.example.com",
                json!({
                    "numbers": {
                        "apiTimeoutMs": 60000,
                        "claudeCodeMaxOutputTokens": 8192
                    }
                }),
            ),
        )
        .expect("render should succeed");
        let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        let env = parsed.get("env").and_then(serde_json::Value::as_object).unwrap();
        assert_eq!(
            env.get("API_TIMEOUT_MS").and_then(serde_json::Value::as_str),
            Some("60000")
        );
        assert_eq!(
            env.get("CLAUDE_CODE_MAX_OUTPUT_TOKENS").and_then(serde_json::Value::as_str),
            Some("8192")
        );
    }

    #[test]
    fn proxy_renders_into_https_and_http_proxy() {
        let rendered = ProviderRuntimeService::render_claude_settings(
            "",
            &build_input(
                "third_party",
                "https://my-proxy.example.com",
                json!({
                    "proxy": {
                        "httpsProxy": "http://127.0.0.1:7890",
                        "httpProxy": "http://127.0.0.1:7890"
                    }
                }),
            ),
        )
        .expect("render should succeed");
        let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        let env = parsed.get("env").and_then(serde_json::Value::as_object).unwrap();
        assert_eq!(
            env.get("HTTPS_PROXY").and_then(serde_json::Value::as_str),
            Some("http://127.0.0.1:7890")
        );
        assert_eq!(
            env.get("HTTP_PROXY").and_then(serde_json::Value::as_str),
            Some("http://127.0.0.1:7890")
        );
    }

    #[test]
    fn extra_env_passthrough_preserves_user_keys_but_skips_managed() {
        let rendered = ProviderRuntimeService::render_claude_settings(
            "",
            &build_input(
                "third_party",
                "https://my-proxy.example.com",
                json!({
                    "extraEnv": {
                        "MY_CUSTOM_KEY": "custom-value",
                        "ANTHROPIC_API_KEY": "should-be-dropped-by-managed-filter"
                    }
                }),
            ),
        )
        .expect("render should succeed");
        let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        let env = parsed.get("env").and_then(serde_json::Value::as_object).unwrap();
        assert_eq!(
            env.get("MY_CUSTOM_KEY").and_then(serde_json::Value::as_str),
            Some("custom-value")
        );
        // 受管理字段不会被 extraEnv 覆盖
        assert_ne!(
            env.get("ANTHROPIC_API_KEY").and_then(serde_json::Value::as_str),
            Some("should-be-dropped-by-managed-filter")
        );
    }

    #[test]
    fn unrelated_existing_settings_are_preserved() {
        let rendered = ProviderRuntimeService::render_claude_settings(
            r#"{ "env": { "UNRELATED": "keep-me" }, "permissions": { "allow": ["bash"] } }"#,
            &build_input("official", "https://api.anthropic.com", json!({})),
        )
        .expect("render should succeed");
        let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        let env = parsed.get("env").and_then(serde_json::Value::as_object).unwrap();
        assert_eq!(
            env.get("UNRELATED").and_then(serde_json::Value::as_str),
            Some("keep-me")
        );
        assert!(parsed.get("permissions").is_some());
    }

    #[test]
    fn does_not_force_top_level_model_to_sonnet() {
        let rendered = ProviderRuntimeService::render_claude_settings(
            r#"{ "model": "opus" }"#,
            &build_input("official", "https://api.anthropic.com", json!({})),
        )
        .expect("render should succeed");
        let parsed: serde_json::Value = serde_json::from_str(&rendered).unwrap();
        // 用户原本设置的 top-level model 应被保留而不是被改为 sonnet
        assert_eq!(parsed.get("model").and_then(serde_json::Value::as_str), Some("opus"));
    }
}
