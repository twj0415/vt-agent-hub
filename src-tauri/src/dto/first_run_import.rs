use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FirstRunImportStatusDto {
    pub status: String,
    pub should_prompt: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FirstRunImportPreviewDto {
    pub status: String,
    pub scan_version: String,
    pub roots: Vec<FirstRunImportRootDto>,
    pub candidates: Vec<FirstRunImportCandidateDto>,
    pub warnings: Vec<String>,
    pub credential_policy: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FirstRunImportRootDto {
    pub tool: String,
    pub path: String,
    pub exists: bool,
    pub candidate_count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FirstRunImportCandidateDto {
    pub id: String,
    pub asset_type: String,
    pub target_asset_type: String,
    pub source_tool_id: i32,
    pub source_tool: String,
    pub source_kind: String,
    pub name: String,
    pub summary: String,
    pub source_path: String,
    pub relative_path: String,
    pub status: String,
    pub conflict: Option<String>,
    pub existing_id: Option<i32>,
    pub default_selected: bool,
    pub selectable: bool,
    pub recommended_action: String,
    pub content_preview: String,
    pub warnings: Vec<String>,
    pub metadata: Value,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FirstRunImportApplyInputDto {
    pub selected_ids: Vec<String>,
    pub conflict_strategy: Option<String>,
    pub confirm: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FirstRunImportApplyResultDto {
    pub imported_rules: usize,
    pub imported_skills: usize,
    pub imported_providers: usize,
    pub skipped: usize,
    pub renamed: usize,
    pub overwritten: usize,
    pub assets: Vec<FirstRunImportAppliedAssetDto>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FirstRunImportAppliedAssetDto {
    pub id: i32,
    pub asset_type: String,
    pub name: String,
    pub source_tool: String,
    pub source_path: String,
    pub operation: String,
}
