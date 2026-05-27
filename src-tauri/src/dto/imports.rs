use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryImportAssetDto {
    pub asset_type: String,
    pub name: String,
    pub source_path: String,
    pub status: String,
    pub conflict: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryImportReportDto {
    pub source: String,
    pub branch: String,
    pub conflict_strategy: String,
    pub preview_only: bool,
    pub imported_rules: usize,
    pub imported_skills: usize,
    pub detected_presets: usize,
    pub skipped: usize,
    pub overwritten: usize,
    pub renamed: usize,
    pub assets: Vec<RepositoryImportAssetDto>,
    pub warnings: Vec<String>,
}
