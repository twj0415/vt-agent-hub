use serde_json::{json, Value};
use toml::Value as TomlValue;

use super::constants::DEFAULT_BASE_URL;
use super::ProviderRuntimeService;

#[derive(Debug, Clone)]
pub(super) struct ParsedCodexConfig {
    pub(super) provider_name: String,
    pub(super) model: String,
    pub(super) reasoning: String,
    pub(super) base_url: String,
    pub(super) config_json: Value,
}

impl ProviderRuntimeService {
    pub(super) fn parse_codex_config(content: &str) -> Result<ParsedCodexConfig, String> {
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
