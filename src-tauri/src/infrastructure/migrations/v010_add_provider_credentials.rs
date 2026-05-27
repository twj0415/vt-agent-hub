use rusqlite::Connection;

use super::{ensure_column, table_exists};

pub(super) fn apply(conn: &Connection) -> Result<(), String> {
    if table_exists(conn, "provider_tool_configs")? {
        ensure_column(
            conn,
            "provider_tool_configs",
            "credential_ref",
            "text not null default ''",
        )?;
    }
    Ok(())
}

// 每个工具同时只能有一个启用中的供应商配置；
// 先把可能存在的违规历史数据收敛成单个 active,再用 partial unique index 让数据库守住不变量。
