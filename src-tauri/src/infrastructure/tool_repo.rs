use rusqlite::{params, OptionalExtension};

use crate::infrastructure::credential_store::CredentialStore;
use crate::infrastructure::database::Database;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolRecord {
    pub id: i32,
    pub name: String,
    pub enabled: bool,
}

pub struct ToolRepo<'a> {
    db: &'a Database,
}

impl<'a> ToolRepo<'a> {
    pub fn new(db: &'a Database) -> Self {
        Self { db }
    }

    pub fn list(&self) -> Result<Vec<ToolRecord>, String> {
        let mut stmt = self
            .db
            .connection()
            .prepare("select id, name, enabled from tools where enabled = 1 order by id asc")
            .map_err(|error| error.to_string())?;

        let rows = stmt
            .query_map([], |row| {
                let enabled: i32 = row.get(2)?;

                Ok(ToolRecord {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    enabled: enabled == 1,
                })
            })
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;

        Ok(rows)
    }

    pub fn set_enabled(&self, tool_id: i32, enabled: bool) -> Result<(), String> {
        let exists = self
            .db
            .connection()
            .query_row(
                "select 1 from tools where id = ?1",
                params![tool_id],
                |_| Ok(()),
            )
            .optional()
            .map_err(|error| error.to_string())?
            .is_some();
        if !exists {
            return Err(format!("Tool {} does not exist.", tool_id));
        }

        let credential_refs = if enabled {
            Vec::new()
        } else {
            self.collect_provider_credential_refs(tool_id)?
        };

        let tx = self
            .db
            .connection()
            .unchecked_transaction()
            .map_err(|error| error.to_string())?;
        tx.execute(
            "update tools set enabled = ?1 where id = ?2",
            params![i32::from(enabled), tool_id],
        )
        .map_err(|error| error.to_string())?;

        if !enabled {
            tx.execute(
                "delete from project_rule_bindings where tool_id = ?1",
                params![tool_id],
            )
            .map_err(|error| error.to_string())?;
            tx.execute(
                "delete from tool_global_rule_bindings where tool_id = ?1",
                params![tool_id],
            )
            .map_err(|error| error.to_string())?;
            tx.execute(
                "delete from tool_skill_bindings where tool_id = ?1",
                params![tool_id],
            )
            .map_err(|error| error.to_string())?;
            tx.execute(
                "delete from tool_skill_installs where tool_id = ?1",
                params![tool_id],
            )
            .map_err(|error| error.to_string())?;
            tx.execute(
                "delete from provider_tool_configs where tool_id = ?1",
                params![tool_id],
            )
            .map_err(|error| error.to_string())?;
        }

        tx.commit().map_err(|error| error.to_string())?;

        for credential_ref in credential_refs {
            let _ = CredentialStore::clear_provider_token(&credential_ref);
        }

        Ok(())
    }

    fn collect_provider_credential_refs(&self, tool_id: i32) -> Result<Vec<String>, String> {
        let mut stmt = self
            .db
            .connection()
            .prepare(
                "select credential_ref from provider_tool_configs \
                 where tool_id = ?1 and credential_ref != ''",
            )
            .map_err(|error| error.to_string())?;
        let rows = stmt
            .query_map(params![tool_id], |row| row.get::<_, String>(0))
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }
}
