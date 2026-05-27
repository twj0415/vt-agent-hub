use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryDiagnosticIssueDto {
    pub scope: String,
    pub key: String,
    pub level: String,
    pub level_code: i32,
    pub detail: String,
    pub related_path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryDiagnosticsDto {
    pub project_count: usize,
    pub rule_count: usize,
    pub skill_count: usize,
    pub library_root: String,
    pub created_paths: Vec<String>,
    pub existing_paths: Vec<String>,
    pub issue_count: usize,
    pub health_state: String,
    pub health_state_code: i32,
    pub issues: Vec<LibraryDiagnosticIssueDto>,
}
