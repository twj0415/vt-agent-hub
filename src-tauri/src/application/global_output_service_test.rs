#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::application::global_output_service::GlobalOutputService;
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
    fn previews_codex_global_agents_from_tool_global_bindings() {
        let context = ServiceContext::at_db(unique_db_path("global-output"));
        let writer = WriteService::with_context(context.clone()).expect("writer should initialize");
        let first_rule = writer
            .save_rule(
                None,
                301,
                "global-output-one",
                301,
                502,
                "Summary",
                "Global output one",
            )
            .expect("first global rule should save");
        let second_rule = writer
            .save_rule(
                None,
                302,
                "global-output-two",
                302,
                502,
                "Summary",
                "Global output two",
            )
            .expect("second global rule should save");
        writer
            .replace_tool_global_rule_bindings(101, &[first_rule, second_rule])
            .expect("global rule bindings should save");

        let service =
            GlobalOutputService::with_context(context).expect("service should initialize");
        let preview = service.preview(101).expect("global preview should succeed");

        assert_eq!(preview.tool_id, 101);
        assert_eq!(preview.rule_count, 2);
        assert!(preview.after_content.contains("managedBy: vt-hub-manager"));
        assert!(preview.after_content.contains("scope: tool"));
        assert!(!preview.after_content.contains("## 合并规则清单"));
        assert!(!preview.after_content.contains("global-output-one(v1)"));
        assert!(!preview.after_content.contains("global-output-two(v1)"));
        assert!(preview
            .after_content
            .contains("## 1. global-output-one `v1`"));
        assert!(preview
            .after_content
            .contains("## 2. global-output-two `v1`"));
        assert!(preview
            .after_content
            .contains("---\n\n## 2. global-output-two `v1`"));
        assert!(
            preview.target_path.ends_with(".codex\\AGENTS.md")
                || preview.target_path.ends_with(".codex/AGENTS.md")
        );
    }

    #[test]
    fn previews_claude_global_claude_from_tool_global_bindings() {
        let context = ServiceContext::at_db(unique_db_path("global-output-claude"));
        let writer = WriteService::with_context(context.clone()).expect("writer should initialize");
        let rule_id = writer
            .save_rule(
                None,
                301,
                "claude-global-output",
                301,
                502,
                "Summary",
                "Claude global body",
            )
            .expect("claude global rule should save");
        writer
            .replace_tool_global_rule_bindings(102, &[rule_id])
            .expect("claude global rule binding should save");

        let service =
            GlobalOutputService::with_context(context).expect("service should initialize");
        let preview = service
            .preview(102)
            .expect("claude global preview should succeed");

        assert_eq!(preview.tool_id, 102);
        assert_eq!(preview.rule_count, 1);
        assert!(
            preview.target_path.ends_with(".claude\\CLAUDE.md")
                || preview.target_path.ends_with(".claude/CLAUDE.md")
        );
        assert!(preview.after_content.contains("name: \"Claude\""));
        assert!(preview.after_content.contains("scope: tool"));
        assert!(preview
            .after_content
            .contains("## 1. claude-global-output `v1`"));
        assert!(preview.after_content.contains("Claude global body"));
        assert!(preview
            .warning
            .as_deref()
            .unwrap_or_default()
            .contains("Claude global CLAUDE.md"));
        assert!(preview.diff.contains("generated global CLAUDE.md"));
        assert!(!preview.diff.contains("global AGENTS.md"));
        assert!(!preview
            .warning
            .as_deref()
            .unwrap_or_default()
            .contains("Codex"));
    }

    #[test]
    fn keeps_tool_global_rule_bindings_isolated_by_tool_id() {
        let context = ServiceContext::at_db(unique_db_path("global-output-tool-isolation"));
        let writer = WriteService::with_context(context.clone()).expect("writer should initialize");
        let rule_id = writer
            .save_rule(
                None,
                301,
                "codex-only-global-output",
                301,
                502,
                "Summary",
                "Codex only body",
            )
            .expect("codex global rule should save");
        writer
            .replace_tool_global_rule_bindings(101, &[rule_id])
            .expect("codex global rule binding should save");

        let service =
            GlobalOutputService::with_context(context).expect("service should initialize");
        let result = service.preview(102);

        assert_eq!(
            result.unwrap_err(),
            "Tool global output has no bound rules."
        );
    }

    #[test]
    fn preview_uses_latest_rule_version_after_binding() {
        let context = ServiceContext::at_db(unique_db_path("global-output-latest-rule"));
        let writer = WriteService::with_context(context.clone()).expect("writer should initialize");
        let rule_id = writer
            .save_rule(
                None,
                301,
                "global-latest-rule",
                301,
                502,
                "Summary",
                "Old global body",
            )
            .expect("global rule should save");
        writer
            .replace_tool_global_rule_bindings(101, &[rule_id])
            .expect("global rule binding should save");
        writer
            .save_rule(
                Some(rule_id),
                301,
                "global-latest-rule",
                301,
                502,
                "Summary",
                "New global body",
            )
            .expect("global rule update should save");

        let service =
            GlobalOutputService::with_context(context).expect("service should initialize");
        let preview = service.preview(101).expect("global preview should succeed");

        assert_eq!(preview.rule_count, 1);
        assert!(preview
            .after_content
            .contains("## 1. global-latest-rule `v2`"));
        assert!(preview.after_content.contains("New global body"));
        assert!(!preview.after_content.contains("Old global body"));
    }

    #[test]
    fn rejects_global_preview_without_tool_global_rules() {
        let context = ServiceContext::at_db(unique_db_path("global-output-empty"));
        let service =
            GlobalOutputService::with_context(context).expect("service should initialize");

        let result = service.preview(101);

        assert!(result.is_err());
    }
}
