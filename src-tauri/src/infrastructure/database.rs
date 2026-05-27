use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::{params, Connection};

use crate::core::library_layout::LIBRARY_AREAS;
use crate::core::paths;
use crate::core::product::DATA_DIR_NAME;
use crate::core::tool_registry::TOOL_REGISTRY;
use crate::infrastructure::migrations;

pub struct Database {
    conn: Connection,
    path: PathBuf,
}

impl Database {
    pub fn open_at(path: &Path) -> Result<Self, String> {
        let parent = path
            .parent()
            .ok_or_else(|| "Invalid database path".to_string())?;

        fs::create_dir_all(parent).map_err(|error| error.to_string())?;

        let conn = Connection::open(path).map_err(|error| error.to_string())?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|error| error.to_string())?;
        let db = Self {
            conn,
            path: path.to_path_buf(),
        };
        db.init()?;
        Ok(db)
    }

    pub fn open_default() -> Result<Self, String> {
        Self::ensure_storage_layout()?;
        let db_path = Self::default_path()?;
        Self::open_at(&db_path)
    }

    pub fn default_path() -> Result<PathBuf, String> {
        Ok(Self::storage_root()?.join("app.db"))
    }

    pub fn storage_root() -> Result<PathBuf, String> {
        paths::app_storage_root(DATA_DIR_NAME)
    }

    pub fn library_root() -> Result<PathBuf, String> {
        Ok(Self::storage_root()?.join("library"))
    }

    pub fn backups_root() -> Result<PathBuf, String> {
        Ok(Self::storage_root()?.join("backups"))
    }

    pub fn logs_root() -> Result<PathBuf, String> {
        Ok(Self::storage_root()?.join("logs"))
    }

    pub fn snapshots_root() -> Result<PathBuf, String> {
        Ok(Self::storage_root()?.join("snapshots"))
    }

    pub fn runtime_root() -> Result<PathBuf, String> {
        Ok(Self::storage_root()?.join("runtime"))
    }

    fn ensure_storage_layout() -> Result<(), String> {
        let roots = [
            Self::storage_root()?,
            Self::library_root()?,
            Self::backups_root()?,
            Self::logs_root()?,
            Self::snapshots_root()?,
            Self::runtime_root()?,
        ];

        for root in roots {
            fs::create_dir_all(root).map_err(|error| error.to_string())?;
        }

        let library_root = Self::library_root()?;
        for area in LIBRARY_AREAS {
            fs::create_dir_all(library_root.join(area.relative_path))
                .map_err(|error| error.to_string())?;
        }

        Ok(())
    }

    fn sync_library_root_setting(&self) -> Result<(), String> {
        let library_root = Self::library_root()?.display().to_string();
        self.conn
            .execute(
                "insert or replace into settings (key, value) values (?1, ?2)",
                params!["library_root", library_root],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn init(&self) -> Result<(), String> {
        migrations::run(&self.conn)?;
        self.seed()?;
        Ok(())
    }

    fn seed(&self) -> Result<(), String> {
        for tool in TOOL_REGISTRY {
            let name = match tool.key {
                "codex" => "Codex",
                "claude" => "Claude",
                "cursor" => "Cursor",
                key => key,
            };
            self.conn
                .execute(
                    "insert into tools (id, name, enabled) values (?1, ?2, ?3) \
                     on conflict(id) do update set name = excluded.name, enabled = excluded.enabled",
                    params![tool.id, name, i32::from(tool.enabled)],
                )
                .map_err(|error| error.to_string())?;
        }
        self.conn
            .execute(
                "insert or ignore into settings (key, value) values (?1, ?2)",
                params!["theme", "system"],
            )
            .map_err(|error| error.to_string())?;
        self.conn
            .execute(
                "insert or ignore into settings (key, value) values (?1, ?2)",
                params!["library_root", Self::library_root()?.display().to_string()],
            )
            .map_err(|error| error.to_string())?;
        self.sync_library_root_setting()?;
        self.purge_legacy_plaintext_credentials()?;
        Ok(())
    }

    fn purge_legacy_plaintext_credentials(&self) -> Result<(), String> {
        self.conn
            .execute(
                "delete from settings where key like 'tool_%_credential' and key not like 'tool_%_credential_state'",
                [],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    pub fn clone_for_internal_use(&self) -> Result<Self, String> {
        Self::open_at(&self.path)
    }
}
