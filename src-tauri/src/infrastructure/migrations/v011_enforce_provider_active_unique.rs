use rusqlite::Connection;

use super::table_exists;

pub(super) fn apply(conn: &Connection) -> Result<(), String> {
    if !table_exists(conn, "provider_tool_configs")? {
        return Ok(());
    }

    conn.execute(
        "update provider_tool_configs set is_active = 0 \
         where is_active = 1 and id not in ( \
           select min(id) from provider_tool_configs where is_active = 1 group by tool_id \
         )",
        [],
    )
    .map_err(|error| error.to_string())?;

    conn.execute(
        "create unique index if not exists ux_provider_one_active_per_tool \
         on provider_tool_configs(tool_id) where is_active = 1",
        [],
    )
    .map_err(|error| error.to_string())?;

    Ok(())
}
