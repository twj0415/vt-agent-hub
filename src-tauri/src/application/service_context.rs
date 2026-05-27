use std::path::{Path, PathBuf};

use crate::infrastructure::database::Database;
use crate::infrastructure::resource_repo::ResourceRepo;
use crate::infrastructure::skill_asset_repo::SkillAssetRepo;

#[derive(Debug, Clone)]
pub struct ServiceContext {
    db_path: PathBuf,
    default_storage: bool,
}

impl ServiceContext {
    pub fn default() -> Result<Self, String> {
        Ok(Self {
            db_path: Database::default_path()?,
            default_storage: true,
        })
    }

    pub fn at_db(path: impl AsRef<Path>) -> Self {
        Self {
            db_path: path.as_ref().to_path_buf(),
            default_storage: false,
        }
    }

    pub fn open_db(&self) -> Result<Database, String> {
        let db = if self.default_storage {
            Database::open_default()?
        } else {
            Database::open_at(&self.db_path)?
        };

        if let Ok(library_root) = self.library_root() {
            db.connection()
                .execute(
                    "insert or replace into settings (key, value) values (?1, ?2)",
                    rusqlite::params!["library_root", library_root.display().to_string()],
                )
                .map_err(|error| error.to_string())?;
        }

        self.sync_library_mirrors(&db)?;

        Ok(db)
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    pub fn storage_root(&self) -> Result<PathBuf, String> {
        self.db_path
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| "Invalid database path".to_string())
    }

    pub fn library_root(&self) -> Result<PathBuf, String> {
        Ok(self.storage_root()?.join("library"))
    }

    fn sync_library_mirrors(&self, db: &Database) -> Result<(), String> {
        let library_root = self.library_root()?;
        if library_root.exists() && !library_root.is_dir() {
            return Ok(());
        }

        let resource_repo = ResourceRepo::new(db);
        let skill_repo = SkillAssetRepo::new(self.clone());
        for skill in resource_repo.list_latest_skill_versions()? {
            skill_repo.write_skill(&skill.name, &skill.body)?;
        }

        Ok(())
    }
}
