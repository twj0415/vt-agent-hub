use rusqlite::{params, OptionalExtension};

use crate::infrastructure::database::Database;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingRecord {
    pub id: i32,
    pub key: String,
    pub value: String,
}

pub struct SettingsRepo<'a> {
    db: &'a Database,
}

impl<'a> SettingsRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn list(&self) -> Result<Vec<SettingRecord>, String> {
        let mut stmt = self
            .db
            .connection()
            .prepare("select rowid, key, value from settings order by key asc")
            .map_err(|error| error.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                Ok(SettingRecord {
                    id: row.get(0)?,
                    key: row.get(1)?,
                    value: row.get(2)?,
                })
            })
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;

        Ok(rows)
    }

    pub fn get(&self, key: &str) -> Result<Option<String>, String> {
        self.db
            .connection()
            .query_row(
                "select value from settings where key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())
    }

    pub fn upsert(&self, key: &str, value: &str) -> Result<(), String> {
        self.db
            .connection()
            .execute(
                "insert into settings (key, value) values (?1, ?2) on conflict(key) do update set value = excluded.value",
                params![key, value],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn delete(&self, key: &str) -> Result<(), String> {
        self.db
            .connection()
            .execute("delete from settings where key = ?1", params![key])
            .map_err(|error| error.to_string())?;
        Ok(())
    }
}
