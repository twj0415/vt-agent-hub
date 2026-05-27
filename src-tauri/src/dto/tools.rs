use serde::Serialize;

use crate::dto::ProjectRulePackItemDto;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolRulePackBindingDto {
    pub pack_id: i32,
    pub pack_name: String,
    pub pack_type: String,
    pub pack_version_id: i32,
    pub pack_version_no: i32,
    pub update_policy: String,
    pub enabled: bool,
    pub items: Vec<ProjectRulePackItemDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolSkillInstallDto {
    pub skill_asset_id: i32,
    pub required_version_id: Option<i32>,
    pub installed_version_id: Option<i32>,
    pub state: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolSnapshotDto {
    pub id: i32,
    pub name: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolsSnapshotDto {
    pub tools: Vec<ToolSnapshotDto>,
    pub global_rule_binding: Option<ToolRulePackBindingDto>,
    pub skill_pack_binding: Option<ToolRulePackBindingDto>,
    pub skill_installs: Vec<ToolSkillInstallDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolDiagnosticsDto {
    pub installation_detected: bool,
    pub version: String,
    pub live_config_path: String,
    pub credential_state: String,
    pub credential_state_code: i32,
    pub skill_state: String,
    pub skill_state_code: i32,
    pub project_output_state: String,
    pub project_output_state_code: i32,
    pub repair_state: String,
    pub repair_state_code: i32,
    pub repair_hint: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolActionResultDto {
    pub ok: bool,
    pub state: String,
    pub detail: String,
    pub manual_steps: Vec<String>,
}
