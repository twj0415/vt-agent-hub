use rusqlite::Connection;

use super::ensure_column;

pub(super) fn apply(conn: &Connection) -> Result<(), String> {
    ensure_column(
        conn,
        "projects",
        "project_type",
        "integer not null default 203",
    )
}
