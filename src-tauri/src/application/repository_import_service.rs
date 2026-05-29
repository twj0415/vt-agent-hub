use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Cursor;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use serde::Deserialize;
use tar::Archive;

use crate::application::app_container::AppContainer;
use crate::application::operation_service::OperationService;
use crate::application::service_context::ServiceContext;
use crate::application::write_service::{parse_markdown_rule, WriteService};
use crate::core::{taxonomy, validation};
use crate::dto::{
    GitHubRepoImportResultDto, GitHubRepoPreviewDto, GitHubRepoRefDto, GitHubSkillConflictDto,
    GitHubSkillImportSelectionDto, GitHubSkillPreviewDto, ImportedGitHubSkillDto,
    LocalSkillsImportResultDto, LocalSkillsPreviewDto, RepositoryImportAssetDto,
    RepositoryImportReportDto,
};
use crate::infrastructure::database::Database;
use crate::infrastructure::resource_repo::ResourceRepo;
use crate::infrastructure::skill_asset_repo::SkillAssetRepo;

const TARGET_STATE_READY: i32 = 502;
const SKILL_STATE_NOT_INSTALLED: i32 = 601;
const USER_AGENT: &str = "vt-agent-hub";

#[derive(Debug, Clone)]
pub(super) struct GitHubRepoRef {
    pub(super) owner: String,
    pub(super) repo: String,
    pub(super) branch: String,
    pub(super) normalized_url: String,
}

#[derive(Debug, Clone)]
pub(super) struct SnapshotSkillManifest {
    pub(super) source_path: String,
    root_directory: String,
    skill_directory_name: String,
    skill_md_path: String,
}

#[derive(Debug, Clone)]
pub(super) struct GitHubSkillCandidate {
    pub(super) manifest: SnapshotSkillManifest,
    pub(super) skill_id: String,
    skill_name: String,
    pub(super) description: Option<String>,
    body: String,
    source_dir: PathBuf,
}

#[derive(Debug, Deserialize)]
struct GitHubRepoApiResponse {
    default_branch: String,
}

#[derive(Debug, Deserialize)]
struct SkillFrontmatter {
    name: Option<String>,
    description: Option<String>,
}

pub struct RepositoryImportService {
    db: Arc<Mutex<Database>>,
    context: ServiceContext,
}

impl GitHubRepoRef {
    fn to_dto(&self) -> GitHubRepoRefDto {
        GitHubRepoRefDto {
            owner: self.owner.clone(),
            repo: self.repo.clone(),
            branch: self.branch.clone(),
            normalized_url: self.normalized_url.clone(),
        }
    }
}

impl RepositoryImportService {
    pub fn new() -> Result<Self, String> {
        Self::with_context(ServiceContext::default()?)
    }

    pub fn with_context(context: ServiceContext) -> Result<Self, String> {
        let db = context.open_db()?;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            context,
        })
    }

    pub fn with_container(container: &AppContainer) -> Self {
        Self {
            db: container.db(),
            context: container.context().clone(),
        }
    }

    pub fn preview_repository(
        &self,
        source: &str,
        branch: &str,
        conflict_strategy: &str,
    ) -> Result<RepositoryImportReportDto, String> {
        self.import_repository(source, branch, conflict_strategy, true)
    }

    pub fn apply_repository(
        &self,
        source: &str,
        branch: &str,
        conflict_strategy: &str,
    ) -> Result<RepositoryImportReportDto, String> {
        self.import_repository(source, branch, conflict_strategy, false)
    }

    pub fn preview_github_repo_import(
        &self,
        repo_url: &str,
    ) -> Result<GitHubRepoPreviewDto, String> {
        let repo = Self::resolve_github_repo_ref(repo_url)?;
        let snapshot_dir = Self::download_repo_snapshot(&repo)?;
        let result = self.preview_github_snapshot(&repo, &snapshot_dir);
        let _ = fs::remove_dir_all(&snapshot_dir);
        result
    }

    pub fn import_github_repo_skills(
        &self,
        repo_url: &str,
        selections: Vec<GitHubSkillImportSelectionDto>,
    ) -> Result<GitHubRepoImportResultDto, String> {
        let repo = Self::resolve_github_repo_ref(repo_url)?;
        let snapshot_dir = Self::download_repo_snapshot(&repo)?;
        let result = self.import_github_snapshot(&repo, &snapshot_dir, selections);
        let _ = fs::remove_dir_all(&snapshot_dir);
        result
    }

    pub fn preview_local_skills(&self, path: &str) -> Result<LocalSkillsPreviewDto, String> {
        let root = self.validate_local_skills_root(path)?;
        let fallback = Self::local_fallback_name(&root);
        let candidates = Self::build_skill_candidates_from_snapshot(&root, &fallback)?;
        let mut skills = Vec::with_capacity(candidates.len());

        for candidate in candidates {
            let existing = {
                let db = self.db.lock().expect("db poisoned");
                ResourceRepo::new(&db).find_latest_skill_version_by_name(&candidate.skill_id)?
            };
            skills.push(Self::candidate_preview(
                candidate,
                existing.map(|skill| GitHubSkillConflictDto {
                    existing_skill_id: skill.asset_id,
                    existing_name: skill.name,
                }),
            ));
        }

        Ok(LocalSkillsPreviewDto {
            root_path: root.display().to_string(),
            skills,
        })
    }

    pub fn import_local_skills(
        &self,
        path: &str,
        selections: Vec<GitHubSkillImportSelectionDto>,
    ) -> Result<LocalSkillsImportResultDto, String> {
        let root = self.validate_local_skills_root(path)?;
        let fallback = Self::local_fallback_name(&root);
        let (imported_skills, skipped_skills) = self.apply_skill_snapshot(
            &root,
            &fallback,
            selections,
            "Local skill import",
            "local-skill-import",
            "local",
        )?;

        Ok(LocalSkillsImportResultDto {
            root_path: root.display().to_string(),
            imported_skills,
            skipped_skills,
        })
    }

    fn validate_local_skills_root(&self, path: &str) -> Result<PathBuf, String> {
        let trimmed = path.trim();
        if trimmed.is_empty() {
            return Err("Local skills directory path is required.".to_string());
        }
        let raw = PathBuf::from(trimmed);
        let canonical = raw
            .canonicalize()
            .map_err(|error| format!("Local skills directory not accessible: {error}"))?;
        if !canonical.is_dir() {
            return Err("Local skills source must be a directory.".to_string());
        }
        if let Ok(library_root) = self.context.library_root() {
            if let Ok(canonical_library) = library_root.canonicalize() {
                if canonical.starts_with(&canonical_library) {
                    return Err(
                        "Local skills source must be outside the app library directory."
                            .to_string(),
                    );
                }
            }
        }
        Ok(canonical)
    }

    fn local_fallback_name(root: &Path) -> String {
        root.file_name()
            .and_then(|value| value.to_str())
            .map(str::to_string)
            .unwrap_or_else(|| "local-skill".to_string())
    }

    fn preview_github_snapshot(
        &self,
        repo: &GitHubRepoRef,
        snapshot_dir: &Path,
    ) -> Result<GitHubRepoPreviewDto, String> {
        let candidates = Self::build_github_skill_candidates_from_snapshot(repo, snapshot_dir)?;
        let mut skills = Vec::with_capacity(candidates.len());

        for candidate in candidates {
            let existing = {
                let db = self.db.lock().expect("db poisoned");
                ResourceRepo::new(&db).find_latest_skill_version_by_name(&candidate.skill_id)?
            };
            skills.push(Self::candidate_preview(
                candidate,
                existing.map(|skill| GitHubSkillConflictDto {
                    existing_skill_id: skill.asset_id,
                    existing_name: skill.name,
                }),
            ));
        }

        Ok(GitHubRepoPreviewDto {
            repo: repo.to_dto(),
            skills,
        })
    }

    pub(super) fn import_github_snapshot(
        &self,
        repo: &GitHubRepoRef,
        snapshot_dir: &Path,
        selections: Vec<GitHubSkillImportSelectionDto>,
    ) -> Result<GitHubRepoImportResultDto, String> {
        let fallback = repo
            .repo
            .strip_suffix("-skill")
            .unwrap_or(&repo.repo)
            .to_string();
        let (imported_skills, skipped_skills) = self.apply_skill_snapshot(
            snapshot_dir,
            &fallback,
            selections,
            "GitHub skill import",
            "github-skill-import",
            "GitHub",
        )?;

        Ok(GitHubRepoImportResultDto {
            repo: repo.to_dto(),
            imported_skills,
            skipped_skills,
        })
    }

    fn apply_skill_snapshot(
        &self,
        snapshot_dir: &Path,
        root_fallback_name: &str,
        selections: Vec<GitHubSkillImportSelectionDto>,
        record_title: &str,
        record_action: &str,
        record_noun: &str,
    ) -> Result<(Vec<ImportedGitHubSkillDto>, Vec<String>), String> {
        let candidates =
            Self::build_skill_candidates_from_snapshot(snapshot_dir, root_fallback_name)?;
        let candidates = candidates
            .into_iter()
            .map(|candidate| (candidate.manifest.source_path.clone(), candidate))
            .collect::<HashMap<_, _>>();
        let mut seen_sources = HashSet::new();
        let mut seen_targets = HashSet::new();
        let mut imported_skills = Vec::new();
        let mut skipped_skills = Vec::new();

        for selection in selections {
            Self::validate_github_selection(&selection)?;
            if !seen_sources.insert(selection.source_path.clone()) {
                return Err(format!(
                    "Duplicate selection for '{}'.",
                    selection.source_path
                ));
            }

            let candidate = candidates.get(&selection.source_path).ok_or_else(|| {
                format!(
                    "Selected skill '{}' was not found in skill snapshot.",
                    selection.source_path
                )
            })?;

            if selection.resolution == "skip" {
                skipped_skills.push(selection.source_path);
                continue;
            }

            let target_name = Self::selection_target_name(candidate, &selection)?;
            if !seen_targets.insert(target_name.to_lowercase()) {
                return Err(format!(
                    "Multiple selected skills resolve to '{}'.",
                    target_name
                ));
            }

            let existing = {
                let db = self.db.lock().expect("db poisoned");
                ResourceRepo::new(&db).find_latest_skill_version_by_name(&target_name)?
            };
            if selection.resolution == "rename" && existing.is_some() {
                return Err(format!("Renamed skill '{}' already exists.", target_name));
            }
            if selection.resolution == "overwrite"
                && existing.is_none()
                && target_name != candidate.skill_id
            {
                return Err(format!("Cannot overwrite missing skill '{}'.", target_name));
            }

            let operation = if existing.is_some() {
                "overwritten"
            } else {
                "imported"
            };
            let asset_id = self.save_github_skill(
                candidate,
                &target_name,
                existing.map(|skill| skill.asset_id),
            )?;
            imported_skills.push(ImportedGitHubSkillDto {
                source_path: candidate.manifest.source_path.clone(),
                skill_id: target_name.clone(),
                skill_name: target_name,
                asset_id,
                operation: operation.to_string(),
            });
        }

        let db = self.db.lock().expect("db poisoned");
        OperationService::record(
            &db,
            None,
            "operation",
            record_title,
            record_action,
            &format!(
                "Imported {} {} skill(s).",
                imported_skills.len(),
                record_noun
            ),
        )?;

        Ok((imported_skills, skipped_skills))
    }

    fn save_github_skill(
        &self,
        candidate: &GitHubSkillCandidate,
        target_name: &str,
        target_id: Option<i32>,
    ) -> Result<i32, String> {
        validation::validate_skill(
            401,
            target_name,
            401,
            TARGET_STATE_READY,
            SKILL_STATE_NOT_INSTALLED,
            &candidate.body,
        )?;
        let summary = candidate.description.clone().unwrap_or_default();
        let saved = {
            let db = self.db.lock().expect("db poisoned");
            ResourceRepo::new(&db).save_skill_version(
                target_id,
                &Self::asset_key(target_name),
                401,
                target_name,
                401,
                TARGET_STATE_READY,
                &summary,
                &candidate.body,
            )?
        };

        SkillAssetRepo::new(self.context.clone()).write_skill_package(
            target_name,
            &candidate.source_dir,
            &candidate.body,
        )?;
        Ok(saved.asset_id)
    }

    fn import_repository(
        &self,
        source: &str,
        branch: &str,
        conflict_strategy: &str,
        preview_only: bool,
    ) -> Result<RepositoryImportReportDto, String> {
        Self::validate_conflict_strategy(conflict_strategy)?;
        let branch = if branch.trim().is_empty() {
            "HEAD"
        } else {
            branch.trim()
        };
        let (checkout_dir, temp_root) = self.resolve_checkout(source, branch)?;
        let result = self.scan_and_import(
            &checkout_dir,
            source,
            branch,
            conflict_strategy,
            preview_only,
        );

        if let Some(temp_root) = temp_root {
            let _ = fs::remove_dir_all(temp_root);
        }

        let report = result?;
        if !preview_only {
            let db = self.db.lock().expect("db poisoned");
            OperationService::record(
                &db,
                None,
                "operation",
                "Repository import",
                "repository-import",
                &format!(
                    "Imported repository assets: {} rule(s), {} skill(s), {} preset(s) detected.",
                    report.imported_rules, report.imported_skills, report.detected_presets
                ),
            )?;
        }
        Ok(report)
    }

    fn resolve_checkout(
        &self,
        source: &str,
        branch: &str,
    ) -> Result<(PathBuf, Option<PathBuf>), String> {
        let trimmed = source.trim();
        if trimmed.is_empty() {
            return Err("Repository source is required.".to_string());
        }

        let local_path = PathBuf::from(trimmed);
        if local_path.exists() {
            if !local_path.is_dir() {
                return Err("Repository source path must be a directory.".to_string());
            }
            return Ok((local_path, None));
        }

        let repository_url = Self::validate_github_url(trimmed)?;
        let temp_root = std::env::temp_dir().join(format!(
            "vt-agent-hub-rebuild-github-import-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or(0)
        ));
        let checkout_dir = temp_root.join("repo");
        fs::create_dir_all(&temp_root).map_err(|error| error.to_string())?;

        let output = Command::new("git")
            .args(["clone", "--depth", "1", "--branch", branch, &repository_url])
            .arg(&checkout_dir)
            .output()
            .map_err(|error| format!("Failed to start git clone: {error}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let _ = fs::remove_dir_all(&temp_root);
            return Err(format!("git clone failed: {stderr}"));
        }

        Ok((checkout_dir, Some(temp_root)))
    }

    fn scan_and_import(
        &self,
        checkout_dir: &Path,
        source: &str,
        branch: &str,
        conflict_strategy: &str,
        preview_only: bool,
    ) -> Result<RepositoryImportReportDto, String> {
        // 与 RepositoryImportService 共享 self.db Arc,保证 WriteService::save_rule
        // 的业务写 + 审计写与本服务后续的审计写在同一连接上。
        let write_service = WriteService::with_db_arc(self.db.clone(), self.context.clone());
        let mut report = RepositoryImportReportDto {
            source: source.to_string(),
            branch: branch.to_string(),
            conflict_strategy: conflict_strategy.to_string(),
            preview_only,
            imported_rules: 0,
            imported_skills: 0,
            detected_presets: 0,
            skipped: 0,
            overwritten: 0,
            renamed: 0,
            assets: Vec::new(),
            warnings: Vec::new(),
        };

        self.scan_rules(
            checkout_dir,
            &write_service,
            &mut report,
            conflict_strategy,
            preview_only,
        )?;
        self.scan_skills(
            checkout_dir,
            &write_service,
            &mut report,
            conflict_strategy,
            preview_only,
        )?;
        self.scan_presets(checkout_dir, &mut report)?;

        Ok(report)
    }

    fn scan_rules(
        &self,
        checkout_dir: &Path,
        write_service: &WriteService,
        report: &mut RepositoryImportReportDto,
        conflict_strategy: &str,
        preview_only: bool,
    ) -> Result<(), String> {
        let rules_root = checkout_dir.join("rules");
        if !rules_root.exists() {
            report
                .warnings
                .push("Repository has no rules directory.".to_string());
            return Ok(());
        }

        for file in Self::collect_files(&rules_root, "md")? {
            let file_name = file
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("rule");
            let source_body = fs::read_to_string(&file).map_err(|error| error.to_string())?;
            let parts = parse_markdown_rule(&source_body);
            let name = if parts.name.is_empty() {
                file_name.to_string()
            } else {
                parts.name.clone()
            };
            let category_code = parts
                .category_code
                .or_else(|| {
                    file.parent()
                        .and_then(|parent| parent.file_name())
                        .and_then(|value| value.to_str())
                        .map(Self::rule_category_code)
                })
                .unwrap_or(304);
            // 短临时持锁:仅做存在性查询,立即 drop guard,避免阻塞下面的 write_service.save_rule
            let existing = {
                let db = self.db.lock().expect("db poisoned");
                ResourceRepo::new(&db).find_latest_rule_version_by_name(&name)?
            };
            let (status, conflict) =
                Self::status_for_existing(existing.is_some(), conflict_strategy);
            report.assets.push(RepositoryImportAssetDto {
                asset_type: "rule".to_string(),
                name: name.clone(),
                source_path: file.display().to_string(),
                status: status.to_string(),
                conflict,
            });
            Self::count_status(report, status, "rule");

            if preview_only || status == "skipped" {
                continue;
            }

            let summary = parts.description;
            let body = parts.body;
            let target_name = if status == "renamed" {
                let db = self.db.lock().expect("db poisoned");
                Self::next_rule_name(&ResourceRepo::new(&db), &name)?
            } else {
                name
            };
            let target_id = if status == "overwritten" {
                existing.map(|rule| rule.asset_id)
            } else {
                None
            };
            write_service.save_rule(
                target_id,
                category_code,
                &target_name,
                category_code,
                TARGET_STATE_READY,
                &summary,
                &body,
            )?;
        }

        Ok(())
    }

    fn scan_skills(
        &self,
        checkout_dir: &Path,
        write_service: &WriteService,
        report: &mut RepositoryImportReportDto,
        conflict_strategy: &str,
        preview_only: bool,
    ) -> Result<(), String> {
        let skills_root = checkout_dir.join("skills");
        if !skills_root.exists() {
            report
                .warnings
                .push("Repository has no skills directory.".to_string());
            return Ok(());
        }

        for entry in fs::read_dir(&skills_root).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            if !entry.path().is_dir() {
                continue;
            }
            let skill_md = entry.path().join("SKILL.md");
            if !skill_md.is_file() {
                report.warnings.push(format!(
                    "Skipped skill without SKILL.md: {}",
                    entry.path().display()
                ));
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_string();
            let existing = {
                let db = self.db.lock().expect("db poisoned");
                ResourceRepo::new(&db).find_latest_skill_version_by_name(&name)?
            };
            let (status, conflict) =
                Self::status_for_existing(existing.is_some(), conflict_strategy);
            report.assets.push(RepositoryImportAssetDto {
                asset_type: "skill".to_string(),
                name: name.clone(),
                source_path: entry.path().display().to_string(),
                status: status.to_string(),
                conflict,
            });
            Self::count_status(report, status, "skill");

            if preview_only || status == "skipped" {
                continue;
            }

            let body = fs::read_to_string(skill_md).map_err(|error| error.to_string())?;
            let summary = format!("Imported from repository {}", entry.path().display());
            let target_name = if status == "renamed" {
                let db = self.db.lock().expect("db poisoned");
                Self::next_skill_name(&ResourceRepo::new(&db), &name)?
            } else {
                name
            };
            let target_id = if status == "overwritten" {
                existing.map(|skill| skill.asset_id)
            } else {
                None
            };
            write_service.save_skill(
                target_id,
                401,
                &target_name,
                401,
                TARGET_STATE_READY,
                SKILL_STATE_NOT_INSTALLED,
                &summary,
                &body,
            )?;
        }

        Ok(())
    }

    fn scan_presets(
        &self,
        checkout_dir: &Path,
        report: &mut RepositoryImportReportDto,
    ) -> Result<(), String> {
        let presets_root = checkout_dir.join("presets");
        if !presets_root.exists() {
            return Ok(());
        }

        let files = Self::collect_files(&presets_root, "json")?;
        report.detected_presets = files.len();
        for file in files {
            report.assets.push(RepositoryImportAssetDto {
                asset_type: "preset".to_string(),
                name: file
                    .file_stem()
                    .and_then(|value| value.to_str())
                    .unwrap_or("preset")
                    .to_string(),
                source_path: file.display().to_string(),
                status: "detected".to_string(),
                conflict: false,
            });
        }
        Ok(())
    }

    fn collect_files(root: &Path, extension: &str) -> Result<Vec<PathBuf>, String> {
        let mut result = Vec::new();
        Self::collect_files_inner(root, extension, &mut result)?;
        result.sort();
        Ok(result)
    }

    fn collect_files_inner(
        root: &Path,
        extension: &str,
        result: &mut Vec<PathBuf>,
    ) -> Result<(), String> {
        for entry in fs::read_dir(root).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                Self::collect_files_inner(&path, extension, result)?;
            } else if path
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|value| value.eq_ignore_ascii_case(extension))
            {
                result.push(path);
            }
        }
        Ok(())
    }

    fn status_for_existing(existing: bool, conflict_strategy: &str) -> (&'static str, bool) {
        if !existing {
            return ("imported", false);
        }
        match conflict_strategy {
            "skip" => ("skipped", true),
            "overwrite" => ("overwritten", true),
            "rename" => ("renamed", true),
            _ => ("skipped", true),
        }
    }

    fn count_status(report: &mut RepositoryImportReportDto, status: &str, asset_type: &str) {
        match status {
            "skipped" => report.skipped += 1,
            "overwritten" => report.overwritten += 1,
            "renamed" => report.renamed += 1,
            "imported" if asset_type == "rule" => report.imported_rules += 1,
            "imported" if asset_type == "skill" => report.imported_skills += 1,
            _ => {}
        }
    }

    fn next_rule_name(repo: &ResourceRepo<'_>, base_name: &str) -> Result<String, String> {
        for index in 2..1000 {
            let candidate = format!("{base_name} ({index})");
            if repo.find_latest_rule_version_by_name(&candidate)?.is_none() {
                return Ok(candidate);
            }
        }
        Err(format!(
            "No available import name for rule '{}'.",
            base_name
        ))
    }

    fn next_skill_name(repo: &ResourceRepo<'_>, base_name: &str) -> Result<String, String> {
        for index in 2..1000 {
            let candidate = format!("{base_name} ({index})");
            if repo
                .find_latest_skill_version_by_name(&candidate)?
                .is_none()
            {
                return Ok(candidate);
            }
        }
        Err(format!(
            "No available import name for skill '{}'.",
            base_name
        ))
    }
    fn validate_conflict_strategy(value: &str) -> Result<(), String> {
        if ["skip", "rename", "overwrite"].contains(&value) {
            Ok(())
        } else {
            Err(format!("Unsupported import conflict strategy: {value}"))
        }
    }

    pub(super) fn parse_github_url(value: &str) -> Result<(String, String), String> {
        let trimmed = value.trim().trim_end_matches('/');
        if !trimmed.starts_with("https://github.com/") {
            return Err("Enter a valid GitHub repository URL.".to_string());
        }
        let remainder = trimmed.trim_start_matches("https://github.com/");
        let parts = remainder.split('/').collect::<Vec<_>>();
        if parts.len() < 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err("Enter a valid GitHub repository URL.".to_string());
        }
        let owner = parts[0].to_string();
        let repo = parts[1].trim_end_matches(".git").to_string();
        if !Self::valid_github_segment(&owner) || !Self::valid_github_segment(&repo) {
            return Err("GitHub repository URL contains invalid path segments.".to_string());
        }
        Ok((owner, repo))
    }

    fn resolve_github_repo_ref(repo_url: &str) -> Result<GitHubRepoRef, String> {
        let (owner, repo) = Self::parse_github_url(repo_url)?;
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .map_err(|error| error.to_string())?;
        let api_url = format!("https://api.github.com/repos/{owner}/{repo}");
        let branch = match client.get(api_url).send() {
            Ok(response) if response.status().is_success() => {
                response
                    .json::<GitHubRepoApiResponse>()
                    .map_err(|error| {
                        format!("Failed to parse GitHub repository metadata: {error}")
                    })?
                    .default_branch
            }
            Ok(response) if response.status().as_u16() == 403 => "HEAD".to_string(),
            Ok(response) => {
                return Err(format!(
                    "GitHub repository metadata request failed: {}",
                    response.status()
                ))
            }
            Err(error) => {
                return Err(format!(
                    "Failed to fetch GitHub repository metadata: {error}"
                ))
            }
        };
        Ok(GitHubRepoRef {
            owner: owner.clone(),
            repo: repo.clone(),
            branch,
            normalized_url: format!("https://github.com/{owner}/{repo}"),
        })
    }

    fn download_repo_snapshot(repo: &GitHubRepoRef) -> Result<PathBuf, String> {
        let temp_root = std::env::temp_dir().join(format!(
            "vt-agent-hub-github-skill-import-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or(0)
        ));
        fs::create_dir_all(&temp_root).map_err(|error| error.to_string())?;
        let result = Self::download_repository_archive(repo)
            .and_then(|bytes| Self::snapshot_from_repository_archive(&temp_root, bytes));
        if result.is_err() {
            let _ = fs::remove_dir_all(&temp_root);
        }
        result
    }

    fn download_repository_archive(repo: &GitHubRepoRef) -> Result<Vec<u8>, String> {
        let client = Client::builder()
            .user_agent(USER_AGENT)
            .build()
            .map_err(|error| error.to_string())?;
        let archive_url = format!(
            "https://codeload.github.com/{}/{}/tar.gz/{}",
            repo.owner, repo.repo, repo.branch
        );
        let response = client
            .get(archive_url)
            .send()
            .map_err(|error| format!("Failed to download GitHub repository archive: {error}"))?;
        if !response.status().is_success() {
            return Err(format!(
                "GitHub repository archive request failed: {}",
                response.status()
            ));
        }
        response
            .bytes()
            .map(|bytes| bytes.to_vec())
            .map_err(|error| format!("Failed to read GitHub repository archive: {error}"))
    }

    fn snapshot_from_repository_archive(
        temp_root: &Path,
        bytes: Vec<u8>,
    ) -> Result<PathBuf, String> {
        let snapshot_dir = temp_root.join("snapshot");
        fs::create_dir_all(&snapshot_dir).map_err(|error| error.to_string())?;
        let decoder = GzDecoder::new(Cursor::new(bytes));
        let mut archive = Archive::new(decoder);

        for entry in archive.entries().map_err(|error| error.to_string())? {
            let mut entry = entry.map_err(|error| error.to_string())?;
            let relative =
                Self::relative_archive_path(&entry.path().map_err(|error| error.to_string())?)?;
            if relative.as_os_str().is_empty() {
                continue;
            }
            if !Self::is_safe_repo_relative_path(&relative) {
                return Err(format!(
                    "Unsafe path in GitHub archive: {}",
                    relative.display()
                ));
            }
            let target = snapshot_dir.join(relative);
            if entry.header().entry_type().is_dir() {
                fs::create_dir_all(&target).map_err(|error| error.to_string())?;
            } else if entry.header().entry_type().is_file() {
                if let Some(parent) = target.parent() {
                    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
                }
                entry.unpack(&target).map_err(|error| error.to_string())?;
            }
        }

        Ok(snapshot_dir)
    }

    fn relative_archive_path(path: &Path) -> Result<PathBuf, String> {
        let mut components = path.components();
        let Some(first) = components.next() else {
            return Ok(PathBuf::new());
        };
        if !matches!(first, Component::Normal(_)) {
            return Err("GitHub archive contains an invalid root path.".to_string());
        }
        Ok(components.collect())
    }

    fn is_safe_repo_relative_path(path: &Path) -> bool {
        path.components()
            .all(|component| matches!(component, Component::Normal(_)))
    }

    pub(super) fn build_github_skill_candidates_from_snapshot(
        repo: &GitHubRepoRef,
        snapshot_dir: &Path,
    ) -> Result<Vec<GitHubSkillCandidate>, String> {
        let fallback = repo
            .repo
            .strip_suffix("-skill")
            .unwrap_or(&repo.repo)
            .to_string();
        Self::build_skill_candidates_from_snapshot(snapshot_dir, &fallback)
    }

    pub(super) fn build_skill_candidates_from_snapshot(
        snapshot_dir: &Path,
        root_fallback_name: &str,
    ) -> Result<Vec<GitHubSkillCandidate>, String> {
        let manifests = Self::collect_skill_manifests(snapshot_dir)?;
        let mut candidates = Vec::new();
        for manifest in manifests {
            let skill_md = snapshot_dir.join(&manifest.skill_md_path);
            let body = fs::read_to_string(&skill_md).map_err(|error| error.to_string())?;
            let frontmatter = Self::parse_skill_frontmatter(&body);
            let fallback = if manifest.source_path == "." {
                root_fallback_name.to_string()
            } else {
                manifest.skill_directory_name.clone()
            };
            let skill_id = Self::sanitize_skill_id(
                frontmatter
                    .as_ref()
                    .and_then(|item| item.name.as_deref())
                    .unwrap_or(&fallback),
            )?;
            let skill_name = skill_id.clone();
            let source_dir = if manifest.source_path == "." {
                snapshot_dir.to_path_buf()
            } else {
                snapshot_dir.join(&manifest.source_path)
            };
            candidates.push(GitHubSkillCandidate {
                manifest,
                skill_id,
                skill_name,
                description: frontmatter.and_then(|item| item.description),
                body,
                source_dir,
            });
        }
        candidates
            .sort_by(|left, right| left.manifest.source_path.cmp(&right.manifest.source_path));
        Ok(candidates)
    }

    fn collect_skill_manifests(snapshot_dir: &Path) -> Result<Vec<SnapshotSkillManifest>, String> {
        let mut manifest_paths = Vec::new();
        Self::collect_skill_manifest_paths(snapshot_dir, snapshot_dir, &mut manifest_paths)?;
        Ok(manifest_paths
            .into_iter()
            .filter_map(|path| Self::classify_skill_manifest_path(&path))
            .collect::<Vec<_>>())
    }

    fn collect_skill_manifest_paths(
        snapshot_dir: &Path,
        current: &Path,
        result: &mut Vec<String>,
    ) -> Result<(), String> {
        for entry in fs::read_dir(current).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let path = entry.path();
            let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
            if metadata.file_type().is_symlink() {
                continue;
            }
            if metadata.is_dir() {
                Self::collect_skill_manifest_paths(snapshot_dir, &path, result)?;
            } else if metadata.is_file()
                && path
                    .file_name()
                    .and_then(|value| value.to_str())
                    .is_some_and(|name| name.eq_ignore_ascii_case("SKILL.md"))
            {
                let relative = path
                    .strip_prefix(snapshot_dir)
                    .map_err(|error| error.to_string())?
                    .to_string_lossy()
                    .replace('\\', "/");
                result.push(relative);
            }
        }
        Ok(())
    }

    fn classify_skill_manifest_path(path: &str) -> Option<SnapshotSkillManifest> {
        let normalized = path.trim_matches('/');
        if normalized.is_empty() {
            return None;
        }
        if normalized.eq_ignore_ascii_case("SKILL.md") {
            return Some(SnapshotSkillManifest {
                source_path: ".".to_string(),
                root_directory: "/".to_string(),
                skill_directory_name: String::new(),
                skill_md_path: "SKILL.md".to_string(),
            });
        }

        let parts = normalized.split('/').collect::<Vec<_>>();
        let (skill_md, source_parts) = parts.split_last()?;
        if !skill_md.eq_ignore_ascii_case("SKILL.md") {
            return None;
        }

        match source_parts {
            [skill_dir] if *skill_dir != ".github" && *skill_dir != "skills" => {
                Some(SnapshotSkillManifest {
                    source_path: (*skill_dir).to_string(),
                    root_directory: "/".to_string(),
                    skill_directory_name: (*skill_dir).to_string(),
                    skill_md_path: normalized.to_string(),
                })
            }
            _ if source_parts.first() == Some(&"skills") && source_parts.len() >= 2 => {
                Some(SnapshotSkillManifest {
                    source_path: source_parts.join("/"),
                    root_directory: source_parts[..source_parts.len() - 1].join("/"),
                    skill_directory_name: source_parts.last()?.to_string(),
                    skill_md_path: normalized.to_string(),
                })
            }
            _ => None,
        }
    }

    fn parse_skill_frontmatter(content: &str) -> Option<SkillFrontmatter> {
        let trimmed = content.trim_start();
        if !trimmed.starts_with("---") {
            return None;
        }
        let rest = &trimmed[3..];
        let end = rest.find("---")?;
        serde_yaml::from_str::<SkillFrontmatter>(&rest[..end]).ok()
    }

    fn sanitize_skill_id(raw: &str) -> Result<String, String> {
        let lowered = raw.trim().to_lowercase();
        let mut sanitized = String::new();
        let mut last_was_dash = false;
        for ch in lowered.chars() {
            if ch.is_ascii_alphanumeric() {
                sanitized.push(ch);
                last_was_dash = false;
            } else if !last_was_dash {
                sanitized.push('-');
                last_was_dash = true;
            }
        }
        let sanitized = sanitized.trim_matches('-').to_string();
        if sanitized.is_empty() {
            return Err(format!("Skill identifier '{}' is not supported.", raw));
        }
        Ok(sanitized)
    }

    fn validate_github_selection(selection: &GitHubSkillImportSelectionDto) -> Result<(), String> {
        if !["skip", "overwrite", "rename"].contains(&selection.resolution.as_str()) {
            return Err(format!(
                "Unsupported GitHub skill resolution: {}",
                selection.resolution
            ));
        }
        if selection.source_path.trim().is_empty() {
            return Err("GitHub skill sourcePath is required.".to_string());
        }
        Ok(())
    }

    fn selection_target_name(
        candidate: &GitHubSkillCandidate,
        selection: &GitHubSkillImportSelectionDto,
    ) -> Result<String, String> {
        if selection.resolution == "rename" {
            let renamed = selection.renamed_skill_id.as_deref().ok_or_else(|| {
                "renamedSkillId is required when resolution is rename.".to_string()
            })?;
            Self::sanitize_skill_id(renamed)
        } else {
            Ok(candidate.skill_id.clone())
        }
    }

    fn candidate_preview(
        candidate: GitHubSkillCandidate,
        conflict: Option<GitHubSkillConflictDto>,
    ) -> GitHubSkillPreviewDto {
        GitHubSkillPreviewDto {
            source_path: candidate.manifest.source_path,
            skill_id: candidate.skill_id,
            skill_name: candidate.skill_name,
            description: candidate.description,
            root_directory: candidate.manifest.root_directory,
            skill_directory_name: candidate.manifest.skill_directory_name,
            conflict,
        }
    }

    fn asset_key(name: &str) -> String {
        let lowered = name.trim().to_lowercase();
        let mut key = String::new();
        let mut last_dash = false;

        for ch in lowered.chars() {
            if ch.is_alphanumeric() {
                key.push(ch);
                last_dash = false;
            } else if !last_dash {
                key.push('-');
                last_dash = true;
            }
        }

        let key = key.trim_matches('-').to_string();
        if key.is_empty() {
            let hash = name.bytes().fold(0xcbf29ce484222325u64, |acc, byte| {
                (acc ^ u64::from(byte)).wrapping_mul(0x100000001b3)
            });
            format!("skill-{hash:016x}")
        } else {
            key
        }
    }

    fn validate_github_url(value: &str) -> Result<String, String> {
        let trimmed = value.trim().trim_end_matches('/');
        if !trimmed.starts_with("https://github.com/") {
            return Err(
                "Enter a valid GitHub repository URL or an existing local repository path."
                    .to_string(),
            );
        }
        let remainder = trimmed.trim_start_matches("https://github.com/");
        let parts = remainder.split('/').collect::<Vec<_>>();
        if parts.len() < 2 || parts[0].is_empty() || parts[1].is_empty() {
            return Err("Enter a valid GitHub repository URL.".to_string());
        }
        let owner = parts[0];
        let repo = parts[1].trim_end_matches(".git");
        if !Self::valid_github_segment(owner) || !Self::valid_github_segment(repo) {
            return Err("GitHub repository URL contains invalid path segments.".to_string());
        }
        Ok(format!("https://github.com/{owner}/{repo}.git"))
    }

    fn valid_github_segment(value: &str) -> bool {
        !value.is_empty()
            && value
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
    }

    fn rule_category_code(name: &str) -> i32 {
        taxonomy::parse_rule_category_alias(name).unwrap_or(taxonomy::DEFAULT_RULE_CATEGORY)
    }
}
