use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::application::app_container::AppContainer;
use crate::application::operation_service::OperationService;
use crate::application::service_context::ServiceContext;
use crate::dto::{
    BackupActionResultDto, BackupEntryDto, BackupRestorePreviewDto, BackupSnapshotDto,
};
use crate::infrastructure::codex_runtime_repo::CodexRuntimeRepo;
use crate::infrastructure::database::Database;
use crate::infrastructure::project_repo::ProjectRepo;

pub struct BackupService {
    db: Arc<Mutex<Database>>,
    backups_root: PathBuf,
}

impl BackupService {
    pub fn new() -> Result<Self, String> {
        Self::with_context(ServiceContext::default()?)
    }

    pub fn with_context(context: ServiceContext) -> Result<Self, String> {
        let backups_root = context.storage_root()?.join("backups");
        let db = context.open_db()?;
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            backups_root,
        })
    }

    pub fn with_container(container: &AppContainer) -> Result<Self, String> {
        let backups_root = container.context().storage_root()?.join("backups");
        Ok(Self {
            db: container.db(),
            backups_root,
        })
    }

    pub fn list(&self) -> Result<BackupSnapshotDto, String> {
        let root = self.backups_root.clone();
        let mut entries = Vec::new();
        if root.exists() {
            self.collect_entries(&root, &mut entries)?;
        }
        entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(BackupSnapshotDto { entries })
    }

    pub fn preview_restore(&self, backup_id: &str) -> Result<BackupRestorePreviewDto, String> {
        let entry = self.resolve_entry(backup_id)?;
        let before_content = if Path::new(&entry.target_path).exists() {
            fs::read_to_string(&entry.target_path).map_err(|error| error.to_string())?
        } else {
            String::new()
        };
        let after_content = fs::read_to_string(&entry.path).map_err(|error| error.to_string())?;

        Ok(BackupRestorePreviewDto {
            backup_id: entry.id,
            backup_path: entry.path,
            target_path: entry.target_path.clone(),
            target_exists: Path::new(&entry.target_path).exists(),
            before_content: before_content.clone(),
            after_content: after_content.clone(),
            diff: format!(
                "--- live target\n{}\n+++ backup\n{}",
                before_content, after_content
            ),
            warning: Some("Restoring overwrites the live target after confirmation.".to_string()),
        })
    }

    pub fn restore(
        &self,
        backup_id: &str,
        confirm_risk: bool,
    ) -> Result<BackupActionResultDto, String> {
        if !confirm_risk {
            return Err("Risk confirmation is required before restoring a backup.".to_string());
        }
        let preview = self.preview_restore(backup_id)?;
        let target_path = PathBuf::from(&preview.target_path);
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::write(&target_path, preview.after_content).map_err(|error| error.to_string())?;

        let db = self.db.lock().expect("db poisoned");
        OperationService::record(
            &db,
            None,
            "repair",
            "Backup restore",
            "backup-restore",
            &format!(
                "Restored backup {} to {}.",
                preview.backup_path, preview.target_path
            ),
        )?;

        Ok(BackupActionResultDto {
            ok: true,
            message: "Backup restored.".to_string(),
            path: preview.target_path,
        })
    }

    pub fn delete(&self, backup_id: &str) -> Result<BackupActionResultDto, String> {
        let entry = self.resolve_entry(backup_id)?;
        fs::remove_file(&entry.path).map_err(|error| error.to_string())?;
        let db = self.db.lock().expect("db poisoned");
        OperationService::record(
            &db,
            None,
            "operation",
            "Backup delete",
            "backup-delete",
            &format!("Deleted backup {}.", entry.path),
        )?;
        Ok(BackupActionResultDto {
            ok: true,
            message: "Backup deleted.".to_string(),
            path: entry.path,
        })
    }

    fn resolve_entry(&self, backup_id: &str) -> Result<BackupEntryDto, String> {
        self.list()?
            .entries
            .into_iter()
            .find(|entry| entry.id == backup_id)
            .ok_or_else(|| format!("Backup {} does not exist.", backup_id))
    }

    fn collect_entries(
        &self,
        root: &Path,
        entries: &mut Vec<BackupEntryDto>,
    ) -> Result<(), String> {
        for entry in fs::read_dir(root).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let path = entry.path();
            if path.is_dir() {
                self.collect_entries(&path, entries)?;
                continue;
            }
            if !path.is_file() {
                continue;
            }
            if let Some(item) = self.entry_for_path(&path)? {
                entries.push(item);
            }
        }
        Ok(())
    }

    fn entry_for_path(&self, path: &Path) -> Result<Option<BackupEntryDto>, String> {
        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("backup")
            .to_string();
        let parent_name = path
            .parent()
            .and_then(|parent| parent.file_name())
            .and_then(|value| value.to_str())
            .unwrap_or("");
        let project_id = parent_name
            .strip_prefix("project-")
            .and_then(|value| value.parse::<i32>().ok());
        let target_path = self.target_for(project_id, &file_name)?;
        if target_path.is_none() {
            return Ok(None);
        }
        let metadata = fs::metadata(path).map_err(|error| error.to_string())?;
        let created_at = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs().to_string())
            .unwrap_or_else(|| "0".to_string());
        let id = path.display().to_string();

        Ok(Some(BackupEntryDto {
            id: id.clone(),
            scope: Self::scope_for(project_id, &file_name),
            project_id,
            file_name,
            path: id,
            target_path: target_path.unwrap().display().to_string(),
            created_at,
            size: metadata.len(),
        }))
    }

    fn target_for(
        &self,
        project_id: Option<i32>,
        file_name: &str,
    ) -> Result<Option<PathBuf>, String> {
        let normalized = file_name.to_ascii_lowercase();
        if normalized.ends_with("config.toml") {
            return Ok(Some(CodexRuntimeRepo::config_path()));
        }
        if normalized.ends_with("agents.md") && project_id == Some(0) {
            return Ok(Some(CodexRuntimeRepo::global_agents_path()));
        }
        if normalized.ends_with("agents.md") {
            if let Some(project_id) = project_id {
                let db = self.db.lock().expect("db poisoned");
                let Some(project) = ProjectRepo::new(&db).find_optional(project_id)? else {
                    return Ok(None);
                };
                return Ok(Some(PathBuf::from(project.path).join("AGENTS.md")));
            }
        }
        Ok(None)
    }

    fn scope_for(project_id: Option<i32>, file_name: &str) -> String {
        let normalized = file_name.to_ascii_lowercase();
        if normalized.ends_with("config.toml") {
            "config".to_string()
        } else if project_id == Some(0) {
            "global".to_string()
        } else {
            "project".to_string()
        }
    }
}

#[allow(dead_code)]
fn timestamp_value() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::service_context::ServiceContext;

    #[test]
    fn list_skips_backups_for_missing_projects() {
        let root = std::env::temp_dir().join(format!(
            "vt-agent-hub-backup-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or(0)
        ));
        let db_path = root.join("state").join("app.db");
        let context = ServiceContext::at_db(&db_path);
        let service = BackupService::with_context(context).expect("service should initialize");

        let backup_dir = root.join("state").join("backups").join("project-1");
        std::fs::create_dir_all(&backup_dir).expect("backup dir should create");
        std::fs::write(backup_dir.join("123-AGENTS.md"), "backup")
            .expect("backup file should write");

        let snapshot = service.list().expect("backup list should not fail");
        assert!(snapshot.entries.is_empty());
    }
}
