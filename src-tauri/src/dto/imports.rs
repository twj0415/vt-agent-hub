use serde::{Deserialize, Serialize};

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

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GitHubRepoRefDto {
    pub owner: String,
    pub repo: String,
    pub branch: String,
    pub normalized_url: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GitHubSkillConflictDto {
    pub existing_skill_id: i32,
    pub existing_name: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GitHubSkillPreviewDto {
    pub source_path: String,
    pub skill_id: String,
    pub skill_name: String,
    pub description: Option<String>,
    pub root_directory: String,
    pub skill_directory_name: String,
    pub conflict: Option<GitHubSkillConflictDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubRepoPreviewDto {
    pub repo: GitHubRepoRefDto,
    pub skills: Vec<GitHubSkillPreviewDto>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GitHubSkillImportSelectionDto {
    pub source_path: String,
    pub resolution: String,
    pub renamed_skill_id: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportedGitHubSkillDto {
    pub source_path: String,
    pub skill_id: String,
    pub skill_name: String,
    pub asset_id: i32,
    pub operation: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubRepoImportResultDto {
    pub repo: GitHubRepoRefDto,
    pub imported_skills: Vec<ImportedGitHubSkillDto>,
    pub skipped_skills: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalSkillsPreviewDto {
    pub root_path: String,
    pub skills: Vec<GitHubSkillPreviewDto>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalSkillsImportResultDto {
    pub root_path: String,
    pub imported_skills: Vec<ImportedGitHubSkillDto>,
    pub skipped_skills: Vec<String>,
}
