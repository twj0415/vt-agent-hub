use rusqlite::Connection;

pub(super) fn apply(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"
        create table if not exists rule_assets (
            id integer primary key,
            asset_key text not null unique,
            created_at text not null default current_timestamp
        );
        create table if not exists rule_versions (
            id integer primary key,
            rule_asset_id integer not null,
            version_no integer not null,
            code integer not null,
            name text not null,
            category_code integer not null,
            sort_order integer not null default 0,
            state integer not null,
            summary text not null,
            body text not null,
            hash text not null,
            created_at text not null default current_timestamp,
            unique(rule_asset_id, version_no)
        );
        create table if not exists skill_assets (
            id integer primary key,
            asset_key text not null unique,
            created_at text not null default current_timestamp
        );
        create table if not exists skill_versions (
            id integer primary key,
            skill_asset_id integer not null,
            version_no integer not null,
            code integer not null,
            name text not null,
            category_code integer not null,
            state integer not null,
            summary text not null,
            body text not null,
            hash text not null,
            created_at text not null default current_timestamp,
            unique(skill_asset_id, version_no)
        );
        create table if not exists packs (
            id integer primary key,
            pack_key text not null unique,
            name text not null,
            pack_type text not null,
            tool_id integer,
            status text not null default 'ready',
            managed integer not null default 0
        );
        create table if not exists pack_versions (
            id integer primary key,
            pack_id integer not null,
            version_no integer not null,
            change_note text not null default '',
            created_at text not null default current_timestamp,
            unique(pack_id, version_no)
        );
        create table if not exists pack_version_items (
            id integer primary key,
            pack_version_id integer not null,
            item_type text not null,
            asset_id integer not null,
            asset_version_id integer not null,
            sort_order integer not null default 0,
            required integer not null default 1
        );
        create table if not exists project_rule_pack_bindings (
            id integer primary key,
            project_id integer not null,
            tool_id integer,
            pack_id integer not null,
            pack_version_id integer not null,
            enabled integer not null default 1,
            update_policy text not null default 'notify',
            unique(project_id, tool_id)
        );
        create table if not exists tool_global_rule_pack_bindings (
            id integer primary key,
            tool_id integer not null unique,
            pack_id integer not null,
            pack_version_id integer not null,
            enabled integer not null default 1,
            update_policy text not null default 'notify'
        );
        create table if not exists tool_skill_pack_bindings (
            id integer primary key,
            tool_id integer not null unique,
            pack_id integer not null,
            pack_version_id integer not null,
            enabled integer not null default 1,
            update_policy text not null default 'notify'
        );
        create table if not exists tool_skill_installs (
            id integer primary key,
            tool_id integer not null,
            skill_asset_id integer not null,
            required_version_id integer,
            installed_version_id integer,
            state text not null default 'not_installed',
            updated_at text not null default '',
            unique(tool_id, skill_asset_id)
        );
        create table if not exists project_rule_overrides (
            id integer primary key,
            project_id integer not null,
            tool_id integer,
            action text not null,
            rule_asset_id integer not null,
            rule_version_id integer,
            sort_order integer not null default 0
        );
        "#,
    )
    .map_err(|error| error.to_string())?;

    Ok(())
}
