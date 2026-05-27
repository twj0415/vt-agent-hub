use rusqlite::params;

use super::{ResourceRepo, RuleImpactRecord, RuleVersionRecord, SkillVersionRecord};

impl<'a> ResourceRepo<'a> {
    pub fn save_rule_version(
        &self,
        asset_id: Option<i32>,
        asset_key: &str,
        code: i32,
        name: &str,
        category_code: i32,
        sort_order: i32,
        state: i32,
        summary: &str,
        body: &str,
    ) -> Result<RuleVersionRecord, String> {
        let tx = self
            .db
            .connection()
            .unchecked_transaction()
            .map_err(|error| error.to_string())?;

        let asset_id = if let Some(asset_id) = asset_id {
            tx.execute(
                "update rule_assets set asset_key = ?1 where id = ?2",
                params![asset_key, asset_id],
            )
            .map_err(|error| error.to_string())?;
            asset_id
        } else {
            tx.execute(
                "insert into rule_assets (asset_key) values (?1)",
                params![asset_key],
            )
            .map_err(|error| error.to_string())?;
            tx.last_insert_rowid() as i32
        };

        let version_no: i32 = tx
            .query_row(
                "select coalesce(max(version_no), 0) + 1 from rule_versions where rule_asset_id = ?1",
                params![asset_id],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;

        let hash = format!("rule-{}-{}", version_no, body.len());
        tx.execute(
            "insert into rule_versions (rule_asset_id, version_no, code, name, category_code, sort_order, state, summary, body, hash) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![asset_id, version_no, code, name, category_code, sort_order, state, summary, body, hash],
        )
        .map_err(|error| error.to_string())?;
        let version_id = tx.last_insert_rowid() as i32;

        tx.commit().map_err(|error| error.to_string())?;
        self.find_rule_version(version_id)
    }

    pub fn save_skill_version(
        &self,
        asset_id: Option<i32>,
        asset_key: &str,
        code: i32,
        name: &str,
        category_code: i32,
        state: i32,
        summary: &str,
        body: &str,
    ) -> Result<SkillVersionRecord, String> {
        let tx = self
            .db
            .connection()
            .unchecked_transaction()
            .map_err(|error| error.to_string())?;

        let asset_id = if let Some(asset_id) = asset_id {
            tx.execute(
                "update skill_assets set asset_key = ?1 where id = ?2",
                params![asset_key, asset_id],
            )
            .map_err(|error| error.to_string())?;
            asset_id
        } else {
            tx.execute(
                "insert into skill_assets (asset_key) values (?1)",
                params![asset_key],
            )
            .map_err(|error| error.to_string())?;
            tx.last_insert_rowid() as i32
        };

        let version_no: i32 = tx
            .query_row(
                "select coalesce(max(version_no), 0) + 1 from skill_versions where skill_asset_id = ?1",
                params![asset_id],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;

        let hash = format!("skill-{}-{}", version_no, body.len());
        tx.execute(
            "insert into skill_versions (skill_asset_id, version_no, code, name, category_code, state, summary, body, hash) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![asset_id, version_no, code, name, category_code, state, summary, body, hash],
        )
        .map_err(|error| error.to_string())?;
        let version_id = tx.last_insert_rowid() as i32;

        tx.commit().map_err(|error| error.to_string())?;
        self.find_skill_version(version_id)
    }

    pub fn delete_rule_asset(&self, asset_id: i32) -> Result<(), String> {
        let tx = self
            .db
            .connection()
            .unchecked_transaction()
            .map_err(|error| error.to_string())?;

        tx.execute(
            "delete from project_rule_bindings where rule_asset_id = ?1",
            params![asset_id],
        )
        .map_err(|error| error.to_string())?;
        tx.execute(
            "delete from tool_global_rule_bindings where rule_asset_id = ?1",
            params![asset_id],
        )
        .map_err(|error| error.to_string())?;
        let deleted = tx
            .execute("delete from rule_assets where id = ?1", params![asset_id])
            .map_err(|error| error.to_string())?;
        tx.execute(
            "delete from rule_versions where rule_asset_id = ?1",
            params![asset_id],
        )
        .map_err(|error| error.to_string())?;
        if deleted == 0 {
            return Err(format!("Rule asset {} does not exist.", asset_id));
        }
        tx.commit().map_err(|error| error.to_string())
    }

    pub fn skill_bound_tool_count(&self, asset_id: i32) -> Result<usize, String> {
        let count = self
            .db
            .connection()
            .query_row(
                "select count(distinct tool_id) from tool_skill_bindings where skill_asset_id = ?1",
                params![asset_id],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|error| error.to_string())?;
        Ok(count as usize)
    }

    pub fn delete_skill_asset(&self, asset_id: i32) -> Result<(), String> {
        let tx = self
            .db
            .connection()
            .unchecked_transaction()
            .map_err(|error| error.to_string())?;

        tx.execute(
            "delete from tool_skill_bindings where skill_asset_id = ?1",
            params![asset_id],
        )
        .map_err(|error| error.to_string())?;
        tx.execute(
            "delete from tool_skill_installs where skill_asset_id = ?1",
            params![asset_id],
        )
        .map_err(|error| error.to_string())?;
        let deleted = tx
            .execute("delete from skill_assets where id = ?1", params![asset_id])
            .map_err(|error| error.to_string())?;
        tx.execute(
            "delete from skill_versions where skill_asset_id = ?1",
            params![asset_id],
        )
        .map_err(|error| error.to_string())?;
        if deleted == 0 {
            return Err(format!("Skill asset {} does not exist.", asset_id));
        }
        tx.commit().map_err(|error| error.to_string())
    }

    pub fn rule_impact(&self, asset_id: i32) -> Result<RuleImpactRecord, String> {
        let rule = self.find_latest_rule_version_by_asset(asset_id)?;

        let mut project_stmt = self
            .db
            .connection()
            .prepare(
                r#"
                select distinct projects.name
                from project_rule_bindings
                inner join projects on projects.id = project_rule_bindings.project_id
                where project_rule_bindings.rule_asset_id = ?1
                order by projects.name asc
                "#,
            )
            .map_err(|error| error.to_string())?;
        let project_names = project_stmt
            .query_map(params![asset_id], |row| row.get::<_, String>(0))
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;

        let mut project_tool_stmt = self
            .db
            .connection()
            .prepare(
                r#"
                select distinct project_rule_bindings.tool_id
                from project_rule_bindings
                where project_rule_bindings.rule_asset_id = ?1
                  and project_rule_bindings.tool_id is not null
                order by project_rule_bindings.tool_id asc
                "#,
            )
            .map_err(|error| error.to_string())?;
        let project_tool_ids = project_tool_stmt
            .query_map(params![asset_id], |row| row.get::<_, i32>(0))
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;

        let mut global_tool_stmt = self
            .db
            .connection()
            .prepare(
                r#"
                select distinct tool_global_rule_bindings.tool_id
                from tool_global_rule_bindings
                where tool_global_rule_bindings.rule_asset_id = ?1
                order by tool_global_rule_bindings.tool_id asc
                "#,
            )
            .map_err(|error| error.to_string())?;
        let global_tool_ids = global_tool_stmt
            .query_map(params![asset_id], |row| row.get::<_, i32>(0))
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        let mut tool_ids = project_tool_ids.clone();
        for tool_id in &global_tool_ids {
            if !tool_ids.contains(tool_id) {
                tool_ids.push(*tool_id);
            }
        }
        tool_ids.sort_unstable();

        Ok(RuleImpactRecord {
            rule_asset_id: asset_id,
            rule_name: rule.name,
            project_names,
            tool_ids,
            project_tool_ids,
            global_tool_ids,
        })
    }
}
