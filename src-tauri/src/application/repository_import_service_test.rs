#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::application::repository_import_service::{GitHubRepoRef, RepositoryImportService};
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

    fn github_repo_ref() -> GitHubRepoRef {
        GitHubRepoRef {
            owner: "owner".to_string(),
            repo: "demo-skill".to_string(),
            branch: "main".to_string(),
            normalized_url: "https://github.com/owner/demo-skill".to_string(),
        }
    }

    #[test]
    fn parses_github_git_suffix_urls() {
        let (owner, repo) =
            RepositoryImportService::parse_github_url("https://github.com/anthropics/skills.git")
                .expect("url should parse");

        assert_eq!(owner, "anthropics");
        assert_eq!(repo, "skills");
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

    #[test]
    fn github_snapshot_candidates_follow_skill_md_rules() {
        let root = unique_temp_dir("github-snapshot-candidates");
        let snapshot = root.join("snapshot");
        fs::create_dir_all(snapshot.join("skill-a")).unwrap();
        fs::create_dir_all(snapshot.join("skills").join("nested").join("skill-b")).unwrap();
        fs::create_dir_all(snapshot.join(".github")).unwrap();
        fs::create_dir_all(snapshot.join("skills")).unwrap();
        fs::write(
            snapshot.join("SKILL.md"),
            "---\nname: Root Skill\ndescription: Root desc\n---\n\n# Root",
        )
        .unwrap();
        fs::write(snapshot.join("skill-a").join("SKILL.md"), "# Skill A").unwrap();
        fs::write(
            snapshot
                .join("skills")
                .join("nested")
                .join("skill-b")
                .join("SKILL.md"),
            "# Skill B",
        )
        .unwrap();
        fs::write(snapshot.join(".github").join("SKILL.md"), "# Hidden").unwrap();
        fs::write(snapshot.join("skills").join("SKILL.md"), "# Container").unwrap();

        let candidates = RepositoryImportService::build_github_skill_candidates_from_snapshot(
            &github_repo_ref(),
            &snapshot,
        )
        .expect("candidates should build");
        let source_paths = candidates
            .iter()
            .map(|candidate| candidate.manifest.source_path.as_str())
            .collect::<Vec<_>>();

        assert_eq!(source_paths, vec![".", "skill-a", "skills/nested/skill-b"]);
        assert_eq!(candidates[0].skill_id, "root-skill");
        assert_eq!(candidates[0].description.as_deref(), Some("Root desc"));
    }

    #[test]
    fn github_import_copies_complete_skill_directory() {
        let root = unique_temp_dir("github-import-package");
        let snapshot = root.join("snapshot");
        fs::create_dir_all(snapshot.join("skills").join("demo").join("scripts")).unwrap();
        fs::write(
            snapshot.join("skills").join("demo").join("SKILL.md"),
            "# Demo",
        )
        .unwrap();
        fs::write(
            snapshot
                .join("skills")
                .join("demo")
                .join("scripts")
                .join("run.py"),
            "print('demo')",
        )
        .unwrap();

        let context = ServiceContext::at_db(root.join("state").join("app.db"));
        let service = RepositoryImportService::with_context(context.clone())
            .expect("service should initialize");
        service
            .import_github_snapshot(
                &github_repo_ref(),
                &snapshot,
                vec![crate::dto::GitHubSkillImportSelectionDto {
                    source_path: "skills/demo".to_string(),
                    resolution: "overwrite".to_string(),
                    renamed_skill_id: None,
                }],
            )
            .expect("import should work");

        assert!(root
            .join("state")
            .join("library")
            .join("skills")
            .join("demo")
            .join("scripts")
            .join("run.py")
            .is_file());
    }
}
