use rusqlite::params;
use rusqlite::{Connection, OptionalExtension};

use super::{asset_key, table_exists, unique_rule_asset_key};

pub(super) fn apply(conn: &Connection) -> Result<(), String> {
    if !table_exists(conn, "rules")? {
        return Ok(());
    }

    let mut stmt = conn
        .prepare(
            "select id, code, name, category_code, sort_order, state, summary, body from rules order by id asc",
        )
        .map_err(|error| error.to_string())?;
    let legacy_rules = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i32>(0)?,
                row.get::<_, i32>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i32>(3)?,
                row.get::<_, i32>(4)?,
                row.get::<_, i32>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
            ))
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    drop(stmt);

    for (legacy_id, code, name, category_code, sort_order, state, summary, body) in legacy_rules {
        let asset_key = unique_rule_asset_key(conn, &asset_key(&name), legacy_id)?;
        let asset_id = match conn
            .query_row(
                "select id from rule_assets where asset_key = ?1",
                params![asset_key],
                |row| row.get::<_, i32>(0),
            )
            .optional()
            .map_err(|error| error.to_string())?
        {
            Some(asset_id) => asset_id,
            None => {
                conn.execute(
                    "insert into rule_assets (asset_key) values (?1)",
                    params![asset_key],
                )
                .map_err(|error| error.to_string())?;
                conn.last_insert_rowid() as i32
            }
        };

        let version_exists: i32 = conn
            .query_row(
                "select count(*) from rule_versions where rule_asset_id = ?1",
                params![asset_id],
                |row| row.get(0),
            )
            .map_err(|error| error.to_string())?;
        if version_exists == 0 {
            let hash = format!("legacy-rule-{}-{}", legacy_id, body.len());
            conn.execute(
                "insert into rule_versions (rule_asset_id, version_no, code, name, category_code, sort_order, state, summary, body, hash) values (?1, 1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![asset_id, code, name, category_code, sort_order, state, summary, body, hash],
            )
            .map_err(|error| error.to_string())?;
        }
    }

    conn.execute_batch("drop table if exists rules;")
        .map_err(|error| error.to_string())
}
