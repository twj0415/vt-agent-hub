use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntryDto {
    pub id: i32,
    pub project_id: Option<i32>,
    pub tool_id: Option<i32>,
    pub related_rule_id: Option<i32>,
    pub kind: String,
    pub title: String,
    pub created_at: String,
    pub action: String,
    pub result: String,
    pub level: String,
    pub level_code: i32,
    pub detail: String,
    pub related_path: String,
    pub navigation_target: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistoryFilterDto {
    pub project_ids: Vec<i32>,
    pub tool_ids: Vec<i32>,
    pub kinds: Vec<String>,
    pub results: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HistorySnapshotDto {
    pub entries: Vec<HistoryEntryDto>,
    pub filters: HistoryFilterDto,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupEntryDto {
    pub id: String,
    pub scope: String,
    pub project_id: Option<i32>,
    pub file_name: String,
    pub path: String,
    pub target_path: String,
    pub created_at: String,
    pub size: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupSnapshotDto {
    pub entries: Vec<BackupEntryDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupRestorePreviewDto {
    pub backup_id: String,
    pub backup_path: String,
    pub target_path: String,
    pub target_exists: bool,
    pub before_content: String,
    pub after_content: String,
    pub diff: String,
    pub warning: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupActionResultDto {
    pub ok: bool,
    pub message: String,
    pub path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticExportResultDto {
    pub path: String,
    pub issue_count: usize,
    pub message: String,
}
