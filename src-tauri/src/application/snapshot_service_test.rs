#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::application::service_context::ServiceContext;
    use crate::application::snapshot_service::SnapshotService;

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
    fn settings_snapshot_exposes_paths_and_truth_sources() {
        let db_path = unique_db_path("snapshot-settings");
        let context = ServiceContext::at_db(&db_path);
        let expected_library_root = context.library_root().expect("library root should resolve");
        let service = SnapshotService::with_context(context);

        let snapshot = service
            .get_settings_snapshot()
            .expect("settings snapshot should load");

        assert!(!snapshot.items.is_empty());
        assert!(snapshot.paths.iter().any(|item| item.key == "library_root"
            && item.path == expected_library_root.display().to_string()));
        assert!(snapshot
            .paths
            .iter()
            .any(|item| item.key == "project_output" && item.note.contains("ToolAdapter")));
        assert!(snapshot
            .truth_sources
            .iter()
            .any(|item| item.key == "credentials" && item.canonical == "secure_storage"));
    }

    #[test]
    fn catalog_snapshot_is_empty_until_assets_are_added() {
        let db_path = unique_db_path("snapshot-catalog-skill");
        let context = ServiceContext::at_db(&db_path);
        let service = SnapshotService::with_context(context);

        let snapshot = service
            .get_catalog_snapshot()
            .expect("catalog snapshot should load");

        assert!(snapshot.rules.is_empty());
        assert!(snapshot.skills.is_empty());
    }
}
