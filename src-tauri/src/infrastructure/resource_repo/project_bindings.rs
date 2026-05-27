use std::collections::BTreeMap;

use rusqlite::{params, OptionalExtension};

use super::{PackItemRecord, ProjectRulePackBindingRecord, ResourceRepo};

impl<'a> ResourceRepo<'a> {
    pub fn project_rule_bindings(
        &self,
        project_id: i32,
    ) -> Result<Vec<ProjectRulePackBindingRecord>, String> {
        let mut stmt = self
            .db
            .connection()
            .prepare(
                r#"
                select
                    project_id,
                    tool_id,
                    rule_asset_id,
                    sort_order
                from project_rule_bindings
                where project_id = ?1
                order by tool_id asc, sort_order asc, id asc
                "#,
            )
            .map_err(|error| error.to_string())?;

        let rows = stmt
            .query_map(params![project_id], |row| {
                Ok((
                    row.get::<_, i32>(0)?,
                    row.get::<_, Option<i32>>(1)?,
                    row.get::<_, i32>(2)?,
                    row.get::<_, i32>(3)?,
                ))
            })
            .map_err(|error| error.to_string())?;

        let rows = rows
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        drop(stmt);

        let mut grouped = BTreeMap::<Option<i32>, Vec<PackItemRecord>>::new();
        for (_, tool_id, asset_id, sort_order) in rows {
            let rule = self.find_latest_rule_version_by_asset(asset_id)?;
            grouped.entry(tool_id).or_default().push(PackItemRecord {
                item_type: "rule".to_string(),
                asset_id,
                asset_version_id: rule.version_id,
                asset_version_no: rule.version_no,
                sort_order,
                required: true,
            });
        }

        Ok(grouped
            .into_iter()
            .map(|(tool_id, items)| ProjectRulePackBindingRecord {
                project_id,
                tool_id,
                pack_id: 0,
                pack_name: match tool_id {
                    Some(tool_id) => format!("Project {project_id} Tool {tool_id} Rules"),
                    None => format!("Project {project_id} Common Rules"),
                },
                pack_type: "project_rules".to_string(),
                pack_version_id: 0,
                pack_version_no: 1,
                update_policy: "notify".to_string(),
                enabled: true,
                items,
            })
            .collect())
    }

    pub fn clear_project_rule_binding(
        &self,
        project_id: i32,
        tool_id: Option<i32>,
    ) -> Result<(), String> {
        let tx = self
            .db
            .connection()
            .unchecked_transaction()
            .map_err(|error| error.to_string())?;

        tx.query_row(
            "select 1 from projects where id = ?1",
            params![project_id],
            |_| Ok(()),
        )
        .optional()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("Project {} does not exist.", project_id))?;

        tx.execute(
            "delete from project_rule_bindings where project_id = ?1 and tool_id is ?2",
            params![project_id, tool_id],
        )
        .map_err(|error| error.to_string())?;

        tx.commit().map_err(|error| error.to_string())
    }

    pub fn replace_project_rule_binding_from_rules(
        &self,
        project_id: i32,
        tool_id: Option<i32>,
        rule_asset_ids: &[i32],
    ) -> Result<i32, String> {
        if rule_asset_ids.is_empty() {
            self.clear_project_rule_binding(project_id, tool_id)?;
            return Ok(0);
        }

        let tx = self
            .db
            .connection()
            .unchecked_transaction()
            .map_err(|error| error.to_string())?;

        for asset_id in rule_asset_ids {
            tx.query_row(
                "select 1 from rule_assets where id = ?1",
                params![asset_id],
                |_| Ok(()),
            )
            .optional()
            .map_err(|error| error.to_string())?
            .ok_or_else(|| format!("Rule asset {} does not exist.", asset_id))?;
        }

        let latest_versions = rule_asset_ids
            .iter()
            .enumerate()
            .map(|(index, asset_id)| Ok((index as i32, *asset_id)))
            .collect::<Result<Vec<_>, String>>()?;

        tx.query_row(
            "select 1 from projects where id = ?1",
            params![project_id],
            |_| Ok(()),
        )
        .optional()
        .map_err(|error| error.to_string())?
        .ok_or_else(|| format!("Project {} does not exist.", project_id))?;

        if let Some(tool_id) = tool_id {
            tx.query_row(
                "select 1 from tools where id = ?1 and enabled = 1",
                params![tool_id],
                |_| Ok(()),
            )
            .optional()
            .map_err(|error| error.to_string())?
            .ok_or_else(|| format!("Tool {} is not available.", tool_id))?;
        }

        tx.execute(
            "delete from project_rule_bindings where project_id = ?1 and tool_id is ?2",
            params![project_id, tool_id],
        )
        .map_err(|error| error.to_string())?;

        for (sort_order, asset_id) in latest_versions {
            tx.execute(
                "insert or ignore into project_rule_bindings (project_id, tool_id, rule_asset_id, sort_order) values (?1, ?2, ?3, ?4)",
                params![project_id, tool_id, asset_id, sort_order],
            )
            .map_err(|error| error.to_string())?;
        }

        tx.commit().map_err(|error| error.to_string())?;
        Ok(0)
    }
}
