#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use rusqlite::Connection;

    use crate::infrastructure::database::Database;
    use crate::infrastructure::migrations;

    fn unique_db_path(label: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir()
            .join(format!("vt-agent-hub-rebuild-{label}-{suffix}"))
            .join("state")
            .join("app.db")
    }

    fn schema_version(conn: &Connection) -> String {
        conn.query_row(
            "select value from schema_meta where key = 'schema_version'",
            [],
            |row| row.get(0),
        )
        .unwrap()
    }

    fn column_exists(conn: &Connection, table: &str, column: &str) -> bool {
        let sql = format!("pragma table_info({table})");
        let mut stmt = conn.prepare(&sql).unwrap();
        stmt.query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .into_iter()
            .any(|name| name == column)
    }

    fn table_exists(conn: &Connection, table: &str) -> bool {
        conn.query_row(
            "select count(*) from sqlite_master where type = 'table' and name = ?1",
            [table],
            |row| row.get::<_, i32>(0),
        )
        .unwrap()
            > 0
    }

    #[test]
    fn initializes_empty_database_to_latest_schema() {
        let db = Database::open_at(&unique_db_path("migration-empty")).expect("db should open");

        assert_eq!(
            schema_version(db.connection()),
            migrations::latest_schema_version().to_string()
        );
        assert!(column_exists(db.connection(), "projects", "project_type"));
        assert!(column_exists(db.connection(), "history_logs", "project_id"));
        assert!(column_exists(db.connection(), "history_logs", "detail"));
        assert!(column_exists(db.connection(), "history_logs", "action"));
        assert!(column_exists(db.connection(), "history_logs", "tool_id"));
        assert!(column_exists(db.connection(), "history_logs", "result"));
        assert!(column_exists(
            db.connection(),
            "history_logs",
            "result_code"
        ));
        assert!(column_exists(
            db.connection(),
            "history_logs",
            "related_rule_id"
        ));
        assert!(column_exists(
            db.connection(),
            "history_logs",
            "related_path"
        ));
        assert!(column_exists(
            db.connection(),
            "history_logs",
            "navigation_target"
        ));
        assert!(!table_exists(db.connection(), "rules"));
        assert!(column_exists(
            db.connection(),
            "rule_versions",
            "sort_order"
        ));
        assert!(column_exists(
            db.connection(),
            "provider_tool_configs",
            "credential_ref"
        ));
    }

    #[test]
    fn upgrades_legacy_v1_database_without_losing_rows() {
        let path = unique_db_path("migration-legacy");
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(
                r#"
                create table schema_meta (key text primary key, value text not null);
                insert into schema_meta (key, value) values ('schema_version', '1');
                create table tools (id integer primary key, name text not null, enabled integer not null);
                create table projects (id integer primary key, name text not null, path text not null);
                create table presets (
                    id integer primary key,
                    tool_id integer not null,
                    schema_version integer not null,
                    name text not null,
                    provider text not null,
                    model text not null,
                    reasoning text not null,
                    base_url text not null,
                    note text not null,
                    state integer not null
                );
                create table rules (
                    id integer primary key,
                    code integer not null,
                    name text not null,
                    category_code integer not null,
                    state integer not null,
                    summary text not null,
                    body text not null
                );
                insert into rules (id, code, name, category_code, state, summary, body)
                values (3, 301, 'Legacy rule', 301, 502, 'Legacy summary', 'Legacy body');
                create table skills (
                    id integer primary key,
                    code integer not null,
                    name text not null,
                    category_code integer not null,
                    state integer not null,
                    install_state integer not null,
                    summary text not null,
                    body text not null
                );
                create table bindings (
                    id integer primary key,
                    target_type integer not null,
                    target_id integer not null,
                    tool_id integer not null,
                    rule_id integer not null
                );
                create table history_logs (
                    id integer primary key,
                    kind text not null,
                    title text not null,
                    created_at text not null
                );
                create table settings (key text primary key, value text not null);
                insert into projects (id, name, path) values (9, 'Legacy', 'C:\legacy');
                insert into history_logs (id, kind, title, created_at) values (7, 'operation', 'Legacy op', '2026-05-13');
                "#,
            )
            .unwrap();
        }

        let db = Database::open_at(&path).expect("legacy db should upgrade");

        assert_eq!(
            schema_version(db.connection()),
            migrations::latest_schema_version().to_string()
        );
        assert!(column_exists(db.connection(), "projects", "project_type"));
        assert!(column_exists(db.connection(), "history_logs", "detail"));
        assert!(column_exists(db.connection(), "history_logs", "action"));
        assert!(column_exists(db.connection(), "history_logs", "tool_id"));
        assert!(column_exists(db.connection(), "history_logs", "result"));
        assert!(column_exists(
            db.connection(),
            "history_logs",
            "result_code"
        ));
        assert!(column_exists(
            db.connection(),
            "history_logs",
            "related_rule_id"
        ));
        assert!(column_exists(
            db.connection(),
            "history_logs",
            "related_path"
        ));
        assert!(column_exists(
            db.connection(),
            "history_logs",
            "navigation_target"
        ));
        assert!(!table_exists(db.connection(), "rules"));
        let legacy_rule_body: String = db
            .connection()
            .query_row(
                "select body from rule_versions where name = 'Legacy rule'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(legacy_rule_body, "Legacy body");
        let project_name: String = db
            .connection()
            .query_row("select name from projects where id = 9", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(project_name, "Legacy");
    }

    #[test]
    fn repeated_start_keeps_bootstrap_data_minimal() {
        let path = unique_db_path("migration-repeat");

        let first = Database::open_at(&path).expect("first open should work");
        drop(first);
        let second = Database::open_at(&path).expect("second open should work");

        let tool_count: i32 = second
            .connection()
            .query_row("select count(*) from tools where id = 101", [], |row| {
                row.get(0)
            })
            .unwrap();
        let schema_rows: i32 = second
            .connection()
            .query_row(
                "select count(*) from schema_meta where key = 'schema_version'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        let project_count: i32 = second
            .connection()
            .query_row("select count(*) from projects", [], |row| row.get(0))
            .unwrap();
        let preset_count: i32 = second
            .connection()
            .query_row("select count(*) from presets", [], |row| row.get(0))
            .unwrap();

        assert_eq!(tool_count, 1);
        assert_eq!(schema_rows, 1);
        assert_eq!(project_count, 0);
        assert_eq!(preset_count, 0);
    }
}
