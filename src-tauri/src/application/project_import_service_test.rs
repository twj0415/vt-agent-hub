#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::application::project_import_service::ProjectImportService;
    use crate::application::service_context::ServiceContext;

    fn unique_temp_dir(label: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("vt-agent-hub-rebuild-{label}-{suffix}"))
    }

    fn run_git(args: &[&str], cwd: &PathBuf) -> bool {
        Command::new("git")
            .args(args)
            .current_dir(cwd)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    #[test]
    fn rejects_existing_clone_target() {
        let root = unique_temp_dir("project-import-existing");
        let target = root.join("repo");
        fs::create_dir_all(&target).unwrap();
        let context = ServiceContext::at_db(root.join("state").join("app.db"));
        let service =
            ProjectImportService::with_context(context).expect("service should initialize");

        let result = service.import_from_git(
            "https://github.com/example/repo.git",
            target.to_str().unwrap(),
            None,
            None,
            201,
        );

        assert_eq!(result.unwrap_err(), "Target path already exists.");
    }

    #[test]
    fn clones_local_repository_and_imports_project_entity() {
        let root = unique_temp_dir("project-import-git");
        let source = root.join("source");
        fs::create_dir_all(&source).unwrap();
        fs::write(source.join("README.md"), "# Demo\n").unwrap();

        if !run_git(&["init"], &source) {
            return;
        }
        assert!(run_git(&["add", "README.md"], &source));
        assert!(run_git(
            &[
                "-c",
                "user.name=Test",
                "-c",
                "user.email=test@example.com",
                "commit",
                "-m",
                "init"
            ],
            &source
        ));

        let context = ServiceContext::at_db(root.join("state").join("app.db"));
        let service =
            ProjectImportService::with_context(context).expect("service should initialize");
        let target = root.join("nested").join("repo");

        let imported = service
            .import_from_git(
                source.to_str().unwrap(),
                target.to_str().unwrap(),
                None,
                None,
                201,
            )
            .expect("git import should work");

        assert_eq!(imported.name, "repo");
        assert_eq!(imported.path, target.to_string_lossy());
        assert!(target.join("README.md").exists());
    }
}
