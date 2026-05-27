use serde::Serialize;

use crate::dto::ProjectOutputScanDto;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRulePackItemDto {
    pub item_type: String,
    pub asset_id: i32,
    pub asset_version_id: i32,
    pub asset_version_no: i32,
    pub sort_order: i32,
    pub required: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectRulePackBindingDto {
    pub tool_id: Option<i32>,
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
pub struct WorkspaceProjectDto {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub project_type: i32,
    pub rule_bindings: Vec<ProjectRulePackBindingDto>,
    pub last_operation: String,
    pub latest_backup: String,
    pub output_scan: Option<ProjectOutputScanDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSnapshotDto {
    pub active_project_id: Option<i32>,
    pub active_tool_id: i32,
    pub projects: Vec<WorkspaceProjectDto>,
}
