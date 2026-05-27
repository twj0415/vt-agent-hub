use rusqlite::Connection;

use super::ensure_column;

pub(super) fn apply(conn: &Connection) -> Result<(), String> {
    ensure_column(
        conn,
        "rule_versions",
        "sort_order",
        "integer not null default 0",
    )
}
