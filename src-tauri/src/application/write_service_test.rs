#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

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

    fn service(label: &str) -> (WriteService, ServiceContext) {
        let context = ServiceContext::at_db(unique_db_path(label));
        let service =
            WriteService::with_context(context.clone()).expect("service should initialize");
        (service, context)
    }

    #[test]
    fn saves_rule_as_versioned_asset() {
        let (service, _) = service("rule-version");
        let saved = service
            .save_rule(
                None,
                301,
                "rule-version-test",
                301,
                502,
                "Summary",
                "Rule body",
            )
            .expect("rule should save");

        assert!(saved > 0);
        assert_eq!(service.db_count("rule_versions").unwrap(), 1);
    }

    #[test]
    fn imports_rule_description_from_markdown_frontmatter() {
        let (service, _) = service("rule-import-description");
        let source = std::env::temp_dir().join("vt-agent-hub-rule-import-description.md");
        fs::write(
            &source,
            "\u{feff}\n<!-- managed -->\n---\nname: Frontmatter Rule\ndescription: Frontmatter description\ncategory: project\n---\n\n# Rule\n\nBody",
        )
        .expect("rule source should write");

        let imported = service
            .import_rule(
                &source.display().to_string(),
                "Frontmatter Rule",
                301,
                "Frontmatter description",
                "rename",
            )
            .expect("rule should import");

        assert_eq!(imported.rule.name, "Frontmatter Rule");
        assert_eq!(imported.rule.summary, "Frontmatter description");
        assert_eq!(imported.rule.category_code, 301);
        assert_eq!(imported.rule.body, "# Rule\n\nBody");
    }

    #[test]
    fn imports_rule_with_non_ascii_name() {
        let (service, _) = service("rule-import-non-ascii");
        let source_dir = std::env::temp_dir()
            .join("vt-agent-hub-测试数据")
            .join("rules");
        fs::create_dir_all(&source_dir).expect("source dir should write");
        let source = source_dir.join("个人偏好-工具级.md");
        fs::write(
            &source,
            "---\nname: 个人偏好-通用\ndescription: 用户级规则\ncategory: personal\n---\n\nBody",
        )
        .expect("rule source should write");

        let imported = service
            .import_rule(
                &source.display().to_string(),
                "个人偏好-通用",
                301,
                "用户级规则",
                "rename",
            )
            .expect("non-ascii rule should import");

        assert_eq!(imported.rule.name, "个人偏好-通用");

        let second = service
            .import_rule(
                &source.display().to_string(),
                "工具偏好-通用",
                301,
                "用户级规则",
                "rename",
            )
            .expect("second non-ascii rule should import without asset key collision");

        assert_eq!(second.rule.name, "工具偏好-通用");
        assert_eq!(service.db_count("rule_assets").unwrap(), 2);
    }

    #[test]
    fn preview_rule_import_reads_frontmatter_fields_and_body() {
        let (service, _) = service("rule-preview-frontmatter");
        let source = std::env::temp_dir().join("vt-agent-hub-rule-preview-frontmatter.md");
        fs::write(
            &source,
            "---\nname: Preview Rule\ndescription: Preview summary\ncategory: code-quality\n---\n\nBody",
        )
        .expect("rule source should write");

        let preview = service
            .preview_rule_import(&source.display().to_string())
            .expect("rule preview should parse");

        assert_eq!(preview.name, "Preview Rule");
        assert_eq!(preview.summary, "Preview summary");
        assert_eq!(preview.body, "Body");
    }

    #[test]
    fn preview_rule_import_keeps_missing_frontmatter_fields_empty() {
        let (service, _) = service("rule-preview-missing-frontmatter");
        let source = std::env::temp_dir().join("vt-agent-hub-rule-preview-missing-frontmatter.md");
        fs::write(&source, "Body only").expect("rule source should write");

        let preview = service
            .preview_rule_import(&source.display().to_string())
            .expect("rule preview should parse");

        assert_eq!(preview.name, "");
        assert_eq!(preview.summary, "");
        assert_eq!(preview.body, "Body only");
    }

    #[test]
    fn saves_skill_as_versioned_asset() {
        let (service, _) = service("skill-version");
        let saved = service
            .save_skill(
                None,
                402,
                "skill-version-test",
                402,
                502,
                602,
                "Summary",
                "Skill body",
            )
            .expect("skill should save");

        assert!(saved > 0);
    }

    #[test]
    fn binds_project_rules_through_pack_binding() {
        let (service, _) = service("project-pack");
        let project_id = service
            .save_project(
                None,
                "Project Pack Test",
                &std::env::temp_dir().display().to_string(),
                203,
                true,
            )
            .expect("project should save");
        let rule_id = service
            .save_rule(
                None,
                301,
                "project-pack-rule",
                301,
                502,
                "Summary",
                "Rule body",
            )
            .expect("rule should save");
        service
            .replace_project_rule_bindings(project_id, Some(101), &[rule_id])
            .expect("project pack binding should save");

        assert!(!service
            .project_rule_ids(project_id, 101)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn binds_project_common_and_tool_specific_rules_from_concrete_rule_selection() {
        let (service, _) = service("project-rule-selection");
        let project_id = service
            .save_project(
                None,
                "Project Rule Selection Test",
                &std::env::temp_dir().display().to_string(),
                203,
                true,
            )
            .expect("project should save");
        let common_rule_id = service
            .save_rule(
                None,
                301,
                "project-common-rule",
                301,
                502,
                "Summary",
                "Common body",
            )
            .expect("common rule should save");
        let tool_rule_id = service
            .save_rule(
                None,
                304,
                "project-tool-rule",
                304,
                502,
                "Summary",
                "Tool body",
            )
            .expect("tool rule should save");

        service
            .replace_project_rule_bindings(project_id, None, &[common_rule_id])
            .expect("common binding should save");
        service
            .replace_project_rule_bindings(project_id, Some(101), &[tool_rule_id])
            .expect("tool binding should save");

        let ids = service
            .project_rule_ids(project_id, 101)
            .expect("rule ids should resolve");
        assert!(ids.contains(&common_rule_id));
        assert!(ids.contains(&tool_rule_id));
    }

    #[test]
    fn rejects_deleting_bound_rule_asset() {
        let (service, _) = service("bound-rule-delete");
        let project_id = service
            .save_project(
                None,
                "Bound Rule Delete Test",
                &std::env::temp_dir().display().to_string(),
                203,
                true,
            )
            .expect("project should save");
        let rule_id = service
            .save_rule(
                None,
                301,
                "bound-delete-rule",
                301,
                502,
                "Summary",
                "Bound body",
            )
            .expect("rule should save");
        service
            .replace_project_rule_bindings(project_id, None, &[rule_id])
            .expect("project rule binding should save");

        let result = service.delete_rule(rule_id);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unbind it before deleting"));
        assert_eq!(service.db_count("rule_assets").unwrap(), 1);
        assert_eq!(service.db_count("rule_versions").unwrap(), 1);
    }

    #[test]
    fn rejects_binding_missing_rule_asset() {
        let (service, _) = service("missing-rule-binding");
        let project_id = service
            .save_project(
                None,
                "Missing Rule Binding Test",
                &std::env::temp_dir().display().to_string(),
                203,
                true,
            )
            .expect("project should save");

        let result = service.replace_project_rule_bindings(project_id, None, &[999_999]);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Rule asset 999999 does not exist"));
    }

    #[test]
    fn rejects_binding_missing_skill_asset() {
        let (service, _) = service("missing-skill-binding");

        let result = service.replace_tool_skill_bindings(101, &[999_999]);

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Skill asset 999999 does not exist"));
    }

    #[test]
    fn rejects_deleting_bound_skill_asset() {
        let (service, _) = service("bound-skill-delete");
        let skill_id = service
            .save_skill(
                None,
                402,
                "bound-delete-skill",
                402,
                502,
                602,
                "Summary",
                "Skill body",
            )
            .expect("skill should save");
        service
            .replace_tool_skill_bindings(101, &[skill_id])
            .expect("tool skill binding should save");

        let result = service.delete_skill(skill_id);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unbind it before deleting"));
        assert_eq!(service.db_count("skill_assets").unwrap(), 1);
        assert_eq!(service.db_count("skill_versions").unwrap(), 1);
    }

    #[test]
    fn deleting_project_removes_project_rule_bindings() {
        let (service, _) = service("delete-project-bindings");
        let project_id = service
            .save_project(
                None,
                "Delete Project Binding Test",
                &std::env::temp_dir().display().to_string(),
                203,
                true,
            )
            .expect("project should save");
        let rule_id = service
            .save_rule(
                None,
                301,
                "delete-project-bound-rule",
                301,
                502,
                "Summary",
                "Rule body",
            )
            .expect("rule should save");
        service
            .replace_project_rule_bindings(project_id, None, &[rule_id])
            .expect("binding should save");

        service
            .delete_project(project_id)
            .expect("project should delete");

        assert!(service
            .project_rule_ids(project_id, 101)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn binds_tool_global_rules_through_pack_binding() {
        let (service, context) = service("tool-global-pack");
        let rule_id = service
            .save_rule(
                None,
                301,
                "tool-global-rule",
                301,
                502,
                "Summary",
                "Tool global body",
            )
            .expect("rule should save");
        service
            .replace_tool_global_rule_bindings(101, &[rule_id])
            .expect("tool global pack binding should save");

        let db = crate::infrastructure::database::Database::open_at(context.db_path())
            .expect("db should reopen");
        let repo = crate::infrastructure::resource_repo::ResourceRepo::new(&db);
        let binding = repo
            .tool_global_rule_binding(101)
            .expect("global binding should load")
            .expect("global binding should exist");
        assert!(binding.items.iter().any(|item| item.asset_id == rule_id));
    }
}
