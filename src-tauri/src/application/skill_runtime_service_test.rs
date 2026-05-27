#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::application::service_context::ServiceContext;
    use crate::application::skill_runtime_service::SkillRuntimeService;
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

    fn unique_skill_name(label: &str) -> String {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        format!("{label}-{suffix}")
    }

    #[test]
    fn installs_marks_stale_repairs_and_uninstalls_skill_runtime_copy() {
        let db_path = unique_db_path("skill-runtime");
        let context = ServiceContext::at_db(&db_path);
        let writer = WriteService::with_context(context.clone()).expect("writer should initialize");
        let skill_id = writer
            .save_skill(
                None,
                402,
                "skill-runtime-test",
                402,
                502,
                601,
                "Shared UI skill",
                "# Frontend Design\n\nLibrary copy.",
            )
            .expect("skill asset should save");

        let service =
            SkillRuntimeService::with_context(context.clone()).expect("service should initialize");
        let installed = service
            .install_skill(skill_id)
            .expect("install should succeed");
        assert!(installed.runtime_exists);
        assert_eq!(installed.install_state, 602);
        assert!(installed.uninstall_action_ready);
        assert!(installed.mark_stale_action_ready);
        assert!(!installed.install_action_ready);

        let stale = service
            .mark_skill_stale(skill_id)
            .expect("mark stale should succeed");
        assert_eq!(stale.install_state, 603);
        assert!(stale.runtime_body.contains("stale marker"));
        assert!(stale.install_action_ready);
        assert!(stale.repair_action_ready);

        let repaired = service
            .repair_skill(skill_id)
            .expect("repair should succeed");
        assert_eq!(repaired.install_state, 602);
        assert_eq!(repaired.runtime_body, repaired.library_body);

        let uninstalled = service
            .uninstall_skill(skill_id)
            .expect("uninstall should succeed");
        assert!(!uninstalled.runtime_exists);
        assert_eq!(uninstalled.install_state, 601);
    }

    #[test]
    fn reports_conflict_when_runtime_copy_differs_from_library() {
        let db_path = unique_db_path("skill-runtime-conflict");
        let context = ServiceContext::at_db(&db_path);
        let skill_name = unique_skill_name("codex-core-conflict-test");
        let writer = WriteService::with_context(context.clone()).expect("writer should initialize");
        let skill_id = writer
            .save_skill(
                None,
                401,
                &skill_name,
                401,
                502,
                601,
                "Core skill",
                "# Codex Core\n\nLibrary body.",
            )
            .expect("skill asset should save");

        let service =
            SkillRuntimeService::with_context(context.clone()).expect("service should initialize");
        let installed = service
            .install_skill(skill_id)
            .expect("initial install should succeed");
        let runtime_skill_md = PathBuf::from(&installed.runtime_skill_md_path);
        fs::write(&runtime_skill_md, "# Codex Core\n\nRuntime override.")
            .expect("runtime override should write");

        let inspected = service
            .inspect_skill(skill_id)
            .expect("inspect should succeed");
        assert_eq!(inspected.install_state, 605);
        assert!(inspected.status_detail.contains("differs"));
        assert!(!inspected.install_action_ready);
        assert!(inspected.repair_action_ready);
        assert!(inspected.uninstall_action_ready);
        assert!(inspected.platform_root.ends_with(".codex\\skills"));

        let error = service
            .install_skill(skill_id)
            .expect_err("install should refuse to overwrite conflict");
        assert!(error.contains("Use repair"));
    }
}
