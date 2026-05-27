use rusqlite::Connection;

pub(super) fn apply(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"
        drop table if exists project_rule_pack_bindings;
        drop table if exists tool_global_rule_pack_bindings;
        drop table if exists tool_skill_pack_bindings;
        drop table if exists pack_version_items;
        drop table if exists pack_versions;
        drop table if exists packs;
        drop table if exists project_rule_overrides;
        "#,
    )
    .map_err(|error| error.to_string())
}
