#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::application::project_workflow_service::ProjectWorkflowService;
    use crate::application::service_context::ServiceContext;
    use crate::application::write_service::WriteService;
    use crate::infrastructure::database::Database;

    fn unique_temp_dir(label: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("vt-agent-hub-rebuild-{label}-{suffix}"))
    }

    #[test]
    fn preview_apply_and_repair_project_agents_flow() {
        let root = unique_temp_dir("workflow");
        let project_dir = root.join("project");
        fs::create_dir_all(&project_dir).unwrap();
        let db_path = root.join("state").join("app.db");
        let context = ServiceContext::at_db(&db_path);
        let writer = WriteService::with_context(context).expect("writer should initialize");
        let project_id = writer
            .save_project(
                None,
                "Workflow Test",
                &project_dir.display().to_string(),
                203,
                true,
            )
            .expect("project should save");
        let rule_id = writer
            .save_rule(
                None,
                301,
                "workflow-rule",
                301,
                502,
                "Summary",
                "Workflow body",
            )
            .expect("rule should save");
        writer
            .replace_project_rule_bindings(project_id, None, &[rule_id])
            .expect("binding should save");

        let db = Database::open_at(&db_path).expect("db should open");
        let service = ProjectWorkflowService::with_db(db);
        let preview = service
            .preview(project_id, 101)
            .expect("preview should succeed");
        assert!(preview.after_content.contains("managedBy: vt-hub-manager"));
        assert!(preview.after_content.contains("scope: project"));
        assert!(!preview.after_content.contains("description:"));
        assert!(!preview.after_content.contains("## 合并规则清单"));
        assert!(!preview.after_content.contains("workflow-rule(v1)"));
        assert!(preview.after_content.contains("## 1. workflow-rule `v1`"));
        assert!(preview.after_content.contains("Workflow body"));
        assert!(!preview.after_content.contains("category: 301"));

        let applied = service
            .apply(project_id, 101, true)
            .expect("apply should write AGENTS.md");
        assert!(applied.target_path.ends_with("AGENTS.md"));

        let unmanaged_path = project_dir.join("AGENTS.md");
        fs::write(&unmanaged_path, "manual rules").unwrap();

        let reopened = Database::open_at(&db_path).expect("db should reopen");
        let repair_service = ProjectWorkflowService::with_db(reopened);
        let repair = repair_service
            .repair(project_id, 101, true)
            .expect("repair should overwrite unmanaged file after confirmation");
        assert_eq!(repair.operation, "project.repair_agents");
    }

    #[test]
    fn preview_uses_latest_rule_version_after_binding() {
        let root = unique_temp_dir("workflow-latest-rule");
        let project_dir = root.join("project");
        fs::create_dir_all(&project_dir).unwrap();
        let db_path = root.join("state").join("app.db");
        let context = ServiceContext::at_db(&db_path);
        let writer = WriteService::with_context(context).expect("writer should initialize");
        let project_id = writer
            .save_project(
                None,
                "Workflow Latest Rule Test",
                &project_dir.display().to_string(),
                203,
                true,
            )
            .expect("project should save");
        let rule_id = writer
            .save_rule(
                None,
                301,
                "workflow-latest-rule",
                301,
                502,
                "Summary",
                "Old body",
            )
            .expect("rule should save");
        writer
            .replace_project_rule_bindings(project_id, None, &[rule_id])
            .expect("binding should save");
        writer
            .save_rule(
                Some(rule_id),
                301,
                "workflow-latest-rule",
                301,
                502,
                "Summary",
                "New body",
            )
            .expect("rule update should save");

        let db = Database::open_at(&db_path).expect("db should open");
        let service = ProjectWorkflowService::with_db(db);
        let preview = service
            .preview(project_id, 101)
            .expect("preview should succeed");

        assert_eq!(preview.rule_count, 1);
        assert!(preview
            .after_content
            .contains("## 1. workflow-latest-rule `v2`"));
        assert!(preview.after_content.contains("New body"));
        assert!(!preview.after_content.contains("Old body"));
    }

    #[test]
    fn apply_allows_syncing_empty_rules_to_managed_project_agents() {
        let root = unique_temp_dir("empty-rules-sync");
        let project_dir = root.join("project");
        fs::create_dir_all(&project_dir).unwrap();
        let db_path = root.join("state").join("app.db");
        let context = ServiceContext::at_db(&db_path);
        let writer = WriteService::with_context(context).expect("writer should initialize");
        let project_id = writer
            .save_project(
                None,
                "Empty Rules Sync Test",
                &project_dir.display().to_string(),
                203,
                true,
            )
            .expect("project should save");
        let rule_id = writer
            .save_rule(
                None,
                301,
                "empty-sync-rule",
                301,
                502,
                "Summary",
                "Remove me",
            )
            .expect("rule should save");
        writer
            .replace_project_rule_bindings(project_id, None, &[rule_id])
            .expect("binding should save");

        let db = Database::open_at(&db_path).expect("db should open");
        let service = ProjectWorkflowService::with_db(db);
        let applied = service
            .apply(project_id, 101, true)
            .expect("first apply should write managed AGENTS.md");
        let target_path = PathBuf::from(applied.target_path.clone());
        assert!(fs::read_to_string(&target_path)
            .unwrap()
            .contains("Remove me"));

        writer
            .replace_project_rule_bindings(project_id, None, &[])
            .expect("binding should clear");
        let preview = service
            .preview(project_id, 101)
            .expect("preview should allow syncing empty bindings for managed output");
        assert_eq!(preview.rule_count, 0);
        assert!(preview.can_apply);

        service
            .apply(project_id, 101, true)
            .expect("apply should sync empty bindings to managed AGENTS.md");
        let synced = fs::read_to_string(&target_path).unwrap();
        assert!(!synced.contains("Remove me"));
    }

    #[test]
    fn cleanup_and_reset_only_allow_managed_project_agents() {
        let root = unique_temp_dir("cleanup-reset");
        let project_dir = root.join("project");
        fs::create_dir_all(&project_dir).unwrap();
        let db_path = root.join("state").join("app.db");
        let context = ServiceContext::at_db(&db_path);
        let writer = WriteService::with_context(context).expect("writer should initialize");
        let project_id = writer
            .save_project(
                None,
                "Cleanup Reset Test",
                &project_dir.display().to_string(),
                203,
                true,
            )
            .expect("project should save");
        let rule_id = writer
            .save_rule(
                None,
                301,
                "cleanup-rule",
                301,
                502,
                "Summary",
                "Cleanup body",
            )
            .expect("rule should save");
        writer
            .replace_project_rule_bindings(project_id, None, &[rule_id])
            .expect("binding should save");

        let db = Database::open_at(&db_path).expect("db should open");
        let service = ProjectWorkflowService::with_db(db);
        let applied = service
            .apply(project_id, 101, true)
            .expect("apply should write managed AGENTS.md");
        let target_path = PathBuf::from(applied.target_path.clone());
        assert!(target_path.exists());

        let cleanup = service
            .cleanup(project_id, 101, true)
            .expect("cleanup should delete managed file");
        assert_eq!(cleanup.operation, "project.cleanup_agents");
        assert!(!PathBuf::from(&cleanup.target_path).exists());
        assert_eq!(cleanup.managed, false);
        assert!(cleanup.backup_path.is_some());

        let reset_missing = service.reset(project_id, 101, true);
        assert!(reset_missing.is_err());

        fs::write(&target_path, "manual rules").unwrap();
        let cleanup_unmanaged = service.cleanup(project_id, 101, true);
        assert!(cleanup_unmanaged.is_err());

        let repaired = service
            .repair(project_id, 101, true)
            .expect("repair should restore managed AGENTS.md");
        assert!(PathBuf::from(repaired.target_path.clone()).exists());

        let reset = service
            .reset(project_id, 101, true)
            .expect("reset should delete managed file");
        assert_eq!(reset.operation, "project.reset_agents");
        assert!(!PathBuf::from(&reset.target_path).exists());
        assert_eq!(reset.message, "Project AGENTS.md reset to unmanaged state.");
    }

    #[test]
    fn scan_reports_missing_project_directory() {
        let root = unique_temp_dir("missing-project-path");
        let project_dir = root.join("project");
        fs::create_dir_all(&project_dir).unwrap();
        let db_path = root.join("state").join("app.db");
        let context = ServiceContext::at_db(&db_path);
        let writer = WriteService::with_context(context).expect("writer should initialize");
        let project_id = writer
            .save_project(
                None,
                "Missing Path Test",
                &project_dir.display().to_string(),
                203,
                true,
            )
            .expect("project should save");

        fs::remove_dir_all(&project_dir).unwrap();

        let db = Database::open_at(&db_path).expect("db should open");
        let service = ProjectWorkflowService::with_db(db);
        let scan = service.scan(project_id, 101).expect("scan should complete");

        assert_eq!(scan.status, "missing");
        assert!(scan.issues.contains(&"project_path_missing".to_string()));
    }
}
