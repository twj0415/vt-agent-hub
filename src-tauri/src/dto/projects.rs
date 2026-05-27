use serde::Serialize;

use crate::dto::ProjectRulePackBindingDto;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectDetailDto {
    pub id: i32,
    pub name: String,
    pub path: String,
    pub project_type: i32,
    pub rule_bindings: Vec<ProjectRulePackBindingDto>,
    pub last_operation: String,
    pub latest_backup: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectOutputScanDto {
    pub project_id: i32,
    pub tool_id: i32,
    pub project_name: String,
    pub target_path: String,
    pub target_exists: bool,
    pub managed: bool,
    pub rule_count: usize,
    pub status: String,
    pub status_code: i32,
    pub issues: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectOutputPreviewDto {
    pub project_id: i32,
    pub tool_id: i32,
    pub project_name: String,
    pub target_path: String,
    pub target_exists: bool,
    pub managed: bool,
    pub rule_count: usize,
    pub backup_required: bool,
    pub can_apply: bool,
    pub warning: Option<String>,
    pub before_content: String,
    pub after_content: String,
    pub diff: String,
    pub issues: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectOutputWriteDto {
    pub project_id: i32,
    pub tool_id: i32,
    pub operation: String,
    pub target_path: String,
    pub backup_path: Option<String>,
    pub managed: bool,
    pub created: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalOutputPreviewDto {
    pub tool_id: i32,
    pub target_path: String,
    pub target_exists: bool,
    pub managed: bool,
    pub rule_count: usize,
    pub backup_required: bool,
    pub can_apply: bool,
    pub warning: Option<String>,
    pub before_content: String,
    pub after_content: String,
    pub diff: String,
    pub issues: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GlobalOutputWriteDto {
    pub tool_id: i32,
    pub operation: String,
    pub target_path: String,
    pub backup_path: Option<String>,
    pub managed: bool,
    pub created: bool,
    pub message: String,
}
