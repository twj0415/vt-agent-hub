use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::product::LEGACY_MANAGED_MARKER;
use crate::infrastructure::database::Database;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectOutputState {
    pub target_path: PathBuf,
    pub target_exists: bool,
    pub managed: bool,
    pub content: String,
}

pub struct ProjectOutputRepo;

impl ProjectOutputRepo {
    pub fn inspect(target_path: &Path, managed_marker: &str) -> Result<ProjectOutputState, String> {
        let target_exists = target_path.exists();
        let content = if target_exists {
            fs::read_to_string(target_path).map_err(|error| error.to_string())?
        } else {
            String::new()
        };

        Ok(ProjectOutputState {
            target_path: target_path.to_path_buf(),
            target_exists,
            managed: Self::is_managed(&content, managed_marker),
            content,
        })
    }

    pub fn write(target_path: &Path, content: &str) -> Result<(), String> {
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }

        fs::write(target_path, content).map_err(|error| {
            format!(
                "Failed to write AGENTS.md {}: {error}",
                target_path.display()
            )
        })
    }

    pub fn backup(project_id: i32, target_path: &Path) -> Result<String, String> {
        let file_name = target_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("AGENTS.md");
        let backup_dir = Database::backups_root()?.join(format!("project-{}", project_id));
        fs::create_dir_all(&backup_dir).map_err(|error| error.to_string())?;

        let backup_path = backup_dir.join(format!("{}-{}", Self::timestamp_value(), file_name));

        fs::copy(target_path, &backup_path).map_err(|error| {
            format!(
                "Failed to backup AGENTS.md {}: {error}",
                target_path.display()
            )
        })?;

        Ok(backup_path.display().to_string())
    }

    pub fn delete(target_path: &Path) -> Result<(), String> {
        if !target_path.exists() {
            return Ok(());
        }

        fs::remove_file(target_path).map_err(|error| {
            format!(
                "Failed to delete AGENTS.md {}: {error}",
                target_path.display()
            )
        })
    }

    pub fn is_managed(content: &str, managed_marker: &str) -> bool {
        content.contains(managed_marker) || content.contains(LEGACY_MANAGED_MARKER)
    }

    fn timestamp_value() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0)
    }
}
