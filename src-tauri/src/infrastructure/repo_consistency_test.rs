#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::application::service_context::ServiceContext;
    use crate::application::write_service::WriteService;
    use crate::infrastructure::database::Database;
    use crate::infrastructure::provider_repo::ProviderRepo;
    use crate::infrastructure::resource_repo::ResourceRepo;
    use crate::infrastructure::tool_repo::ToolRepo;

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
    fn project_rule_pack_binding_replaces_cleanly() {
        let db_path = unique_db_path("binding-rollback");
        let context = ServiceContext::at_db(&db_path);
        let writer = WriteService::with_context(context).expect("writer should initialize");
        let project_id = writer
            .save_project(
                None,
                "Binding Rollback Test",
                &std::env::temp_dir().display().to_string(),
                203,
                true,
            )
            .expect("project should save");
        let rule_id = writer
            .save_rule(None, 301, "binding-rule", 301, 502, "Summary", "Rule body")
            .expect("rule should save");
        writer
            .replace_project_rule_bindings(project_id, Some(101), &[rule_id])
            .expect("binding should save");

        let db = Database::open_at(&db_path).expect("db should open");
        let repo = ResourceRepo::new(&db);

        let binding = repo
            .project_rule_bindings(project_id)
            .expect("bindings should load");
        assert!(!binding.is_empty());
    }

    #[test]
    fn disabling_tool_clears_related_bindings_and_provider_config() {
        let db_path = unique_db_path("tool-disable-cleanup");
        let context = ServiceContext::at_db(&db_path);
        let writer = WriteService::with_context(context).expect("writer should initialize");
        let project_id = writer
            .save_project(
                None,
                "Tool Disable Cleanup Test",
                &std::env::temp_dir().display().to_string(),
                203,
                true,
            )
            .expect("project should save");
        let rule_id = writer
            .save_rule(None, 301, "cleanup-rule", 301, 502, "Summary", "Rule body")
            .expect("rule should save");
        let skill_id = writer
            .save_skill(
                None,
                401,
                "cleanup-skill",
                401,
                502,
                601,
                "Summary",
                "Skill body",
            )
            .expect("skill should save");
        writer
            .replace_project_rule_bindings(project_id, Some(101), &[rule_id])
            .expect("project binding should save");
        writer
            .replace_tool_global_rule_bindings(101, &[rule_id])
            .expect("global binding should save");
        writer
            .replace_tool_skill_bindings(101, &[skill_id])
            .expect("skill binding should save");

        let db = Database::open_at(&db_path).expect("db should open");
        db.connection()
            .execute(
                "insert into providers (id, name, category, website, note, sort_order) values (900, 'Cleanup Provider', 'official', '', '', 0)",
                [],
            )
            .expect("provider should insert");
        db.connection()
            .execute(
                "insert into provider_tool_configs (provider_id, tool_id, schema_version, display_name, model, reasoning, base_url, credential_ref, config_json, is_active, state) values (900, 101, 1, 'Cleanup Config', 'gpt-5.5', 'medium', 'https://api.openai.com/v1', 'cleanup-ref', '{}', 1, 502)",
                [],
            )
            .expect("provider config should insert");

        ToolRepo::new(&db)
            .set_enabled(101, false)
            .expect("tool should disable");

        for table in [
            "project_rule_bindings",
            "tool_global_rule_bindings",
            "tool_skill_bindings",
            "tool_skill_installs",
            "provider_tool_configs",
        ] {
            let count: i32 = db
                .connection()
                .query_row(
                    &format!("select count(*) from {table} where tool_id = 101"),
                    [],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(count, 0, "{table} should not keep disabled tool rows");
        }
    }

    #[test]
    fn activating_provider_config_switches_active_row_without_unique_conflict() {
        let db_path = unique_db_path("provider-activate-switch");
        let db = Database::open_at(&db_path).expect("db should open");
        db.connection()
            .execute(
                "insert into providers (id, name, category, website, note, sort_order) values (901, 'Provider A', 'official', '', '', 0), (902, 'Provider B', 'official', '', '', 10)",
                [],
            )
            .expect("providers should insert");
        db.connection()
            .execute(
                "insert into provider_tool_configs (id, provider_id, tool_id, schema_version, display_name, model, reasoning, base_url, credential_ref, config_json, is_active, state) values (901, 901, 101, 1, 'A', 'gpt-5.5', 'medium', 'https://api.openai.com/v1', '', '{}', 1, 502), (902, 902, 101, 1, 'B', 'gpt-5.5', 'high', 'http://43.173.89.135:8080', '', '{}', 0, 504)",
                [],
            )
            .expect("configs should insert");

        ProviderRepo::new(&db)
            .activate_config(101, 902)
            .expect("provider config should activate without unique conflict");

        let active_id: i32 = db
            .connection()
            .query_row(
                "select id from provider_tool_configs where tool_id = 101 and is_active = 1",
                [],
                |row| row.get(0),
            )
            .expect("active config should exist");
        assert_eq!(active_id, 902);
    }

    #[test]
    fn rule_asset_delete_removes_versions() {
        let db_path = unique_db_path("rule-delete");
        let context = ServiceContext::at_db(&db_path);
        let writer = WriteService::with_context(context).expect("writer should initialize");
        let rule_id = writer
            .save_rule(None, 301, "delete-rule", 301, 502, "Summary", "Rule body")
            .expect("rule should save");

        let db = Database::open_at(&db_path).expect("db should open");
        let repo = ResourceRepo::new(&db);

        repo.delete_rule_asset(rule_id).expect("rule should delete");

        assert!(repo.find_latest_rule_version_by_asset(rule_id).is_err());
    }
}
