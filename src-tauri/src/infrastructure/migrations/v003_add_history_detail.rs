use rusqlite::Connection;

use super::ensure_column;

pub(super) fn apply(conn: &Connection) -> Result<(), String> {
    ensure_column(conn, "history_logs", "project_id", "integer")?;
    ensure_column(conn, "history_logs", "detail", "text not null default ''")
}
