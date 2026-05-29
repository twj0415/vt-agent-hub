use rusqlite::Connection;

pub(super) fn apply(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"
        update providers set category = 'third_party' where category = 'custom_gateway';
        update providers set category = 'custom' where category = 'local';

        create table provider_tool_configs_new (
            id integer primary key,
            provider_id integer not null,
            tool_id integer not null,
            schema_version integer not null,
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

        insert into provider_tool_configs_new (
            id, provider_id, tool_id, schema_version, model, reasoning, base_url,
            credential_ref, config_json, is_active, state, last_check_status,
            last_check_latency_ms, last_check_message, last_checked_at, created_at, updated_at
        )
        select
            id, provider_id, tool_id, schema_version, model, reasoning, base_url,
            credential_ref, config_json, is_active, state, last_check_status,
            last_check_latency_ms, last_check_message, last_checked_at, created_at, updated_at
        from provider_tool_configs;

        drop table provider_tool_configs;
        alter table provider_tool_configs_new rename to provider_tool_configs;
        "#,
    )
    .map_err(|error| error.to_string())
}
