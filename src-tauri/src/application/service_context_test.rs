#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::application::service_context::ServiceContext;

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

    #[test]
    fn context_opens_injected_database_path() {
        let db_path = unique_db_path("context");
        let context = ServiceContext::at_db(&db_path);

        let db = context.open_db().expect("context db should open");
        db.connection()
            .execute(
                "insert or replace into settings (key, value) values ('context_test', 'ok')",
                [],
            )
            .unwrap();
        drop(db);

        let reopened = context.open_db().expect("context db should reopen");
        let value: String = reopened
            .connection()
            .query_row(
                "select value from settings where key = 'context_test'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert_eq!(context.db_path(), db_path.as_path());
        assert_eq!(value, "ok");
    }
}
