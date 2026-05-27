use std::fs;
use std::path::PathBuf;

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
}
