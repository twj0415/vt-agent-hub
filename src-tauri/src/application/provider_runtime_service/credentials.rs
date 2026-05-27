use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use crate::dto::ProviderImportInputDto;
use crate::infrastructure::credential_store::CredentialStore;
use crate::infrastructure::provider_repo::ProviderToolConfigRecord;

use super::ProviderRuntimeService;

impl ProviderRuntimeService {
    pub(super) fn required_import_part<'a>(
        input: &'a ProviderImportInputDto,
        role: &str,
    ) -> Result<&'a str, String> {
        Self::optional_import_part(input, role)
            .filter(|content| !content.trim().is_empty())
            .ok_or_else(|| format!("Provider import requires pasted {role} content."))
    }

    pub(super) fn optional_import_part<'a>(
        input: &'a ProviderImportInputDto,
        role: &str,
    ) -> Option<&'a str> {
        input
            .parts
            .iter()
            .find(|part| part.role.trim().eq_ignore_ascii_case(role))
            .map(|part| part.content.trim())
    }

    pub(super) fn read_auth_token_from_content(content: &str) -> Result<String, String> {
        let value = serde_json::from_str::<Value>(content)
            .map_err(|error| format!("Provider auth JSON is invalid: {error}"))?;
        Ok(value
            .get("OPENAI_API_KEY")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string())
    }

    pub(super) fn render_auth_json(token: &str) -> Result<String, String> {
        serde_json::to_string_pretty(&json!({ "OPENAI_API_KEY": token }))
            .map(|value| value + "\n")
            .map_err(|error| error.to_string())
    }
    pub(super) fn requires_auth(config: &ProviderToolConfigRecord) -> bool {
        config
            .config_json
            .get("requiresOpenaiAuth")
            .and_then(Value::as_bool)
            .unwrap_or(true)
    }

    pub(super) fn required_provider_token(
        config: &ProviderToolConfigRecord,
    ) -> Result<String, String> {
        let token = CredentialStore::load_provider_token(&config.credential_ref)?;
        token
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| "Provider credential is required before applying.".to_string())
    }

    pub(super) fn has_provider_credential(credential_ref: &str) -> bool {
        CredentialStore::load_provider_token(credential_ref)
            .ok()
            .flatten()
            .map(|token| !token.trim().is_empty())
            .unwrap_or(false)
    }

    pub(super) fn resolved_credential_ref(
        input: Option<&str>,
        provider_name: &str,
        tool_id: i32,
    ) -> String {
        input
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(Self::sanitize_credential_ref)
            .unwrap_or_else(|| Self::generate_credential_ref(provider_name, tool_id))
    }

    pub(super) fn generate_credential_ref(provider_name: &str, tool_id: i32) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .unwrap_or(0);
        format!(
            "tool-{tool_id}-provider-{}-{timestamp}",
            Self::slug(provider_name)
        )
    }

    fn sanitize_credential_ref(value: &str) -> String {
        value
            .trim()
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                    ch
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .trim_matches('-')
            .to_string()
    }

    fn slug(value: &str) -> String {
        let slug = value
            .trim()
            .to_ascii_lowercase()
            .chars()
            .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join("-");
        if slug.is_empty() {
            "provider".to_string()
        } else {
            slug
        }
    }
}
