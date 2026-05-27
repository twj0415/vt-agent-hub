use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use crate::application::app_container::AppContainer;
use crate::application::operation_service::OperationService;
use crate::application::service_context::ServiceContext;
use crate::application::write_service::{parse_markdown_rule, WriteService};
use crate::core::taxonomy;
use crate::dto::{RepositoryImportAssetDto, RepositoryImportReportDto};
use crate::infrastructure::database::Database;
use crate::infrastructure::resource_repo::ResourceRepo;

const TARGET_STATE_READY: i32 = 502;
const SKILL_STATE_NOT_INSTALLED: i32 = 601;

pub struct RepositoryImportService {
    db: Arc<Mutex<Database>>,
    context: ServiceContext,
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
