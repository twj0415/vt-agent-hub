#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::application::repository_import_service::RepositoryImportService;
    use crate::application::service_context::ServiceContext;
    use crate::infrastructure::resource_repo::ResourceRepo;

    fn unique_temp_dir(label: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("vt-agent-hub-rebuild-{label}-{suffix}"))
    }

    fn create_repo(root: &PathBuf) {
        fs::create_dir_all(root.join("rules").join("git")).unwrap();
        fs::create_dir_all(root.join("skills").join("demo-skill")).unwrap();
        fs::create_dir_all(root.join("presets")).unwrap();
        fs::write(
            root.join("rules").join("git").join("repo-rule.md"),
            "---\nname: Repository Rule\ndescription: Repository rule description\ncategory: personal\n---\n\nRepository rule.",
        )
        .unwrap();
        fs::write(
            root.join("skills").join("demo-skill").join("SKILL.md"),
            "# Demo skill\n\nRepository skill.",
        )
        .unwrap();
        fs::write(root.join("presets").join("demo.json"), "{}").unwrap();
    }

    #[test]
    fn previews_and_applies_local_repository_import() {
        let root = unique_temp_dir("repository-import");
        let repo_root = root.join("repo");
        create_repo(&repo_root);
        let context = ServiceContext::at_db(root.join("state").join("app.db"));
        let service = RepositoryImportService::with_context(context.clone())
            .expect("service should initialize");

        let preview = service
            .preview_repository(repo_root.to_str().unwrap(), "main", "rename")
            .expect("preview should work");
        assert!(preview.preview_only);
        assert_eq!(preview.imported_rules, 1);
        assert_eq!(preview.imported_skills, 1);
        assert_eq!(preview.detected_presets, 1);

        let applied = service
            .apply_repository(repo_root.to_str().unwrap(), "main", "rename")
            .expect("apply should work");
        assert!(!applied.preview_only);

        let db = context.open_db().expect("db should reopen");
        let resource = ResourceRepo::new(&db);
        let rule = resource
            .find_latest_rule_version_by_name("Repository Rule")
            .unwrap()
            .expect("rule should exist");
        assert_eq!(rule.summary, "Repository rule description");
        assert_eq!(rule.category_code, 301);
        assert_eq!(rule.body, "Repository rule.");
        assert!(resource
            .find_latest_skill_version_by_name("demo-skill")
            .unwrap()
            .is_some());
    }

    #[test]
    fn reports_conflicts_with_skip_strategy() {
        let root = unique_temp_dir("repository-import-conflict");
        let repo_root = root.join("repo");
        create_repo(&repo_root);
        let context = ServiceContext::at_db(root.join("state").join("app.db"));
        let service =
            RepositoryImportService::with_context(context).expect("service should initialize");

        service
            .apply_repository(repo_root.to_str().unwrap(), "main", "rename")
            .expect("first import should work");
        let preview = service
            .preview_repository(repo_root.to_str().unwrap(), "main", "skip")
            .expect("conflict preview should work");

        assert!(preview.skipped >= 2);
        assert!(preview.assets.iter().any(|asset| asset.conflict));
    }
}
