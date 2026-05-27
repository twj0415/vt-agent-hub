use rusqlite::params;

use super::{ResourceRepo, ToolSkillInstallRecord};

impl<'a> ResourceRepo<'a> {
    pub fn list_tool_skill_installs(
        &self,
        tool_id: i32,
    ) -> Result<Vec<ToolSkillInstallRecord>, String> {
        let mut stmt = self
            .db
            .connection()
            .prepare(
                r#"
                select tool_id, skill_asset_id, required_version_id, installed_version_id, state, updated_at
                from tool_skill_installs
                where tool_id = ?1
                order by skill_asset_id asc
                "#,
            )
            .map_err(|error| error.to_string())?;

        let rows = stmt
            .query_map(params![tool_id], |row| {
                Ok(ToolSkillInstallRecord {
                    tool_id: row.get(0)?,
                    skill_asset_id: row.get(1)?,
                    required_version_id: row.get(2)?,
                    installed_version_id: row.get(3)?,
                    state: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            })
            .map_err(|error| error.to_string())?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }

    pub fn sync_required_tool_skill_installs(&self, tool_id: i32) -> Result<(), String> {
        let Some(binding) = self.tool_skill_binding(tool_id)? else {
            return Ok(());
        };

        let tx = self
            .db
            .connection()
            .unchecked_transaction()
            .map_err(|error| error.to_string())?;

        let items = binding
            .items
            .into_iter()
            .filter(|item| item.item_type == "skill")
            .map(|item| (item.asset_id, item.asset_version_id))
            .collect::<Vec<_>>();

        let desired_asset_ids = items
            .iter()
            .map(|(asset_id, _)| *asset_id)
            .collect::<Vec<_>>();
        let existing_asset_ids = {
            let mut existing_stmt = tx
                .prepare("select skill_asset_id from tool_skill_installs where tool_id = ?1")
                .map_err(|error| error.to_string())?;
            let rows = existing_stmt
                .query_map(params![tool_id], |row| row.get::<_, i32>(0))
                .map_err(|error| error.to_string())?;
            rows.collect::<Result<Vec<_>, _>>()
                .map_err(|error| error.to_string())?
        };
        for existing_asset_id in existing_asset_ids {
            if !desired_asset_ids.contains(&existing_asset_id) {
                tx.execute(
                    "delete from tool_skill_installs where tool_id = ?1 and skill_asset_id = ?2",
                    params![tool_id, existing_asset_id],
                )
                .map_err(|error| error.to_string())?;
            }
        }

        for (asset_id, version_id) in items {
            tx.execute(
                r#"
                insert into tool_skill_installs (tool_id, skill_asset_id, required_version_id, installed_version_id, state, updated_at)
                values (?1, ?2, ?3, null, 'not_installed', datetime('now'))
                on conflict(tool_id, skill_asset_id) do update set required_version_id = excluded.required_version_id
                "#,
                params![tool_id, asset_id, version_id],
            )
            .map_err(|error| error.to_string())?;
        }

        tx.commit().map_err(|error| error.to_string())
    }

    pub fn clear_tool_skill_installs(&self, tool_id: i32) -> Result<(), String> {
        self.db
            .connection()
            .execute(
                "delete from tool_skill_installs where tool_id = ?1",
                params![tool_id],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }
}
