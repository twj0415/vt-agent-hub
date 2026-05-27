#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::application::library_diagnostics_service::LibraryDiagnosticsService;
    use crate::application::service_context::ServiceContext;
    use crate::application::write_service::WriteService;

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
    fn scans_library_diagnostics_snapshot() {
        let db_path = unique_db_path("library-diagnostics");
        let context = ServiceContext::at_db(&db_path);
        let expected_library_root = context.library_root().expect("library root should resolve");
        let writer = WriteService::with_context(context.clone()).expect("writer should initialize");
        let project_path = std::env::temp_dir().display().to_string();
        writer
            .save_project(None, "Diagnostics Test", &project_path, 203, true)
            .expect("project should save");
        let service =
            LibraryDiagnosticsService::with_context(context).expect("service should initialize");
        let result = service.scan().expect("diagnostics scan should succeed");

        assert_eq!(result.project_count, 1);
        assert_eq!(
            result.library_root,
            expected_library_root.display().to_string()
        );
        assert!(expected_library_root.join("skills").is_dir());
        assert!(!result.created_paths.is_empty());
        assert_eq!(result.issue_count, result.issues.len());
    }

    #[test]
    fn reports_invalid_library_root_without_aborting_scan() {
        let db_path = unique_db_path("library-invalid-root");
        let context = ServiceContext::at_db(&db_path);
        let library_root = context.library_root().expect("library root should resolve");
        fs::create_dir_all(library_root.parent().unwrap()).expect("storage root should create");
        fs::write(&library_root, "not a directory").expect("invalid library root should write");

        let service =
            LibraryDiagnosticsService::with_context(context).expect("service should initialize");
        let result = service.scan().expect("diagnostics scan should not abort");

        assert!(result
            .issues
            .iter()
            .any(|issue| issue.scope == "library" && issue.key == "library_root_invalid"));
    }
}
