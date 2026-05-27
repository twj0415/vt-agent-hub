use rusqlite::Connection;

pub(super) fn apply(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"
        create table if not exists tools (
            id integer primary key,
            name text not null,
            enabled integer not null
        );
        create table if not exists projects (
            id integer primary key,
            name text not null,
            path text not null
        );
        create table if not exists presets (
            id integer primary key,
            tool_id integer not null,
            schema_version integer not null,
            name text not null,
            provider text not null,
            model text not null,
            reasoning text not null,
            base_url text not null,
            note text not null,
            state integer not null
        );
        create table if not exists skills (
            id integer primary key,
            code integer not null,
            name text not null,
            category_code integer not null,
            state integer not null,
            install_state integer not null,
            summary text not null,
            body text not null
        );
        create table if not exists bindings (
            id integer primary key,
            target_type integer not null,
            target_id integer not null,
            tool_id integer not null,
            rule_id integer not null
        );
        create table if not exists history_logs (
            id integer primary key,
            kind text not null,
            title text not null,
            created_at text not null
        );
        create table if not exists settings (
            key text primary key,
            value text not null
        );
        "#,
    )
    .map_err(|error| error.to_string())?;
    Ok(())
}
