use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

use crate::application::app_container::AppContainer;
use crate::application::operation_service::OperationService;
use crate::application::provider_runtime_service::ProviderRuntimeService;
use crate::application::service_context::ServiceContext;
use crate::application::write_service::parse_markdown_rule;
use crate::core::paths;
use crate::core::product::MANAGED_MARKER;
use crate::core::routes::{ROUTE_PRESETS, ROUTE_RULES, ROUTE_SETTINGS, ROUTE_SKILLS};
use crate::core::status_codes::{HEALTH_NORMAL, TARGET_STATE_READY};
use crate::core::taxonomy::{RULE_CATEGORY_PERSONAL, SKILL_CATEGORY_CODING};
use crate::core::tool_registry::{CLAUDE_TOOL_ID, CODEX_TOOL_ID};
use crate::dto::{
    FirstRunImportAppliedAssetDto, FirstRunImportApplyInputDto, FirstRunImportApplyResultDto,
    FirstRunImportCandidateDto, FirstRunImportPreviewDto, FirstRunImportRootDto,
    FirstRunImportStatusDto,
};
use crate::infrastructure::credential_store::CredentialStore;
use crate::infrastructure::database::Database;
use crate::infrastructure::provider_repo::{ProviderConfigUpsert, ProviderRepo};
use crate::infrastructure::resource_repo::ResourceRepo;
use crate::infrastructure::settings_repo::SettingsRepo;
use crate::infrastructure::skill_asset_repo::SkillAssetRepo;

const SCAN_VERSION: &str = "global-import-v1";
const STATUS_KEY: &str = "first_run_global_import_status";
const SEEN_AT_KEY: &str = "first_run_global_import_seen_at";
const COMPLETED_AT_KEY: &str = "first_run_global_import_completed_at";
const SCAN_VERSION_KEY: &str = "first_run_global_import_scan_version";
const SUMMARY_KEY: &str = "first_run_global_import_summary";
const MAX_TEXT_FILE_BYTES: u64 = 1024 * 1024;
const UNSUPPORTED_WARNING: &str = "该资源类型暂未开发，仅展示来源。";
const FIRST_RUN_IMPORTED_SUMMARY_KEY: &str = "firstRunImport.descriptions.initialImport";

#[derive(Debug, Clone)]
struct GlobalScanCandidate {
    id: String,
    asset_type: String,
    target_asset_type: String,
    source_tool_id: i32,
    source_tool: String,
    source_kind: String,
    name: String,
    summary: String,
    source_path: PathBuf,
    relative_path: String,
    status: String,
    conflict: Option<String>,
    existing_id: Option<i32>,
    default_selected: bool,
    selectable: bool,
    recommended_action: String,
    content_preview: String,
    warnings: Vec<String>,
    metadata: Value,
    body: String,
    skill_dir: Option<PathBuf>,
    credential_token: Option<String>,
}

#[derive(Debug, Clone)]
struct ScanOutput {
    roots: Vec<FirstRunImportRootDto>,
    candidates: Vec<GlobalScanCandidate>,
    warnings: Vec<String>,
}

#[derive(Debug, Clone)]
struct ManagedRuleSection {
    name: String,
    body: String,
}

#[derive(Debug, Clone)]
struct RuleMergeBucket {
    candidate: GlobalScanCandidate,
    source_tool_ids: Vec<i32>,
}

pub struct FirstRunImportService {
    db: Arc<Mutex<Database>>,
    context: ServiceContext,
    claude_root: PathBuf,
    codex_root: PathBuf,
}

impl FirstRunImportService {
    pub fn with_container(container: &AppContainer) -> Self {
        Self {
            db: container.db(),
            context: container.context().clone(),
            claude_root: paths::claude_root(),
            codex_root: paths::codex_root(),
        }
    }

    pub fn with_context(context: ServiceContext) -> Result<Self, String> {
        Self::with_context_and_roots(context, paths::claude_root(), paths::codex_root())
    }

    pub fn with_context_and_roots(
        context: ServiceContext,
        claude_root: PathBuf,
        codex_root: PathBuf,
    ) -> Result<Self, String> {
        let db = context.open_db()?;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            context,
            claude_root,
            codex_root,
        })
    }

    pub fn status(&self) -> Result<FirstRunImportStatusDto, String> {
        let db = self.db.lock().expect("db poisoned");
        let status = SettingsRepo::new(&db)
            .get(STATUS_KEY)?
            .unwrap_or_else(|| "pending".to_string());
        Ok(FirstRunImportStatusDto {
            should_prompt: status == "pending",
            status,
        })
    }

    pub fn preview(&self) -> Result<FirstRunImportPreviewDto, String> {
        let output = self.scan()?;
        let status = if output.candidates.is_empty() {
            let db = self.db.lock().expect("db poisoned");
            let repo = SettingsRepo::new(&db);
            repo.upsert(STATUS_KEY, "no_candidates")?;
            repo.upsert(SEEN_AT_KEY, &current_timestamp())?;
            repo.upsert(SCAN_VERSION_KEY, SCAN_VERSION)?;
            "no_candidates".to_string()
        } else {
            self.status()?.status
        };

        Ok(FirstRunImportPreviewDto {
            status,
            scan_version: SCAN_VERSION.to_string(),
            roots: output.roots,
            candidates: output
                .candidates
                .into_iter()
                .map(Self::candidate_dto)
                .collect(),
            warnings: output.warnings,
            credential_policy: "检测到的密钥可导入到本地凭据存储，界面可显示明文。".to_string(),
        })
    }

    pub fn apply(
        &self,
        input: FirstRunImportApplyInputDto,
    ) -> Result<FirstRunImportApplyResultDto, String> {
        if !input.confirm {
            return Err("First-run import requires explicit confirmation.".to_string());
        }
        let conflict_strategy = input
            .conflict_strategy
            .unwrap_or_else(|| "rename".to_string());
        if !["skip", "rename", "overwrite"].contains(&conflict_strategy.as_str()) {
            return Err(format!(
                "Unsupported first-run import conflict strategy: {conflict_strategy}"
            ));
        }

        let selected = input.selected_ids.into_iter().collect::<HashSet<_>>();
        let output = self.scan()?;
        let mut result = FirstRunImportApplyResultDto {
            imported_rules: 0,
            imported_skills: 0,
            imported_providers: 0,
            skipped: 0,
            renamed: 0,
            overwritten: 0,
            assets: Vec::new(),
            warnings: output.warnings,
        };

        for candidate in output.candidates {
            if !selected.contains(&candidate.id) {
                continue;
            }
            if !candidate.selectable {
                result.skipped += 1;
                result
                    .warnings
                    .push(format!("{}: {}", candidate.name, UNSUPPORTED_WARNING));
                continue;
            }

            match candidate.asset_type.as_str() {
                "rule" => self.apply_rule_candidate(&candidate, &conflict_strategy, &mut result)?,
                "skill" => {
                    self.apply_skill_candidate(&candidate, &conflict_strategy, &mut result)?
                }
                "provider_preset" => self.apply_provider_candidate(&candidate, &mut result)?,
                _ => {
                    result.skipped += 1;
                    result.warnings.push(format!(
                        "Skipped unsupported candidate '{}'.",
                        candidate.name
                    ));
                }
            }
        }

        let summary = json!({
            "importedRules": result.imported_rules,
            "importedSkills": result.imported_skills,
            "importedProviders": result.imported_providers,
            "skipped": result.skipped,
            "renamed": result.renamed,
            "overwritten": result.overwritten,
        });
        let db = self.db.lock().expect("db poisoned");
        let repo = SettingsRepo::new(&db);
        repo.upsert(STATUS_KEY, "completed")?;
        repo.upsert(COMPLETED_AT_KEY, &current_timestamp())?;
        repo.upsert(SCAN_VERSION_KEY, SCAN_VERSION)?;
        repo.upsert(SUMMARY_KEY, &summary.to_string())?;
        OperationService::record_simple(
            &db,
            None,
            None,
            None,
            "operation",
            "First-run global import",
            "first-run-global-import",
            &format!(
                "Imported {} rule(s), {} skill(s), and {} provider preset(s).",
                result.imported_rules, result.imported_skills, result.imported_providers
            ),
            "success",
            HEALTH_NORMAL,
            None,
            ROUTE_SETTINGS,
        )?;

        Ok(result)
    }

    pub fn dismiss(
        &self,
        status: &str,
        reason: Option<&str>,
    ) -> Result<FirstRunImportStatusDto, String> {
        let next_status = match status {
            "dismissed" | "completed" | "no_candidates" => status,
            _ => "dismissed",
        };
        let db = self.db.lock().expect("db poisoned");
        let repo = SettingsRepo::new(&db);
        repo.upsert(STATUS_KEY, next_status)?;
        repo.upsert(SEEN_AT_KEY, &current_timestamp())?;
        repo.upsert(SCAN_VERSION_KEY, SCAN_VERSION)?;
        if let Some(reason) = reason.map(str::trim).filter(|value| !value.is_empty()) {
            repo.upsert(SUMMARY_KEY, &json!({ "reason": reason }).to_string())?;
        }
        Ok(FirstRunImportStatusDto {
            status: next_status.to_string(),
            should_prompt: false,
        })
    }

    pub fn reset_status(&self) -> Result<FirstRunImportStatusDto, String> {
        let db = self.db.lock().expect("db poisoned");
        let repo = SettingsRepo::new(&db);
        for key in [
            STATUS_KEY,
            SEEN_AT_KEY,
            COMPLETED_AT_KEY,
            SCAN_VERSION_KEY,
            SUMMARY_KEY,
        ] {
            repo.delete(key)?;
        }
        Ok(FirstRunImportStatusDto {
            status: "pending".to_string(),
            should_prompt: true,
        })
    }

    fn scan(&self) -> Result<ScanOutput, String> {
        let mut warnings = Vec::new();
        let mut claude_candidates = Vec::new();
        let mut codex_candidates = Vec::new();

        self.scan_claude_root(&self.claude_root, &mut claude_candidates, &mut warnings)?;
        self.scan_codex_root(&self.codex_root, &mut codex_candidates, &mut warnings)?;

        let roots = vec![
            FirstRunImportRootDto {
                tool: "claude".to_string(),
                path: self.claude_root.display().to_string(),
                exists: self.claude_root.is_dir(),
                candidate_count: claude_candidates.len(),
            },
            FirstRunImportRootDto {
                tool: "codex".to_string(),
                path: self.codex_root.display().to_string(),
                exists: self.codex_root.is_dir(),
                candidate_count: codex_candidates.len(),
            },
        ];
        let candidates = self.merge_rule_candidates(claude_candidates, codex_candidates);

        Ok(ScanOutput {
            roots,
            candidates,
            warnings,
        })
    }

    fn scan_claude_root(
        &self,
        root: &Path,
        candidates: &mut Vec<GlobalScanCandidate>,
        warnings: &mut Vec<String>,
    ) -> Result<(), String> {
        if !root.is_dir() {
            return Ok(());
        }
        self.push_markdown_rule(
            root,
            &root.join("CLAUDE.md"),
            "Claude Global Rule",
            "claude",
            CLAUDE_TOOL_ID,
            "global_rule",
            candidates,
            warnings,
        )?;
        self.scan_skill_root(
            root,
            &root.join("skills"),
            "claude",
            CLAUDE_TOOL_ID,
            candidates,
            warnings,
        )?;
        self.scan_unsupported_markdown_root(
            root,
            &root.join("commands"),
            "claude",
            CLAUDE_TOOL_ID,
            "claude_command",
            "command",
            candidates,
            warnings,
        )?;
        self.push_claude_provider(root, &root.join("settings.json"), candidates, warnings)?;
        Ok(())
    }

    fn scan_codex_root(
        &self,
        root: &Path,
        candidates: &mut Vec<GlobalScanCandidate>,
        warnings: &mut Vec<String>,
    ) -> Result<(), String> {
        if !root.is_dir() {
            return Ok(());
        }
        self.push_markdown_rule(
            root,
            &root.join("AGENTS.md"),
            "Codex Global Rule",
            "codex",
            CODEX_TOOL_ID,
            "global_rule",
            candidates,
            warnings,
        )?;
        self.scan_skill_root(
            root,
            &root.join("skills"),
            "codex",
            CODEX_TOOL_ID,
            candidates,
            warnings,
        )?;
        self.scan_unsupported_markdown_root(
            root,
            &root.join("prompts"),
            "codex",
            CODEX_TOOL_ID,
            "codex_prompt",
            "prompt",
            candidates,
            warnings,
        )?;
        self.scan_unsupported_markdown_root(
            root,
            &root.join("commands"),
            "codex",
            CODEX_TOOL_ID,
            "codex_command",
            "command",
            candidates,
            warnings,
        )?;
        self.push_codex_provider(root, &root.join("config.toml"), candidates, warnings)?;
        Ok(())
    }

    fn push_markdown_rule(
        &self,
        root: &Path,
        path: &Path,
        fallback_name: &str,
        source_tool: &str,
        source_tool_id: i32,
        source_kind: &str,
        candidates: &mut Vec<GlobalScanCandidate>,
        warnings: &mut Vec<String>,
    ) -> Result<(), String> {
        if !path.is_file() {
            return Ok(());
        }
        let Some((path, relative_path)) = self.safe_candidate_path(root, path, warnings)? else {
            return Ok(());
        };
        let Some(body) = read_candidate_text(&path, warnings)? else {
            return Ok(());
        };
        let parts = parse_markdown_rule(&body);
        let fallback_name = rule_name_from_path(&path, fallback_name);
        let managed_rules = parse_managed_rule_sections(&body);
        if !managed_rules.is_empty() {
            for (index, section) in managed_rules.into_iter().enumerate() {
                self.push_rule_candidate(
                    candidates,
                    source_tool,
                    source_tool_id,
                    source_kind,
                    &format!("{relative_path}#{}", index + 1),
                    &path,
                    &relative_path,
                    section.name,
                    FIRST_RUN_IMPORTED_SUMMARY_KEY.to_string(),
                    section.body,
                    RULE_CATEGORY_PERSONAL,
                    json!({ "categoryCode": RULE_CATEGORY_PERSONAL, "splitFromManagedOutput": true }),
                )?;
            }
            return Ok(());
        }

        let name = non_empty(parts.name, &fallback_name);
        let summary = non_empty(parts.description, FIRST_RUN_IMPORTED_SUMMARY_KEY);
        self.push_rule_candidate(
            candidates,
            source_tool,
            source_tool_id,
            source_kind,
            &relative_path,
            &path,
            &relative_path,
            name,
            summary,
            parts.body,
            parts.category_code.unwrap_or(RULE_CATEGORY_PERSONAL),
            json!({ "categoryCode": parts.category_code.unwrap_or(RULE_CATEGORY_PERSONAL) }),
        )?;
        Ok(())
    }

    fn push_rule_candidate(
        &self,
        candidates: &mut Vec<GlobalScanCandidate>,
        source_tool: &str,
        source_tool_id: i32,
        source_kind: &str,
        id_suffix: &str,
        path: &Path,
        relative_path: &str,
        name: String,
        summary: String,
        body: String,
        category_code: i32,
        metadata: Value,
    ) -> Result<(), String> {
        let existing = self.find_rule(&name)?;
        let conflict = existing.as_ref().map(|_| "name".to_string());
        let recommended_action = if existing.is_some() {
            "rename"
        } else {
            "create"
        }
        .to_string();
        candidates.push(GlobalScanCandidate {
            id: candidate_id(source_tool, source_kind, id_suffix),
            asset_type: "rule".to_string(),
            target_asset_type: "rule".to_string(),
            source_tool_id,
            source_tool: source_tool.to_string(),
            source_kind: source_kind.to_string(),
            name,
            summary,
            source_path: path.to_path_buf(),
            relative_path: relative_path.to_string(),
            status: if existing.is_some() { "conflict" } else { "ready" }.to_string(),
            conflict,
            existing_id: existing.map(|rule| rule.asset_id),
            default_selected: true,
            selectable: true,
            recommended_action,
            content_preview: preview_text(&body),
            warnings: Vec::new(),
            metadata: if metadata.get("categoryCode").is_some() { metadata } else { json!({ "categoryCode": category_code }) },
            body,
            skill_dir: None,
            credential_token: None,
        });
        Ok(())
    }

    fn merge_rule_candidates(
        &self,
        mut claude_candidates: Vec<GlobalScanCandidate>,
        mut codex_candidates: Vec<GlobalScanCandidate>,
    ) -> Vec<GlobalScanCandidate> {
        let mut merged: Vec<RuleMergeBucket> = Vec::new();
        for candidate in claude_candidates.drain(..).chain(codex_candidates.drain(..)) {
            if candidate.asset_type != "rule" {
                merged.push(RuleMergeBucket {
                    source_tool_ids: vec![candidate.source_tool_id],
                    candidate,
                });
                continue;
            }

            if let Some(bucket) = merged.iter_mut().find(|bucket| {
                bucket.candidate.asset_type == "rule"
                    && bucket.candidate.name.eq_ignore_ascii_case(&candidate.name)
                    && bucket.candidate.body == candidate.body
                    && bucket.candidate.metadata == candidate.metadata
            }) {
                if !bucket.source_tool_ids.contains(&candidate.source_tool_id) {
                    bucket.source_tool_ids.push(candidate.source_tool_id);
                }
                continue;
            }

            merged.push(RuleMergeBucket {
                source_tool_ids: vec![candidate.source_tool_id],
                candidate,
            });
        }

        let mut candidates = merged
            .into_iter()
            .map(|mut bucket| {
                if bucket.source_tool_ids.len() > 1 {
                    let mut metadata = bucket.candidate.metadata;
                    metadata["sourceToolIds"] = json!(bucket.source_tool_ids);
                    bucket.candidate.metadata = metadata;
                }
                bucket.candidate
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| left.id.cmp(&right.id));
        candidates
    }

    fn scan_skill_root(
        &self,
        root: &Path,
        skills_root: &Path,
        source_tool: &str,
        source_tool_id: i32,
        candidates: &mut Vec<GlobalScanCandidate>,
        warnings: &mut Vec<String>,
    ) -> Result<(), String> {
        if !skills_root.is_dir() {
            return Ok(());
        }
        for entry in fs::read_dir(skills_root).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let source_dir = entry.path();
            if !source_dir.is_dir() {
                continue;
            }
            let skill_md = source_dir.join("SKILL.md");
            if !skill_md.is_file() {
                continue;
            }
            let Some((skill_md, relative_path)) =
                self.safe_candidate_path(root, &skill_md, warnings)?
            else {
                continue;
            };
            let Some((source_dir, _)) = self.safe_candidate_path(root, &source_dir, warnings)?
            else {
                continue;
            };
            let Some(body) = read_candidate_text(&skill_md, warnings)? else {
                continue;
            };
            let parts = parse_markdown_rule(&body);
            let fallback_name = source_dir
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or("Skill");
            let name = non_empty(parts.name, fallback_name);
            let summary = non_empty(parts.description, "Imported from global tool skill.");
            let existing = self.find_skill(&name)?;
            let conflict = existing.as_ref().map(|_| "name".to_string());
            let recommended_action = if existing.is_some() {
                "rename"
            } else {
                "create"
            }
            .to_string();
            candidates.push(GlobalScanCandidate {
                id: candidate_id(source_tool, "skill", &relative_path),
                asset_type: "skill".to_string(),
                target_asset_type: "skill".to_string(),
                source_tool_id,
                source_tool: source_tool.to_string(),
                source_kind: "skill".to_string(),
                name,
                summary,
                source_path: skill_md,
                relative_path,
                status: if existing.is_some() {
                    "conflict"
                } else {
                    "ready"
                }
                .to_string(),
                conflict,
                existing_id: existing.map(|skill| skill.asset_id),
                default_selected: true,
                selectable: true,
                recommended_action,
                content_preview: preview_text(&body),
                warnings: Vec::new(),
                metadata: json!({ "packageDir": source_dir.display().to_string() }),
                body,
                skill_dir: Some(source_dir),
                credential_token: None,
            });
        }
        Ok(())
    }

    fn scan_unsupported_markdown_root(
        &self,
        root: &Path,
        source_root: &Path,
        source_tool: &str,
        source_tool_id: i32,
        source_kind: &str,
        asset_type: &str,
        candidates: &mut Vec<GlobalScanCandidate>,
        warnings: &mut Vec<String>,
    ) -> Result<(), String> {
        if !source_root.is_dir() {
            return Ok(());
        }
        for path in collect_markdown_files(source_root)? {
            let Some((path, relative_path)) = self.safe_candidate_path(root, &path, warnings)?
            else {
                continue;
            };
            let Some(body) = read_candidate_text(&path, warnings)? else {
                continue;
            };
            let parts = parse_markdown_rule(&body);
            let name = non_empty(
                parts.name,
                path.file_stem()
                    .and_then(|value| value.to_str())
                    .unwrap_or("Prompt"),
            );
            candidates.push(GlobalScanCandidate {
                id: candidate_id(source_tool, source_kind, &relative_path),
                asset_type: asset_type.to_string(),
                target_asset_type: "none".to_string(),
                source_tool_id,
                source_tool: source_tool.to_string(),
                source_kind: source_kind.to_string(),
                name,
                summary: non_empty(parts.description, UNSUPPORTED_WARNING),
                source_path: path,
                relative_path,
                status: "unsupported".to_string(),
                conflict: None,
                existing_id: None,
                default_selected: false,
                selectable: false,
                recommended_action: "unavailable".to_string(),
                content_preview: preview_text(&parts.body),
                warnings: vec![UNSUPPORTED_WARNING.to_string()],
                metadata: json!({ "unsupported": true }),
                body: String::new(),
                skill_dir: None,
                credential_token: None,
            });
        }
        Ok(())
    }

    fn push_codex_provider(
        &self,
        root: &Path,
        path: &Path,
        candidates: &mut Vec<GlobalScanCandidate>,
        warnings: &mut Vec<String>,
    ) -> Result<(), String> {
        if !path.is_file() {
            return Ok(());
        }
        let Some((path, relative_path)) = self.safe_candidate_path(root, path, warnings)? else {
            return Ok(());
        };
        let Some(body) = read_candidate_text(&path, warnings)? else {
            return Ok(());
        };
        let parsed = match ProviderRuntimeService::parse_codex_config(&body) {
            Ok(parsed) => parsed,
            Err(error) => {
                warnings.push(error);
                return Ok(());
            }
        };
        let credential_token = if root.join("auth.json").is_file() {
            match self.safe_candidate_path(root, &root.join("auth.json"), warnings)? {
                Some((auth_path, _)) => read_codex_auth_token(&auth_path, warnings)?,
                None => None,
            }
        } else {
            None
        };
        let credential_detected = credential_token.is_some();
        candidates.push(GlobalScanCandidate {
            id: candidate_id("codex", "codex_config", &relative_path),
            asset_type: "provider_preset".to_string(),
            target_asset_type: "provider".to_string(),
            source_tool_id: CODEX_TOOL_ID,
            source_tool: "codex".to_string(),
            source_kind: "codex_config".to_string(),
            name: parsed.provider_name.clone(),
            summary: format!("{} · {}", parsed.model, parsed.base_url),
            source_path: path,
            relative_path,
            status: "ready".to_string(),
            conflict: None,
            existing_id: None,
            default_selected: true,
            selectable: true,
            recommended_action: "create".to_string(),
            content_preview: String::new(),
            warnings: Vec::new(),
            metadata: json!({
                "toolId": CODEX_TOOL_ID,
                "model": parsed.model,
                "reasoning": parsed.reasoning,
                "baseUrl": parsed.base_url,
                "credentialDetected": credential_detected,
                "credentialSource": if credential_detected { "auth.json" } else { "" },
                "credentialToken": credential_token.clone(),
                "config": parsed.config_json,
            }),
            body: String::new(),
            skill_dir: None,
            credential_token,
        });
        Ok(())
    }

    fn push_claude_provider(
        &self,
        root: &Path,
        path: &Path,
        candidates: &mut Vec<GlobalScanCandidate>,
        warnings: &mut Vec<String>,
    ) -> Result<(), String> {
        if !path.is_file() {
            return Ok(());
        }
        let Some((path, relative_path)) = self.safe_candidate_path(root, path, warnings)? else {
            return Ok(());
        };
        let Some(body) = read_candidate_text(&path, warnings)? else {
            return Ok(());
        };
        let parsed = match ProviderRuntimeService::parse_claude_settings(&body) {
            Ok(parsed) => parsed,
            Err(error) => {
                warnings.push(error);
                return Ok(());
            }
        };
        candidates.push(GlobalScanCandidate {
            id: candidate_id("claude", "claude_settings", &relative_path),
            asset_type: "provider_preset".to_string(),
            target_asset_type: "provider".to_string(),
            source_tool_id: CLAUDE_TOOL_ID,
            source_tool: "claude".to_string(),
            source_kind: "claude_settings".to_string(),
            name: parsed.provider_name.clone(),
            summary: format!("{} · {}", parsed.model, parsed.base_url),
            source_path: path,
            relative_path,
            status: "ready".to_string(),
            conflict: None,
            existing_id: None,
            default_selected: true,
            selectable: true,
            recommended_action: "create".to_string(),
            content_preview: String::new(),
            warnings: Vec::new(),
            metadata: json!({
                "toolId": CLAUDE_TOOL_ID,
                "model": parsed.model,
                "reasoning": parsed.reasoning,
                "baseUrl": parsed.base_url,
                "category": parsed.category,
                "credentialDetected": parsed.credential_detected,
                "credentialSource": parsed.credential_source,
                "credentialToken": parsed.credential_token.clone(),
                "config": parsed.config_json,
            }),
            body: String::new(),
            skill_dir: None,
            credential_token: parsed.credential_token,
        });
        Ok(())
    }

    fn apply_rule_candidate(
        &self,
        candidate: &GlobalScanCandidate,
        conflict_strategy: &str,
        result: &mut FirstRunImportApplyResultDto,
    ) -> Result<(), String> {
        let existing = self.find_rule(&candidate.name)?;
        let (target_id, target_name, operation) = match (existing, conflict_strategy) {
            (Some(_), "skip") => {
                result.skipped += 1;
                result.warnings.push(format!(
                    "Rule '{}' already exists; import skipped.",
                    candidate.name
                ));
                return Ok(());
            }
            (Some(rule), "overwrite") => {
                (Some(rule.asset_id), candidate.name.clone(), "overwritten")
            }
            (Some(_), "rename") => (None, self.next_rule_name(&candidate.name)?, "renamed"),
            (None, _) => (None, candidate.name.clone(), "created"),
            (Some(_), other) => return Err(format!("Unsupported conflict strategy: {other}")),
        };
        let category_code = candidate
            .metadata
            .get("categoryCode")
            .and_then(Value::as_i64)
            .map(|value| value as i32)
            .unwrap_or(RULE_CATEGORY_PERSONAL);
        let db = self.db.lock().expect("db poisoned");
        let repo = ResourceRepo::new(&db);
        let saved = repo.save_rule_version(
            target_id,
            &asset_key(&target_name, "rule"),
            category_code,
            &target_name,
            category_code,
            0,
            TARGET_STATE_READY,
            &candidate.summary,
            &candidate.body,
        )?;
        sync_tool_rule_binding(&repo, &candidate.metadata, candidate.source_tool_id, saved.asset_id)?;
        OperationService::record_simple(
            &db,
            None,
            Some(candidate.source_tool_id),
            Some(saved.asset_id),
            "operation",
            "First-run rule import",
            "first-run-rule-import",
            &format!(
                "Imported global rule '{}' from {}.",
                target_name, candidate.source_tool
            ),
            "success",
            HEALTH_NORMAL,
            Some(&candidate.source_path.display().to_string()),
            ROUTE_RULES,
        )?;
        drop(db);

        if operation == "renamed" {
            result.renamed += 1;
        } else if operation == "overwritten" {
            result.overwritten += 1;
        } else {
            result.imported_rules += 1;
        }
        result.assets.push(FirstRunImportAppliedAssetDto {
            id: saved.asset_id,
            asset_type: "rule".to_string(),
            name: target_name,
            source_tool: candidate.source_tool.clone(),
            source_path: candidate.source_path.display().to_string(),
            operation: operation.to_string(),
        });
        Ok(())
    }

    fn apply_skill_candidate(
        &self,
        candidate: &GlobalScanCandidate,
        conflict_strategy: &str,
        result: &mut FirstRunImportApplyResultDto,
    ) -> Result<(), String> {
        let existing = self.find_skill(&candidate.name)?;
        let (target_id, target_name, operation) = match (existing, conflict_strategy) {
            (Some(_), "skip") => {
                result.skipped += 1;
                result.warnings.push(format!(
                    "Skill '{}' already exists; import skipped.",
                    candidate.name
                ));
                return Ok(());
            }
            (Some(skill), "overwrite") => {
                (Some(skill.asset_id), candidate.name.clone(), "overwritten")
            }
            (Some(_), "rename") => (None, self.next_skill_name(&candidate.name)?, "renamed"),
            (None, _) => (None, candidate.name.clone(), "created"),
            (Some(_), other) => return Err(format!("Unsupported conflict strategy: {other}")),
        };
        let source_dir = candidate
            .skill_dir
            .as_ref()
            .ok_or_else(|| format!("Skill '{}' source directory is missing.", candidate.name))?;
        let db = self.db.lock().expect("db poisoned");
        let repo = ResourceRepo::new(&db);
        let saved = repo.save_skill_version(
            target_id,
            &asset_key(&target_name, "skill"),
            SKILL_CATEGORY_CODING,
            &target_name,
            SKILL_CATEGORY_CODING,
            TARGET_STATE_READY,
            &candidate.summary,
            &candidate.body,
        )?;
        sync_tool_skill_binding(&repo, candidate.source_tool_id, saved.asset_id)?;
        OperationService::record_simple(
            &db,
            None,
            Some(candidate.source_tool_id),
            None,
            "operation",
            "First-run skill import",
            "first-run-skill-import",
            &format!(
                "Imported global skill '{}' from {}.",
                target_name, candidate.source_tool
            ),
            "success",
            HEALTH_NORMAL,
            Some(&candidate.source_path.display().to_string()),
            ROUTE_SKILLS,
        )?;
        drop(db);
        SkillAssetRepo::new(self.context.clone()).write_skill_package(
            &target_name,
            source_dir,
            &candidate.body,
        )?;

        if operation == "renamed" {
            result.renamed += 1;
        } else if operation == "overwritten" {
            result.overwritten += 1;
        } else {
            result.imported_skills += 1;
        }
        result.assets.push(FirstRunImportAppliedAssetDto {
            id: saved.asset_id,
            asset_type: "skill".to_string(),
            name: target_name,
            source_tool: candidate.source_tool.clone(),
            source_path: candidate.source_path.display().to_string(),
            operation: operation.to_string(),
        });
        Ok(())
    }

    fn apply_provider_candidate(
        &self,
        candidate: &GlobalScanCandidate,
        result: &mut FirstRunImportApplyResultDto,
    ) -> Result<(), String> {
        let tool_id = candidate
            .metadata
            .get("toolId")
            .and_then(Value::as_i64)
            .map(|value| value as i32)
            .unwrap_or(candidate.source_tool_id);
        let model = required_metadata_string(&candidate.metadata, "model")?;
        let reasoning = required_metadata_string(&candidate.metadata, "reasoning")?;
        let base_url = required_metadata_string(&candidate.metadata, "baseUrl")?;
        let category = candidate
            .metadata
            .get("category")
            .and_then(Value::as_str)
            .unwrap_or("official");
        let config_json = candidate
            .metadata
            .get("config")
            .cloned()
            .unwrap_or_else(|| json!({}));
        let db = self.db.lock().expect("db poisoned");
        let repo = ProviderRepo::new(&db);
        let existing_provider = repo
            .list(Some(tool_id))?
            .into_iter()
            .find(|provider| provider.provider.name.eq_ignore_ascii_case(&candidate.name));
        let existing_credential_ref = existing_provider
            .as_ref()
            .and_then(|provider| provider.configs.first())
            .map(|config| config.credential_ref.trim().to_string())
            .filter(|value| !value.is_empty());
        let credential_ref = if candidate.credential_token.is_some() {
            existing_credential_ref.unwrap_or_else(|| {
                ProviderRuntimeService::generate_credential_ref(&candidate.name, tool_id)
            })
        } else {
            existing_credential_ref.unwrap_or_default()
        };
        if let Some(token) = candidate
            .credential_token
            .as_deref()
            .map(str::trim)
            .filter(|token| !token.is_empty())
        {
            CredentialStore::save_provider_token(&credential_ref, token)?;
        }
        let provider_id = repo.upsert_provider(
            existing_provider
                .as_ref()
                .map(|provider| provider.provider.id),
            &candidate.name,
            category,
            "",
            &format!(
                "Imported from global {} configuration.",
                candidate.source_tool
            ),
            &[ProviderConfigUpsert {
                id: existing_provider
                    .as_ref()
                    .and_then(|provider| provider.configs.first().map(|config| config.id)),
                tool_id,
                schema_version: 1,
                display_name: candidate.name.clone(),
                model,
                reasoning,
                base_url,
                credential_ref,
                config_json,
            }],
        )?;
        let imported_config = repo.find_config_for_provider_tool(provider_id, tool_id)?;
        repo.activate_config(tool_id, imported_config.id)?;
        OperationService::record_simple(
            &db,
            None,
            Some(tool_id),
            None,
            "operation",
            "First-run provider import",
            "first-run-provider-import",
            &format!(
                "Imported provider preset '{}' from {}.",
                candidate.name, candidate.source_tool
            ),
            "success",
            HEALTH_NORMAL,
            Some(&candidate.source_path.display().to_string()),
            ROUTE_PRESETS,
        )?;
        drop(db);

        let operation = if existing_provider.is_some() {
            "overwritten"
        } else {
            "created"
        };
        if operation == "overwritten" {
            result.overwritten += 1;
        } else {
            result.imported_providers += 1;
        }
        result.assets.push(FirstRunImportAppliedAssetDto {
            id: provider_id,
            asset_type: "provider_preset".to_string(),
            name: candidate.name.clone(),
            source_tool: candidate.source_tool.clone(),
            source_path: candidate.source_path.display().to_string(),
            operation: operation.to_string(),
        });
        Ok(())
    }

    fn find_rule(
        &self,
        name: &str,
    ) -> Result<Option<crate::infrastructure::resource_repo::RuleVersionRecord>, String> {
        let db = self.db.lock().expect("db poisoned");
        ResourceRepo::new(&db).find_latest_rule_version_by_name(name)
    }

    fn find_skill(
        &self,
        name: &str,
    ) -> Result<Option<crate::infrastructure::resource_repo::SkillVersionRecord>, String> {
        let db = self.db.lock().expect("db poisoned");
        ResourceRepo::new(&db).find_latest_skill_version_by_name(name)
    }

    fn next_rule_name(&self, base_name: &str) -> Result<String, String> {
        let db = self.db.lock().expect("db poisoned");
        for index in 2..1000 {
            let candidate = format!("{base_name} ({index})");
            if ResourceRepo::new(&db)
                .find_latest_rule_version_by_name(&candidate)?
                .is_none()
            {
                return Ok(candidate);
            }
        }
        Err(format!(
            "No available import name for rule '{}'.",
            base_name
        ))
    }

    fn next_skill_name(&self, base_name: &str) -> Result<String, String> {
        let db = self.db.lock().expect("db poisoned");
        for index in 2..1000 {
            let candidate = format!("{base_name} ({index})");
            if ResourceRepo::new(&db)
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

    fn safe_candidate_path(
        &self,
        root: &Path,
        path: &Path,
        warnings: &mut Vec<String>,
    ) -> Result<Option<(PathBuf, String)>, String> {
        let canonical_root = root.canonicalize().map_err(|error| error.to_string())?;
        let canonical_path = path.canonicalize().map_err(|error| error.to_string())?;
        if !canonical_path.starts_with(&canonical_root) {
            warnings.push(format!(
                "Skipped path outside {}: {}",
                canonical_root.display(),
                path.display()
            ));
            return Ok(None);
        }
        let relative = canonical_path
            .strip_prefix(&canonical_root)
            .map_err(|error| error.to_string())?
            .to_string_lossy()
            .replace('\\', "/");
        Ok(Some((canonical_path, relative)))
    }

    fn candidate_dto(candidate: GlobalScanCandidate) -> FirstRunImportCandidateDto {
        FirstRunImportCandidateDto {
            id: candidate.id,
            asset_type: candidate.asset_type,
            target_asset_type: candidate.target_asset_type,
            source_tool_id: candidate.source_tool_id,
            source_tool: candidate.source_tool,
            source_kind: candidate.source_kind,
            name: candidate.name,
            summary: candidate.summary,
            source_path: candidate.source_path.display().to_string(),
            relative_path: candidate.relative_path,
            status: candidate.status,
            conflict: candidate.conflict,
            existing_id: candidate.existing_id,
            default_selected: candidate.default_selected,
            selectable: candidate.selectable,
            recommended_action: candidate.recommended_action,
            content_preview: candidate.content_preview,
            warnings: candidate.warnings,
            metadata: candidate.metadata,
        }
    }
}

fn sync_tool_rule_binding(
    repo: &ResourceRepo<'_>,
    metadata: &Value,
    tool_id: i32,
    rule_asset_id: i32,
) -> Result<(), String> {
    let source_tool_ids = metadata
        .get("sourceToolIds")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_i64)
                .map(|value| value as i32)
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| vec![tool_id]);

    for source_tool_id in source_tool_ids {
        let mut asset_ids = repo
            .tool_global_rule_binding(source_tool_id)?
            .map(|binding| {
                binding
                    .items
                    .into_iter()
                    .filter(|item| item.item_type == "rule")
                    .map(|item| item.asset_id)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        if !asset_ids.contains(&rule_asset_id) {
            asset_ids.push(rule_asset_id);
        }
        repo.replace_tool_global_rule_binding_from_rules(source_tool_id, &asset_ids)?;
    }

    Ok(())
}

fn sync_tool_skill_binding(repo: &ResourceRepo<'_>, tool_id: i32, skill_asset_id: i32) -> Result<(), String> {
    let mut asset_ids = repo
        .tool_skill_binding(tool_id)?
        .map(|binding| {
            binding
                .items
                .into_iter()
                .filter(|item| item.item_type == "skill")
                .map(|item| item.asset_id)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    if !asset_ids.contains(&skill_asset_id) {
        asset_ids.push(skill_asset_id);
    }
    repo.replace_tool_skill_binding_from_skills(tool_id, &asset_ids)?;
    Ok(())
}

fn collect_markdown_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut result = Vec::new();
    collect_markdown_files_inner(root, &mut result)?;
    result.sort();
    Ok(result)
}

fn collect_markdown_files_inner(root: &Path, result: &mut Vec<PathBuf>) -> Result<(), String> {
    for entry in fs::read_dir(root).map_err(|error| error.to_string())? {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
        if metadata.file_type().is_symlink() {
            continue;
        }
        if metadata.is_dir() {
            collect_markdown_files_inner(&path, result)?;
        } else if path
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
        {
            result.push(path);
        }
    }
    Ok(())
}

fn read_candidate_text(path: &Path, warnings: &mut Vec<String>) -> Result<Option<String>, String> {
    match read_text_limited(path) {
        Ok(content) => Ok(Some(content)),
        Err(error) => {
            warnings.push(error);
            Ok(None)
        }
    }
}

fn read_text_limited(path: &Path) -> Result<String, String> {
    let metadata = fs::metadata(path).map_err(|error| error.to_string())?;
    if metadata.len() > MAX_TEXT_FILE_BYTES {
        return Err(format!(
            "File is too large for first-run import: {}",
            path.display()
        ));
    }
    fs::read_to_string(path).map_err(|error| error.to_string())
}

fn preview_text(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.chars().count() <= 1200 {
        trimmed.to_string()
    } else {
        format!("{}…", trimmed.chars().take(1200).collect::<String>())
    }
}

fn candidate_id(source_tool: &str, source_kind: &str, relative_path: &str) -> String {
    format!(
        "{source_tool}:{source_kind}:{}",
        relative_path.replace('\\', "/")
    )
}

fn parse_managed_rule_sections(body: &str) -> Vec<ManagedRuleSection> {
    if !body.contains(MANAGED_MARKER) {
        return Vec::new();
    }

    let parts = parse_markdown_rule(body);
    let mut sections = Vec::new();
    let mut current_name = String::new();
    let mut current_body = Vec::<String>::new();

    for line in parts.body.lines() {
        if let Some(name) = managed_rule_heading_name(line) {
            if !current_name.is_empty() || !current_body.is_empty() {
                push_managed_rule_section(&mut sections, &current_name, &current_body);
            }
            current_name = name;
            current_body.clear();
            continue;
        }

        if !current_name.is_empty() {
            current_body.push(line.to_string());
        }
    }

    if !current_name.is_empty() || !current_body.is_empty() {
        push_managed_rule_section(&mut sections, &current_name, &current_body);
    }

    sections
}

fn managed_rule_heading_name(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix("## ")?.trim();
    let (_, name_with_version) = rest.split_once('.')?;
    let name = name_with_version
        .trim()
        .split_once(" `v")
        .map(|(name, _)| name)
        .unwrap_or(name_with_version)
        .trim();
    if name.is_empty() || name == "Untitled rule" {
        None
    } else {
        Some(name.to_string())
    }
}

fn push_managed_rule_section(sections: &mut Vec<ManagedRuleSection>, name: &str, body_lines: &[String]) {
    let body = body_lines.join("\n").trim().trim_matches('-').trim().to_string();
    if name.trim().is_empty() || body.is_empty() {
        return;
    }
    sections.push(ManagedRuleSection {
        name: name.trim().to_string(),
        body,
    });
}

fn rule_name_from_path(path: &Path, fallback: &str) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn non_empty(value: String, fallback: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

fn asset_key(name: &str, fallback: &str) -> String {
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
    let key = key.trim_matches('-');
    if key.is_empty() {
        fallback.to_string()
    } else {
        key.to_string()
    }
}

fn read_codex_auth_token(path: &Path, warnings: &mut Vec<String>) -> Result<Option<String>, String> {
    if !path.is_file() {
        return Ok(None);
    }
    let Some(body) = read_candidate_text(path, warnings)? else {
        return Ok(None);
    };
    let value = match serde_json::from_str::<Value>(&body) {
        Ok(value) => value,
        Err(error) => {
            warnings.push(format!("Failed to parse Codex auth.json: {error}"));
            return Ok(None);
        }
    };
    Ok(value
        .get("OPENAI_API_KEY")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string))
}

fn required_metadata_string(metadata: &Value, key: &str) -> Result<String, String> {
    metadata
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| format!("Provider metadata is missing {key}."))
}

fn current_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}
