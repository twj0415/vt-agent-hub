mod constants;
mod credentials;
mod files;
mod parser;
mod summary;
mod validation;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde_json::json;

use crate::application::app_container::AppContainer;
use crate::application::operation_service::OperationService;
use crate::application::service_context::ServiceContext;
use crate::application::tool_service::ToolService;
use crate::core::product::PRODUCT_NAME;
use crate::core::routes::ROUTE_PRESETS;
use crate::core::tool_registry::CODEX_TOOL_ID;
use crate::dto::{
    ProviderApplyFilePreviewDto, ProviderApplyPreviewDto, ProviderApplyResultDto,
    ProviderImportDraftDto, ProviderImportInputDto, ProviderSaveInputDto, ProviderSummaryDto,
};
use crate::infrastructure::credential_store::CredentialStore;
use crate::infrastructure::database::Database;
use crate::infrastructure::project_output_repo::ProjectOutputRepo;
use crate::infrastructure::provider_repo::{
    ProviderConfigUpsert, ProviderRepo, ProviderWithConfigs,
};

use constants::CREDENTIAL_MASK;

pub struct ProviderRuntimeService {
    db: Arc<Mutex<Database>>,
    tool_service: ToolService,
}

impl ProviderRuntimeService {
    pub fn new() -> Result<Self, String> {
        Self::with_context(ServiceContext::default()?)
    }

    pub fn with_context(context: ServiceContext) -> Result<Self, String> {
        let db = context.open_db()?;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            tool_service: ToolService::new(),
        })
    }

    pub fn with_container(container: &AppContainer) -> Self {
        Self {
            db: container.db(),
            tool_service: ToolService::new(),
        }
    }

    pub fn list(&self, tool_id: Option<i32>) -> Result<Vec<ProviderSummaryDto>, String> {
        let db = self.db.lock().expect("db poisoned");
        ProviderRepo::new(&db)
            .list(tool_id)?
            .into_iter()
            .map(Self::summary)
            .collect()
    }

    pub fn import_config(
        &self,
        input: ProviderImportInputDto,
    ) -> Result<ProviderImportDraftDto, String> {
        if input.tool_id != CODEX_TOOL_ID {
            return Err(format!(
                "Tool {} provider paste import is not supported yet.",
                input.tool_id
            ));
        }
        if input.parts.is_empty() {
            return Err("Provider import requires at least one pasted config part.".to_string());
        }

        let config_content = Self::required_import_part(&input, "config")?;
        let parsed = Self::parse_codex_config(config_content)?;
        Self::validate_codex_values(&parsed.model, &parsed.reasoning, &parsed.base_url)?;

        let credential_ref = Self::generate_credential_ref(&parsed.provider_name, CODEX_TOOL_ID);
        let credential_token = Self::optional_import_part(&input, "auth")
            .map(Self::read_auth_token_from_content)
            .transpose()?
            .filter(|token| !token.trim().is_empty());
        let has_credential = credential_token.is_some();
        let detected_parts = input
            .parts
            .iter()
            .filter(|part| !part.content.trim().is_empty())
            .map(|part| part.role.trim().to_string())
            .collect::<Vec<_>>();

        let db = self.db.lock().expect("db poisoned");
        OperationService::record_simple(
            &db,
            None,
            Some(CODEX_TOOL_ID),
            None,
            "operation",
            "Provider config import",
            "provider-import-config",
            &format!(
                "Parsed provider config '{}' from pasted content.",
                parsed.provider_name
            ),
            "success",
            crate::core::status_codes::HEALTH_NORMAL,
            None,
            ROUTE_PRESETS,
        )?;

        Ok(ProviderImportDraftDto {
            source_kind: "paste".to_string(),
            detected_parts,
            tool_id: CODEX_TOOL_ID,
            schema_version: 1,
            name: parsed.provider_name.clone(),
            category: "official".to_string(),
            website: String::new(),
            note: "Imported from pasted Codex config.".to_string(),
            display_name: parsed.provider_name,
            model: parsed.model,
            reasoning: parsed.reasoning,
            base_url: parsed.base_url,
            credential_ref,
            has_credential,
            credential_token,
            config_json: parsed.config_json,
        })
    }

    pub fn save(&self, input: ProviderSaveInputDto) -> Result<ProviderSummaryDto, String> {
        Self::validate_input(&input)?;
        let mut configs = Vec::new();

        for config in input.tool_configs {
            let credential_ref = Self::resolved_credential_ref(
                config.credential_ref.as_deref(),
                &input.name,
                config.tool_id,
            );
            if let Some(token) = config
                .credential_token
                .as_deref()
                .map(str::trim)
                .filter(|token| !token.is_empty())
            {
                CredentialStore::save_provider_token(&credential_ref, token)?;
            }

            configs.push(ProviderConfigUpsert {
                id: config.id,
                tool_id: config.tool_id,
                schema_version: config.schema_version,
                display_name: config.display_name.trim().to_string(),
                model: config.model.trim().to_string(),
                reasoning: config.reasoning.trim().to_string(),
                base_url: config.base_url.trim().to_string(),
                credential_ref,
                config_json: config.config_json.unwrap_or_else(|| json!({})),
            });
        }

        let db = self.db.lock().expect("db poisoned");
        let repo = ProviderRepo::new(&db);
        let provider_id = repo.upsert_provider(
            input.id,
            input.name.trim(),
            input.category.trim(),
            input.website.trim(),
            input.note.trim(),
            &configs,
        )?;

        OperationService::record_simple(
            &db,
            None,
            None,
            None,
            "operation",
            if input.id.is_some() {
                "Provider update"
            } else {
                "Provider create"
            },
            "provider-write",
            "Saved provider and tool config in SQLite.",
            "success",
            crate::core::status_codes::HEALTH_NORMAL,
            None,
            ROUTE_PRESETS,
        )?;

        let provider = repo.find_provider(provider_id)?;
        let configs = repo.list(Some(CODEX_TOOL_ID))?;
        let matched = configs
            .into_iter()
            .find(|item| item.provider.id == provider_id)
            .unwrap_or_else(|| ProviderWithConfigs {
                provider,
                configs: Vec::new(),
            });
        Self::summary(matched)
    }

    pub fn delete(&self, provider_id: i32) -> Result<(), String> {
        let db = self.db.lock().expect("db poisoned");
        ProviderRepo::new(&db).delete_provider(provider_id)?;
        OperationService::record_simple(
            &db,
            None,
            None,
            None,
            "operation",
            "Provider delete",
            "provider-delete",
            "Deleted provider from SQLite.",
            "success",
            crate::core::status_codes::HEALTH_NORMAL,
            None,
            ROUTE_PRESETS,
        )?;
        Ok(())
    }

    pub fn duplicate(&self, provider_id: i32) -> Result<ProviderSummaryDto, String> {
        let db = self.db.lock().expect("db poisoned");
        let repo = ProviderRepo::new(&db);
        let cloned_id = repo.duplicate_provider(provider_id)?;
        let provider = repo.find_provider(cloned_id)?;
        let matched = repo
            .list(None)?
            .into_iter()
            .find(|item| item.provider.id == cloned_id)
            .unwrap_or_else(|| ProviderWithConfigs {
                provider,
                configs: Vec::new(),
            });
        Self::summary(matched)
    }

    pub fn preview_apply(&self, config_id: i32) -> Result<ProviderApplyPreviewDto, String> {
        let db = self.db.lock().expect("db poisoned");
        let repo = ProviderRepo::new(&db);
        let config = repo.find_config(config_id)?;
        let provider = repo.find_provider(config.provider_id)?;
        Self::ensure_supported_config(&config)?;

        let target_path = self.tool_service.preset_config_path(config.tool_id)?;
        let before_content = Self::read_config(&target_path)?;
        let after_content = self.tool_service.render_preset_config(
            config.tool_id,
            &Self::preset_config_input(&provider, &config),
            &before_content,
        )?;

        let mut files = vec![ProviderApplyFilePreviewDto {
            label: "config.toml".to_string(),
            target_path: target_path.display().to_string(),
            target_exists: target_path.exists(),
            backup_required: target_path.exists(),
            before_content: before_content.clone(),
            after_content: after_content.clone(),
            diff: Self::build_diff("config.toml", &before_content, &after_content),
        }];

        if Self::requires_auth(&config) {
            let _ = Self::required_provider_token(&config)?;
            let auth_path = Self::auth_path_for_config(&target_path)?;
            let auth_before = Self::read_auth_preview(&auth_path)?;
            let auth_after = Self::render_auth_json(CREDENTIAL_MASK)?;
            files.push(ProviderApplyFilePreviewDto {
                label: "auth.json".to_string(),
                target_path: auth_path.display().to_string(),
                target_exists: auth_path.exists(),
                backup_required: auth_path.exists(),
                before_content: auth_before.clone(),
                after_content: auth_after.clone(),
                diff: Self::build_diff("auth.json", &auth_before, &auth_after),
            });
        }

        OperationService::record_simple(
            &db,
            None,
            Some(config.tool_id),
            None,
            "operation",
            "Provider apply preview",
            "provider-preview",
            &format!(
                "Previewed provider '{}' against Codex config.",
                provider.name
            ),
            "success",
            crate::core::status_codes::HEALTH_NORMAL,
            Some(&target_path.display().to_string()),
            ROUTE_PRESETS,
        )?;

        Ok(ProviderApplyPreviewDto {
            tool_id: config.tool_id,
            provider_id: provider.id,
            config_id: config.id,
            provider_name: provider.name,
            target_path: target_path.display().to_string(),
            target_exists: target_path.exists(),
            backup_required: files.iter().any(|file| file.backup_required),
            before_content: before_content.clone(),
            after_content: after_content.clone(),
            diff: Self::build_diff("config.toml", &before_content, &after_content),
            files,
            warning: Some(format!(
                "Applying writes {PRODUCT_NAME} managed provider settings into config.toml and writes auth.json for the provider credential."
            )),
        })
    }

    pub fn apply(
        &self,
        config_id: i32,
        confirm_risk: bool,
    ) -> Result<ProviderApplyResultDto, String> {
        if !confirm_risk {
            return Err(
                "Risk confirmation is required before applying provider to live config."
                    .to_string(),
            );
        }

        let preview = self.preview_apply(config_id)?;
        let db = self.db.lock().expect("db poisoned");
        let repo = ProviderRepo::new(&db);
        let config = repo.find_config(config_id)?;
        let mut backup_paths = Vec::new();

        for file in &preview.files {
            if file.target_exists {
                backup_paths.push(ProjectOutputRepo::backup(
                    0,
                    &PathBuf::from(&file.target_path),
                )?);
            }
        }

        let config_path = PathBuf::from(&preview.target_path);
        Self::write_text(&config_path, &preview.after_content)?;

        if Self::requires_auth(&config) {
            let token = Self::required_provider_token(&config)?;
            let auth_path = Self::auth_path_for_config(&config_path)?;
            let auth_content = Self::render_auth_json(&token)?;
            Self::write_text(&auth_path, &auth_content)?;
        }

        repo.activate_config(preview.tool_id, preview.config_id)?;

        OperationService::record_simple(
            &db,
            None,
            Some(preview.tool_id),
            None,
            "operation",
            "Provider apply",
            "provider-apply",
            &format!(
                "Applied provider '{}' to Codex config.toml and auth.json.",
                preview.provider_name
            ),
            "success",
            crate::core::status_codes::HEALTH_NORMAL,
            Some(&preview.target_path),
            ROUTE_PRESETS,
        )?;

        Ok(ProviderApplyResultDto {
            tool_id: preview.tool_id,
            provider_id: preview.provider_id,
            config_id: preview.config_id,
            operation: "provider.apply_live_config".to_string(),
            target_path: preview.target_path,
            backup_path: backup_paths.first().cloned(),
            target_paths: preview
                .files
                .iter()
                .map(|file| file.target_path.clone())
                .collect(),
            backup_paths,
            message: "Provider applied to Codex config.toml and auth.json.".to_string(),
        })
    }
}
