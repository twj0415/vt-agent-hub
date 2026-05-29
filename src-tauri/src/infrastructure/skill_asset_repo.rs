use std::fs;
use std::path::{Path, PathBuf};

use crate::application::service_context::ServiceContext;

pub struct SkillAssetRepo {
    context: ServiceContext,
}

impl SkillAssetRepo {
    pub fn new(context: ServiceContext) -> Self {
        Self { context }
    }

    pub fn skill_root(&self, name: &str) -> Result<PathBuf, String> {
        Ok(self.context.library_root()?.join("skills").join(name))
    }

    pub fn write_skill(&self, name: &str, body: &str) -> Result<(), String> {
        let root = self.skill_root(name)?;
        fs::create_dir_all(&root).map_err(|error| error.to_string())?;
        fs::write(root.join("SKILL.md"), body).map_err(|error| error.to_string())
    }

    pub fn write_skill_package(
        &self,
        name: &str,
        source_dir: &Path,
        body: &str,
    ) -> Result<(), String> {
        let root = self.skill_root(name)?;
        let parent = root
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| "Invalid skill package target.".to_string())?;
        fs::create_dir_all(&parent).map_err(|error| error.to_string())?;

        let suffix = Self::unique_suffix();
        let staging = parent.join(format!(".{name}.staging-{suffix}"));
        let backup = parent.join(format!(".{name}.backup-{suffix}"));

        let result = (|| {
            fs::create_dir_all(&staging).map_err(|error| error.to_string())?;
            self.copy_package_dir(source_dir, source_dir, &staging)?;
            fs::write(staging.join("SKILL.md"), body).map_err(|error| error.to_string())?;

            if root.exists() {
                fs::rename(&root, &backup).map_err(|error| error.to_string())?;
            }
            fs::rename(&staging, &root).map_err(|error| error.to_string())?;
            if backup.exists() {
                fs::remove_dir_all(&backup).map_err(|error| error.to_string())?;
            }
            Ok(())
        })();

        if result.is_err() {
            let _ = fs::remove_dir_all(&staging);
            if backup.exists() && !root.exists() {
                let _ = fs::rename(&backup, &root);
            }
        }

        result
    }

    pub fn rename_skill(&self, from: &str, to: &str) -> Result<(), String> {
        if from.eq_ignore_ascii_case(to) {
            return Ok(());
        }

        let source = self.skill_root(from)?;
        if !source.exists() {
            return Ok(());
        }

        let target = self.skill_root(to)?;
        if target.exists() {
            fs::remove_dir_all(&target).map_err(|error| error.to_string())?;
        }

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }

        fs::rename(source, target).map_err(|error| error.to_string())
    }

    pub fn delete_skill(&self, name: &str) -> Result<(), String> {
        let root = self.skill_root(name)?;
        if root.exists() {
            fs::remove_dir_all(root).map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    fn unique_suffix() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0)
    }

    fn copy_package_dir(
        &self,
        source_root: &Path,
        current: &Path,
        target_root: &Path,
    ) -> Result<(), String> {
        for entry in fs::read_dir(current).map_err(|error| error.to_string())? {
            let entry = entry.map_err(|error| error.to_string())?;
            let source_path = entry.path();
            let file_name = entry.file_name();
            if file_name.to_string_lossy().starts_with('.') {
                continue;
            }
            let metadata = fs::symlink_metadata(&source_path).map_err(|error| error.to_string())?;
            if metadata.file_type().is_symlink() {
                continue;
            }

            let relative = source_path
                .strip_prefix(source_root)
                .map_err(|error| error.to_string())?;
            if relative
                .components()
                .any(|component| matches!(component, std::path::Component::ParentDir))
            {
                continue;
            }
            let target_path = target_root.join(relative);

            if metadata.is_dir() {
                fs::create_dir_all(&target_path).map_err(|error| error.to_string())?;
                self.copy_package_dir(source_root, &source_path, target_root)?;
            } else if metadata.is_file() {
                if metadata.len() > 1024 * 1024 {
                    continue;
                }
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
                }
                fs::copy(&source_path, &target_path).map_err(|error| error.to_string())?;
            }
        }
        Ok(())
    }
}
