use rusqlite::Connection;

use super::ensure_column;

pub(super) fn apply(conn: &Connection) -> Result<(), String> {
    ensure_column(conn, "history_logs", "action", "text not null default ''")?;
    ensure_column(conn, "history_logs", "tool_id", "integer")?;
    ensure_column(
        conn,
        "history_logs",
        "result",
        "text not null default 'success'",
    )?;
    ensure_column(
        conn,
        "history_logs",
        "result_code",
        "integer not null default 701",
    )?;
    ensure_column(conn, "history_logs", "related_rule_id", "integer")?;
    ensure_column(
        conn,
        "history_logs",
        "related_path",
        "text not null default ''",
    )?;
    ensure_column(
        conn,
        "history_logs",
        "navigation_target",
        "text not null default ''",
    )?;
    Ok(())
}
