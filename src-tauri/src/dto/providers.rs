use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderToolConfigInputDto {
    pub id: Option<i32>,
    pub tool_id: i32,
    pub schema_version: i32,
    pub display_name: String,
    pub model: String,
    pub reasoning: String,
    pub base_url: String,
    pub credential_ref: Option<String>,
    pub credential_token: Option<String>,
    pub config_json: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderSaveInputDto {
    pub id: Option<i32>,
    pub name: String,
    pub category: String,
    pub website: String,
    pub note: String,
    pub tool_configs: Vec<ProviderToolConfigInputDto>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderImportInputPartDto {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderImportInputDto {
    pub tool_id: i32,
    pub parts: Vec<ProviderImportInputPartDto>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderToolConfigDto {
    pub id: i32,
    pub provider_id: i32,
    pub tool_id: i32,
    pub schema_version: i32,
    pub display_name: String,
    pub model: String,
    pub reasoning: String,
    pub base_url: String,
    pub credential_ref: String,
    pub has_credential: bool,
    pub config_json: Value,
    pub is_active: bool,
    pub state: i32,
    pub last_check_status: String,
    pub last_check_latency_ms: Option<i32>,
    pub last_check_message: String,
    pub last_checked_at: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderSummaryDto {
    pub id: i32,
    pub name: String,
    pub category: String,
    pub website: String,
    pub note: String,
    pub sort_order: i32,
    pub configs: Vec<ProviderToolConfigDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderImportDraftDto {
    pub source_kind: String,
    pub detected_parts: Vec<String>,
    pub tool_id: i32,
    pub schema_version: i32,
    pub name: String,
    pub category: String,
    pub website: String,
    pub note: String,
    pub display_name: String,
    pub model: String,
    pub reasoning: String,
    pub base_url: String,
    pub credential_ref: String,
    pub has_credential: bool,
    pub credential_token: Option<String>,
    pub config_json: Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderApplyFilePreviewDto {
    pub label: String,
    pub target_path: String,
    pub target_exists: bool,
    pub backup_required: bool,
    pub before_content: String,
    pub after_content: String,
    pub diff: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderApplyPreviewDto {
    pub tool_id: i32,
    pub provider_id: i32,
    pub config_id: i32,
    pub provider_name: String,
    pub target_path: String,
    pub target_exists: bool,
    pub backup_required: bool,
    pub before_content: String,
    pub after_content: String,
    pub diff: String,
    pub files: Vec<ProviderApplyFilePreviewDto>,
    pub warning: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderApplyResultDto {
    pub tool_id: i32,
    pub provider_id: i32,
    pub config_id: i32,
    pub operation: String,
    pub target_path: String,
    pub backup_path: Option<String>,
    pub target_paths: Vec<String>,
    pub backup_paths: Vec<String>,
    pub message: String,
}
