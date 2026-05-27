use rusqlite::{params, OptionalExtension};

use super::{PackItemRecord, ResourceRepo, ToolRulePackBindingRecord};

impl<'a> ResourceRepo<'a> {
    pub fn tool_global_rule_binding(
        &self,
        tool_id: i32,
    ) -> Result<Option<ToolRulePackBindingRecord>, String> {
        self.single_tool_direct_binding(
            "tool_global_rule_bindings",
            "rule_asset_id",
            "rule",
            "tool_global_rules",
            tool_id,
        )
    }

    pub fn replace_tool_global_rule_binding_from_rules(
        &self,
        tool_id: i32,
        rule_asset_ids: &[i32],
    ) -> Result<i32, String> {
        self.replace_tool_direct_rule_bindings(
            "tool_global_rule_bindings",
            "rule_asset_id",
            tool_id,
            rule_asset_ids,
        )?;
        Ok(0)
    }

    pub fn tool_skill_binding(
        &self,
        tool_id: i32,
    ) -> Result<Option<ToolRulePackBindingRecord>, String> {
        self.single_tool_direct_binding(
            "tool_skill_bindings",
            "skill_asset_id",
            "skill",
            "tool_skills",
            tool_id,
        )
    }

    pub fn skill_tool_bindings(&self) -> Result<Vec<(i32, i32)>, String> {
        let mut stmt = self
            .db
            .connection()
            .prepare(
                r#"
                select distinct
                    tool_skill_bindings.skill_asset_id,
                    tool_skill_bindings.tool_id
                from tool_skill_bindings
                inner join tools on tools.id = tool_skill_bindings.tool_id and tools.enabled = 1
                order by tool_skill_bindings.skill_asset_id asc, tool_skill_bindings.tool_id asc
                "#,
            )
            .map_err(|error| error.to_string())?;

        let rows = stmt
            .query_map([], |row| Ok((row.get::<_, i32>(0)?, row.get::<_, i32>(1)?)))
            .map_err(|error| error.to_string())?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())
    }

    pub fn replace_tool_skill_binding_from_skills(
        &self,
        tool_id: i32,
        skill_asset_ids: &[i32],
    ) -> Result<i32, String> {
        if skill_asset_ids.is_empty() {
            self.clear_tool_direct_binding("tool_skill_bindings", tool_id)?;
            self.clear_tool_skill_installs(tool_id)?;
            return Ok(0);
        }

        self.replace_tool_direct_skill_bindings(tool_id, skill_asset_ids)?;
        self.sync_required_tool_skill_installs(tool_id)?;
        Ok(0)
    }

    fn single_tool_direct_binding(
        &self,
        table: &str,
        asset_column: &str,
        item_type: &str,
        pack_type: &str,
        tool_id: i32,
    ) -> Result<Option<ToolRulePackBindingRecord>, String> {
        let sql = format!(
            r#"
            select {asset_column}, sort_order
            from {table}
            where tool_id = ?1
            order by sort_order asc, id asc
            "#
        );
        let mut stmt = self
            .db
            .connection()
            .prepare(&sql)
            .map_err(|error| error.to_string())?;
        let rows = stmt
            .query_map(params![tool_id], |row| {
                Ok((row.get::<_, i32>(0)?, row.get::<_, i32>(1)?))
            })
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        drop(stmt);

        if rows.is_empty() {
            return Ok(None);
        }

        let items = rows
            .into_iter()
            .map(|(asset_id, sort_order)| {
                if item_type == "rule" {
                    let rule = self.find_latest_rule_version_by_asset(asset_id)?;
                    Ok(PackItemRecord {
                        item_type: "rule".to_string(),
                        asset_id,
                        asset_version_id: rule.version_id,
                        asset_version_no: rule.version_no,
                        sort_order,
                        required: true,
                    })
                } else {
                    let skill = self.find_latest_skill_version_by_asset(asset_id)?;
                    Ok(PackItemRecord {
                        item_type: "skill".to_string(),
                        asset_id,
                        asset_version_id: skill.version_id,
                        asset_version_no: skill.version_no,
                        sort_order,
                        required: true,
                    })
                }
            })
            .collect::<Result<Vec<_>, String>>()?;

        Ok(Some(ToolRulePackBindingRecord {
            tool_id,
            pack_id: 0,
            pack_name: match item_type {
                "rule" => format!("Tool {tool_id} Global Rules"),
                _ => format!("Tool {tool_id} Skills"),
            },
            pack_type: pack_type.to_string(),
            pack_version_id: 0,
            pack_version_no: 1,
            update_policy: "notify".to_string(),
            enabled: true,
            items,
        }))
    }

    fn replace_tool_direct_rule_bindings(
        &self,
        table: &str,
        asset_column: &str,
        tool_id: i32,
        rule_asset_ids: &[i32],
    ) -> Result<(), String> {
        self.replace_tool_direct_bindings(table, asset_column, tool_id, rule_asset_ids)
    }

    fn replace_tool_direct_skill_bindings(
        &self,
        tool_id: i32,
        skill_asset_ids: &[i32],
    ) -> Result<(), String> {
        self.replace_tool_direct_bindings(
            "tool_skill_bindings",
            "skill_asset_id",
            tool_id,
            skill_asset_ids,
        )
    }

    fn replace_tool_direct_bindings(
        &self,
        table: &str,
        asset_column: &str,
        tool_id: i32,
        asset_ids: &[i32],
    ) -> Result<(), String> {
        let tx = self
            .db
            .connection()
            .unchecked_transaction()
            .map_err(|error| error.to_string())?;

        tx.query_row(
            "select 1 from tools where id = ?1 and enabled = 1",
            params![tool_id],
            |_| Ok(()),
        )
        .optional()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("Tool {} is not available.", tool_id))?;

        let asset_table = match asset_column {
            "rule_asset_id" => "rule_assets",
            "skill_asset_id" => "skill_assets",
            _ => return Err(format!("Unsupported binding asset column: {asset_column}")),
        };
        let exists_sql = format!("select 1 from {asset_table} where id = ?1");
        for asset_id in asset_ids {
            tx.query_row(&exists_sql, params![asset_id], |_| Ok(()))
                .optional()
                .map_err(|error| error.to_string())?
                .ok_or_else(|| match asset_column {
                    "rule_asset_id" => format!("Rule asset {} does not exist.", asset_id),
                    "skill_asset_id" => format!("Skill asset {} does not exist.", asset_id),
                    _ => unreachable!(),
                })?;
        }

        let delete_sql = format!("delete from {table} where tool_id = ?1");
        tx.execute(&delete_sql, params![tool_id])
            .map_err(|error| error.to_string())?;

        let insert_sql = format!(
            "insert or ignore into {table} (tool_id, {asset_column}, sort_order) values (?1, ?2, ?3)"
        );
        for (index, asset_id) in asset_ids.iter().enumerate() {
            tx.execute(&insert_sql, params![tool_id, asset_id, index as i32])
                .map_err(|error| error.to_string())?;
        }

        tx.commit().map_err(|error| error.to_string())
    }

    fn clear_tool_direct_binding(&self, table: &str, tool_id: i32) -> Result<(), String> {
        let tx = self
            .db
            .connection()
            .unchecked_transaction()
            .map_err(|error| error.to_string())?;

        tx.query_row(
            "select 1 from tools where id = ?1 and enabled = 1",
            params![tool_id],
            |_| Ok(()),
        )
        .optional()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("Tool {} is not available.", tool_id))?;

        let delete_sql = format!("delete from {table} where tool_id = ?1");
        tx.execute(&delete_sql, params![tool_id])
            .map_err(|error| error.to_string())?;

        tx.commit().map_err(|error| error.to_string())
    }
}
