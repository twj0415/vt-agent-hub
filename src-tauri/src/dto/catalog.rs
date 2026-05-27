use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillFileNodeDto {
    pub path: String,
    pub is_dir: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillRuntimeDto {
    pub platform_root: String,
    pub library_path: String,
    pub library_skill_md_path: String,
    pub runtime_path: String,
    pub runtime_skill_md_path: String,
    pub library_exists: bool,
    pub runtime_exists: bool,
    pub skill_md_valid: bool,
    pub install_state: i32,
    pub status_detail: String,
    pub library_body: String,
    pub runtime_body: String,
    pub library_tree: Vec<SkillFileNodeDto>,
    pub runtime_tree: Vec<SkillFileNodeDto>,
    pub install_action_ready: bool,
    pub uninstall_action_ready: bool,
    pub repair_action_ready: bool,
    pub mark_stale_action_ready: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleSummaryDto {
    pub asset_id: i32,
    pub version_id: i32,
    pub version_no: i32,
    pub key: String,
    pub code: i32,
    pub name: String,
    pub category_code: i32,
    pub state: i32,
    pub sort_order: i32,
    pub summary: String,
    pub body: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SkillSummaryDto {
    pub asset_id: i32,
    pub version_id: i32,
    pub version_no: i32,
    pub key: String,
    pub code: i32,
    pub name: String,
    pub category_code: i32,
    pub state: i32,
    pub summary: String,
    pub body: String,
    pub runtime: SkillRuntimeDto,
    pub tool_ids: Vec<i32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CatalogSnapshotDto {
    pub rules: Vec<RuleSummaryDto>,
    pub skills: Vec<SkillSummaryDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleImpactDto {
    pub rule_asset_id: i32,
    pub rule_name: String,
    pub bound_project_count: usize,
    pub bound_tool_count: usize,
    pub project_names: Vec<String>,
    pub tool_ids: Vec<i32>,
    pub project_tool_ids: Vec<i32>,
    pub global_tool_ids: Vec<i32>,
    pub requires_project_regeneration: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleImportResultDto {
    pub rule: RuleSummaryDto,
    pub source_path: String,
    pub imported_name: String,
    pub operation: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleImportPreviewDto {
    pub source_path: String,
    pub name: String,
    pub summary: String,
    pub body: String,
}
