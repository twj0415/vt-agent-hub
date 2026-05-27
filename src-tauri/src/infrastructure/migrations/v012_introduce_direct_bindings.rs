use rusqlite::Connection;

use super::table_exists;

pub(super) fn apply(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        r#"
        create table if not exists project_rule_bindings (
            id integer primary key,
            project_id integer not null,
            tool_id integer,
            rule_asset_id integer not null,
            sort_order integer not null default 0,
            created_at text not null default current_timestamp,
            foreign key (project_id) references projects(id) on delete cascade,
            foreign key (tool_id) references tools(id) on delete cascade,
            foreign key (rule_asset_id) references rule_assets(id) on delete cascade
        );
        create unique index if not exists ux_project_rule_bindings_unique
            on project_rule_bindings(project_id, coalesce(tool_id, -1), rule_asset_id);
        create index if not exists ix_project_rule_bindings_lookup
            on project_rule_bindings(project_id, tool_id);

        create table if not exists tool_global_rule_bindings (
            id integer primary key,
            tool_id integer not null,
            rule_asset_id integer not null,
            sort_order integer not null default 0,
            created_at text not null default current_timestamp,
            unique(tool_id, rule_asset_id),
            foreign key (tool_id) references tools(id) on delete cascade,
            foreign key (rule_asset_id) references rule_assets(id) on delete cascade
        );
        create index if not exists ix_tool_global_rule_bindings_tool
            on tool_global_rule_bindings(tool_id);

        create table if not exists tool_skill_bindings (
            id integer primary key,
            tool_id integer not null,
            skill_asset_id integer not null,
            sort_order integer not null default 0,
            created_at text not null default current_timestamp,
            unique(tool_id, skill_asset_id),
            foreign key (tool_id) references tools(id) on delete cascade,
            foreign key (skill_asset_id) references skill_assets(id) on delete cascade
        );
        create index if not exists ix_tool_skill_bindings_tool
            on tool_skill_bindings(tool_id);
        "#,
    )
    .map_err(|error| error.to_string())?;

    if table_exists(conn, "project_rule_pack_bindings")?
        && table_exists(conn, "pack_version_items")?
    {
        conn.execute(
            r#"
            insert or ignore into project_rule_bindings (project_id, tool_id, rule_asset_id, sort_order)
            select prpb.project_id, prpb.tool_id, pvi.asset_id, pvi.sort_order
            from project_rule_pack_bindings prpb
            inner join pack_version_items pvi on pvi.pack_version_id = prpb.pack_version_id
            inner join projects on projects.id = prpb.project_id
            left join tools on tools.id = prpb.tool_id
            inner join rule_assets on rule_assets.id = pvi.asset_id
            where pvi.item_type = 'rule'
              and (prpb.tool_id is null or tools.id is not null)
            "#,
            [],
        )
        .map_err(|error| error.to_string())?;
    }

    if table_exists(conn, "tool_global_rule_pack_bindings")?
        && table_exists(conn, "pack_version_items")?
    {
        conn.execute(
            r#"
            insert or ignore into tool_global_rule_bindings (tool_id, rule_asset_id, sort_order)
            select tgrpb.tool_id, pvi.asset_id, pvi.sort_order
            from tool_global_rule_pack_bindings tgrpb
            inner join pack_version_items pvi on pvi.pack_version_id = tgrpb.pack_version_id
            inner join tools on tools.id = tgrpb.tool_id
            inner join rule_assets on rule_assets.id = pvi.asset_id
            where pvi.item_type = 'rule'
            "#,
            [],
        )
        .map_err(|error| error.to_string())?;
    }

    if table_exists(conn, "tool_skill_pack_bindings")? && table_exists(conn, "pack_version_items")?
    {
        conn.execute(
            r#"
            insert or ignore into tool_skill_bindings (tool_id, skill_asset_id, sort_order)
            select tspb.tool_id, pvi.asset_id, pvi.sort_order
            from tool_skill_pack_bindings tspb
            inner join pack_version_items pvi on pvi.pack_version_id = tspb.pack_version_id
            inner join tools on tools.id = tspb.tool_id
            inner join skill_assets on skill_assets.id = pvi.asset_id
            where pvi.item_type = 'skill'
            "#,
            [],
        )
        .map_err(|error| error.to_string())?;
    }

    Ok(())
}
