use crate::dto::{ProviderSummaryDto, ProviderToolConfigDto};
use crate::infrastructure::provider_repo::{ProviderToolConfigRecord, ProviderWithConfigs};

use super::ProviderRuntimeService;

impl ProviderRuntimeService {
    pub(super) fn summary(item: ProviderWithConfigs) -> Result<ProviderSummaryDto, String> {
        Ok(ProviderSummaryDto {
            id: item.provider.id,
            name: item.provider.name,
            category: item.provider.category,
            website: item.provider.website,
            note: item.provider.note,
            sort_order: item.provider.sort_order,
            configs: item
                .configs
                .into_iter()
                .map(Self::config_summary)
                .collect::<Vec<_>>(),
        })
    }

    pub(super) fn config_summary(config: ProviderToolConfigRecord) -> ProviderToolConfigDto {
        let has_credential = Self::has_provider_credential(&config.credential_ref);
        ProviderToolConfigDto {
            id: config.id,
            provider_id: config.provider_id,
            tool_id: config.tool_id,
            schema_version: config.schema_version,
            model: config.model,
            reasoning: config.reasoning,
            base_url: config.base_url,
            credential_ref: config.credential_ref,
            has_credential,
            config_json: config.config_json,
            is_active: config.is_active,
            state: config.state,
            last_check_status: config.last_check_status,
            last_check_latency_ms: config.last_check_latency_ms,
            last_check_message: config.last_check_message,
            last_checked_at: config.last_checked_at,
        }
    }
}
