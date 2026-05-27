use rusqlite::Connection;

use super::{ensure_column, table_exists};

pub(super) fn apply(conn: &Connection) -> Result<(), String> {
    if table_exists(conn, "rules")? {
        ensure_column(conn, "rules", "sort_order", "integer not null default 0")?;
    }
    Ok(())
}
