use rusqlite::params;
use rusqlite::{Connection, OptionalExtension};

pub(super) fn apply(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"
        create table if not exists providers (
            id integer primary key,
            name text not null,
            category text not null,
            website text not null default '',
            note text not null default '',
            sort_order integer not null default 0,
            created_at text not null default current_timestamp,
            updated_at text not null default current_timestamp
        );
        create table if not exists provider_tool_configs (
            id integer primary key,
            provider_id integer not null,
            tool_id integer not null,
            schema_version integer not null,
            display_name text not null,
            model text not null,
            reasoning text not null,
            base_url text not null,
            credential_ref text not null default '',
            config_json text not null default '{}',
            is_active integer not null default 0,
            state integer not null default 504,
            last_check_status text not null default 'unchecked',
            last_check_latency_ms integer,
            last_check_message text not null default '',
            last_checked_at text not null default '',
            created_at text not null default current_timestamp,
            updated_at text not null default current_timestamp,
            unique(provider_id, tool_id)
        );
        "#,
    )
    .map_err(|error| error.to_string())?;

    let mut stmt = conn
        .prepare("select id, tool_id, schema_version, name, provider, model, reasoning, base_url, note, state from presets order by id asc")
        .map_err(|error| error.to_string())?;
    let presets = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i32>(0)?,
                row.get::<_, i32>(1)?,
                row.get::<_, i32>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
                row.get::<_, String>(7)?,
                row.get::<_, String>(8)?,
                row.get::<_, i32>(9)?,
            ))
        })
        .map_err(|error| error.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;
    drop(stmt);

    for (
        preset_id,
        tool_id,
        schema_version,
        name,
        provider,
        model,
        reasoning,
        base_url,
        note,
        state,
    ) in presets
    {
        let provider_id = conn
            .query_row(
                "select id from providers where name = ?1",
                params![provider],
                |row| row.get::<_, i32>(0),
            )
            .optional()
            .map_err(|error| error.to_string())?;
        let provider_id = match provider_id {
            Some(provider_id) => provider_id,
            None => {
                conn.execute(
                    "insert into providers (name, category, note, sort_order) values (?1, 'official', ?2, ?3)",
                    params![provider, note, preset_id * 10],
                )
                .map_err(|error| error.to_string())?;
                conn.last_insert_rowid() as i32
            }
        };

        conn.execute(
            "insert or ignore into provider_tool_configs (provider_id, tool_id, schema_version, display_name, model, reasoning, base_url, credential_ref, config_json, is_active, state) values (?1, ?2, ?3, ?4, ?5, ?6, ?7, '', '{}', ?8, ?9)",
            params![provider_id, tool_id, schema_version, name, model, reasoning, base_url, if state == 502 { 1 } else { 0 }, state],
        )
        .map_err(|error| error.to_string())?;
    }

    Ok(())
}
