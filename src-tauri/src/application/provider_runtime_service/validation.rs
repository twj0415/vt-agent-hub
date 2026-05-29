use crate::adapters::tool_adapter::PresetConfigBuildInput;
use crate::core::tool_registry::{CLAUDE_TOOL_ID, CODEX_TOOL_ID};
use crate::dto::ProviderSaveInputDto;
use crate::infrastructure::credential_store::CredentialStore;
use crate::infrastructure::provider_repo::{ProviderRecord, ProviderToolConfigRecord};

use super::constants::{CODEX_MODELS, CODEX_REASONING};
use super::ProviderRuntimeService;

impl ProviderRuntimeService {
    pub(super) fn validate_input(input: &ProviderSaveInputDto) -> Result<(), String> {
        if input.name.trim().is_empty() {
            return Err("Provider name is required.".to_string());
        }
        if input.category.trim().is_empty() {
            return Err("Provider category is required.".to_string());
        }
        if input.tool_configs.is_empty() {
            return Err("At least one tool config is required.".to_string());
        }
        for config in &input.tool_configs {
            match config.tool_id {
                CODEX_TOOL_ID => Self::validate_codex_values(
                    config.model.trim(),
                    config.reasoning.trim(),
                    config.base_url.trim(),
                )?,
                CLAUDE_TOOL_ID => Self::validate_claude_values(
                    config.model.trim(),
                    config.reasoning.trim(),
                    config.base_url.trim(),
                )?,
                _ => {
                    return Err(format!(
                        "Tool {} provider config is reserved but cannot be saved yet.",
                        config.tool_id
                    ));
                }
            }
        }
        Ok(())
    }

    pub(super) fn validate_codex_values(
        model: &str,
        reasoning: &str,
        base_url: &str,
    ) -> Result<(), String> {
        if model.trim().is_empty() {
            return Err("Provider model is required.".to_string());
        }
        if !CODEX_MODELS.contains(&model.trim()) {
            return Err(format!("Provider model '{model}' is not supported."));
        }
        if !CODEX_REASONING.contains(&reasoning.trim()) {
            return Err(format!(
                "Provider reasoning '{reasoning}' is not supported."
            ));
        }
        if !base_url.starts_with("http://") && !base_url.starts_with("https://") {
            return Err("Provider base URL must start with http:// or https://.".to_string());
        }
        Ok(())
    }

    fn validate_claude_values(model: &str, reasoning: &str, base_url: &str) -> Result<(), String> {
        if model.trim().is_empty() {
            return Err("Provider model is required.".to_string());
        }
        if reasoning.trim().is_empty() {
            return Err("Provider reasoning is required.".to_string());
        }
        if base_url.trim().is_empty() {
            return Err("Provider base URL is required.".to_string());
        }
        Ok(())
    }

    pub(super) fn ensure_supported_config(config: &ProviderToolConfigRecord) -> Result<(), String> {
        if !matches!(config.tool_id, CODEX_TOOL_ID | CLAUDE_TOOL_ID) {
            return Err(format!(
                "Tool {} provider apply is not supported yet.",
                config.tool_id
            ));
        }
        if config.schema_version != 1 {
            return Err(format!(
                "Provider config schema version {} is not supported.",
                config.schema_version
            ));
        }
        Ok(())
    }

    pub(super) fn preset_config_input(
        provider: &ProviderRecord,
        config: &ProviderToolConfigRecord,
    ) -> PresetConfigBuildInput {
        PresetConfigBuildInput {
            name: provider.name.clone(),
            provider: provider.name.clone(),
            model: config.model.clone(),
            reasoning: config.reasoning.clone(),
            base_url: config.base_url.clone(),
            credential_token: CredentialStore::load_provider_token(&config.credential_ref).ok().flatten(),
            config_json: config.config_json.clone(),
        }
    }

}
