#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::application::first_run_import_service::FirstRunImportService;
    use crate::application::service_context::ServiceContext;
    use crate::core::tool_registry::{CLAUDE_TOOL_ID, CODEX_TOOL_ID};
    use crate::dto::FirstRunImportApplyInputDto;
    use crate::infrastructure::credential_store::CredentialStore;
    use crate::infrastructure::provider_repo::ProviderRepo;
    use crate::infrastructure::resource_repo::ResourceRepo;

    struct TestEnv {
        root: PathBuf,
        claude_root: PathBuf,
        codex_root: PathBuf,
        context: ServiceContext,
        service: FirstRunImportService,
    }

    fn unique_temp_dir(label: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!("vt-agent-hub-first-run-{label}-{suffix}"))
    }

    fn test_env(label: &str) -> TestEnv {
        let root = unique_temp_dir(label);
        let claude_root = root.join("claude-home");
        let codex_root = root.join("codex-home");
        let context = ServiceContext::at_db(root.join("state").join("app.db"));
        let service = FirstRunImportService::with_context_and_roots(
            context.clone(),
            claude_root.clone(),
            codex_root.clone(),
        )
        .expect("service should initialize");

        TestEnv {
            root,
            claude_root,
            codex_root,
            context,
            service,
        }
    }

    fn apply_selected(service: &FirstRunImportService, selected_ids: Vec<String>, strategy: &str) {
        service
            .apply(FirstRunImportApplyInputDto {
                selected_ids,
                conflict_strategy: Some(strategy.to_string()),
                confirm: true,
            })
            .expect("selected candidates should import");
    }

    #[test]
    fn previews_only_configured_global_roots_and_marks_commands_prompts_unsupported() {
        let env = test_env("preview");
        fs::create_dir_all(env.claude_root.join("commands").join("nested")).unwrap();
        fs::create_dir_all(env.codex_root.join("prompts")).unwrap();
        fs::create_dir_all(env.root.join("project").join(".claude")).unwrap();
        fs::write(
            env.claude_root.join("CLAUDE.md"),
            "---\nname: Claude Global Rule\ndescription: Claude global desc\n---\n\nClaude body.",
        )
        .unwrap();
        fs::write(
            env.codex_root.join("AGENTS.md"),
            "---\nname: Codex Global Rule\ndescription: Codex global desc\n---\n\nCodex body.",
        )
        .unwrap();
        fs::write(
            env.claude_root
                .join("commands")
                .join("nested")
                .join("draft.md"),
            "---\nname: Draft Command\ndescription: Command desc\n---\n\nCommand body.",
        )
        .unwrap();
        fs::write(
            env.codex_root.join("prompts").join("prompt.md"),
            "---\nname: Draft Prompt\ndescription: Prompt desc\n---\n\nPrompt body.",
        )
        .unwrap();
        fs::write(
            env.root.join("project").join(".claude").join("CLAUDE.md"),
            "---\nname: Project Rule\n---\n\nProject body.",
        )
        .unwrap();

        let preview = env.service.preview().expect("preview should work");

        assert_eq!(preview.status, "pending");
        assert!(preview.roots.iter().any(|root| {
            root.tool == "claude" && root.exists && root.candidate_count == 2
        }));
        assert!(preview.roots.iter().any(|root| {
            root.tool == "codex" && root.exists && root.candidate_count == 2
        }));
        assert!(preview
            .candidates
            .iter()
            .any(|candidate| candidate.name == "Claude Global Rule"));
        assert!(preview
            .candidates
            .iter()
            .any(|candidate| candidate.name == "Codex Global Rule"));
        assert!(!preview
            .candidates
            .iter()
            .any(|candidate| candidate.name == "Project Rule"));

        for source_kind in ["claude_command", "codex_prompt"] {
            let candidate = preview
                .candidates
                .iter()
                .find(|candidate| candidate.source_kind == source_kind)
                .expect("unsupported candidate should be visible");
            assert_eq!(candidate.status, "unsupported");
            assert_eq!(candidate.target_asset_type, "none");
            assert_eq!(candidate.recommended_action, "unavailable");
            assert!(!candidate.selectable);
            assert!(!candidate.default_selected);
            assert!(candidate
                .warnings
                .iter()
                .any(|warning| warning.contains("未开发")));
        }
    }

    #[test]
    fn rule_candidate_uses_file_name_and_default_summary_when_metadata_missing() {
        let env = test_env("rule-fallback");
        fs::create_dir_all(&env.codex_root).unwrap();
        fs::write(env.codex_root.join("AGENTS.md"), "Use concise answers.").unwrap();

        let preview = env.service.preview().expect("preview should work");
        let candidate = preview
            .candidates
            .iter()
            .find(|candidate| candidate.source_kind == "global_rule")
            .expect("rule candidate should exist");

        assert_eq!(candidate.name, "AGENTS");
        assert_eq!(candidate.summary, "firstRunImport.descriptions.initialImport");
    }

    #[test]
    fn applies_selected_assets_and_imports_provider_credentials() {
        let env = test_env("apply");
        fs::create_dir_all(env.claude_root.join("skills").join("demo-skill").join("scripts"))
            .unwrap();
        fs::create_dir_all(env.claude_root.join("commands")).unwrap();
        fs::write(
            env.claude_root.join("CLAUDE.md"),
            "---\nname: Claude Rule\ndescription: Rule desc\n---\n\nUse project-safe defaults.",
        )
        .unwrap();
        fs::write(
            env.claude_root
                .join("skills")
                .join("demo-skill")
                .join("SKILL.md"),
            "---\nname: Demo Skill\ndescription: Skill desc\n---\n\n# Demo skill\n\nSkill body.",
        )
        .unwrap();
        fs::write(
            env.claude_root
                .join("skills")
                .join("demo-skill")
                .join("scripts")
                .join("run.py"),
            "print('demo')",
        )
        .unwrap();
        fs::write(
            env.claude_root
                .join("skills")
                .join("demo-skill")
                .join(".env"),
            "SECRET=should-not-copy",
        )
        .unwrap();
        fs::write(
            env.claude_root.join("commands").join("draft.md"),
            "---\nname: Draft Command\n---\n\nCommand body.",
        )
        .unwrap();
        fs::write(
            env.claude_root.join("settings.json"),
            r#"{
  "env": {
    "ANTHROPIC_MODEL": "claude-sonnet-4-6",
    "ANTHROPIC_BASE_URL": "https://proxy.example/v1",
    "ANTHROPIC_API_KEY": "sk-test-secret"
  }
}"#,
        )
        .unwrap();

        let preview = env.service.preview().expect("preview should work");
        let preview_json = serde_json::to_string(&preview).unwrap();
        assert!(preview_json.contains("sk-test-secret"));
        assert!(preview.candidates.iter().any(|candidate| {
            candidate.asset_type == "provider_preset"
                && candidate
                    .metadata
                    .get("credentialDetected")
                    .and_then(|value| value.as_bool())
                    == Some(true)
        }));

        let selected_ids = preview
            .candidates
            .iter()
            .filter(|candidate| {
                matches!(
                    candidate.asset_type.as_str(),
                    "rule" | "skill" | "provider_preset"
                ) || candidate.source_kind == "claude_command"
            })
            .map(|candidate| candidate.id.clone())
            .collect::<Vec<_>>();
        let result = env
            .service
            .apply(FirstRunImportApplyInputDto {
                selected_ids,
                conflict_strategy: Some("rename".to_string()),
                confirm: true,
            })
            .expect("selected candidates should import");

        assert_eq!(result.imported_rules, 1);
        assert_eq!(result.imported_skills, 1);
        assert_eq!(result.imported_providers, 1);
        assert_eq!(result.skipped, 1);
        assert!(result
            .warnings
            .iter()
            .any(|warning| warning.contains("未开发")));
        assert!(!serde_json::to_string(&result)
            .unwrap()
            .contains("sk-test-secret"));

        let db = env.context.open_db().expect("db should reopen");
        let resource = ResourceRepo::new(&db);
        let rule = resource
            .find_latest_rule_version_by_name("Claude Rule")
            .unwrap()
            .expect("rule should exist");
        assert_eq!(rule.summary, "Rule desc");
        assert_eq!(rule.body, "Use project-safe defaults.");
        let rule_binding = resource
            .tool_global_rule_binding(CLAUDE_TOOL_ID)
            .unwrap()
            .expect("imported rule should bind to source tool");
        assert_eq!(rule_binding.items.len(), 1);
        assert_eq!(rule_binding.items[0].asset_id, rule.asset_id);
        let skill = resource
            .find_latest_skill_version_by_name("Demo Skill")
            .unwrap()
            .expect("skill should exist");
        assert_eq!(skill.summary, "Skill desc");
        let skill_binding = resource
            .tool_skill_binding(CLAUDE_TOOL_ID)
            .unwrap()
            .expect("imported skill should bind to source tool");
        assert_eq!(skill_binding.items.len(), 1);
        assert_eq!(skill_binding.items[0].asset_id, skill.asset_id);
        let skill_installs = resource.list_tool_skill_installs(CLAUDE_TOOL_ID).unwrap();
        assert_eq!(skill_installs.len(), 1);
        assert_eq!(skill_installs[0].skill_asset_id, skill.asset_id);
        assert!(env
            .root
            .join("state")
            .join("library")
            .join("skills")
            .join("Demo Skill")
            .join("scripts")
            .join("run.py")
            .is_file());
        assert!(!env
            .root
            .join("state")
            .join("library")
            .join("skills")
            .join("Demo Skill")
            .join(".env")
            .exists());

        let providers = ProviderRepo::new(&db)
            .list(Some(CLAUDE_TOOL_ID))
            .expect("providers should list");
        let provider = providers
            .iter()
            .find(|provider| provider.provider.name == "Claude Compatible")
            .expect("Claude provider should exist");
        let config = provider.configs.first().expect("config should exist");
        assert_eq!(config.model, "claude-sonnet-4-6");
        assert_eq!(config.base_url, "https://proxy.example/v1");
        assert!(config.is_active);
        assert_eq!(config.state, 502);
        let active_count: i32 = db
            .connection()
            .query_row(
                "select count(*) from provider_tool_configs where tool_id = ?1 and is_active = 1",
                [CLAUDE_TOOL_ID],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(active_count, 1);
        assert!(!config.credential_ref.is_empty());
        assert_eq!(
            CredentialStore::load_provider_token(&config.credential_ref).unwrap(),
            Some("sk-test-secret".to_string())
        );
        assert_eq!(
            config
                .config_json
                .get("credentialDetected")
                .and_then(|value| value.as_bool()),
            Some(true)
        );
        assert_eq!(
            config
                .config_json
                .get("credentialSource")
                .and_then(|value| value.as_str()),
            Some("settings.env")
        );
        assert!(!config.config_json.to_string().contains("sk-test-secret"));
        assert!(!serde_json::to_string(&result)
            .unwrap()
            .contains("sk-test-secret"));
    }

    #[test]
    fn duplicated_global_rule_content_is_deduped_across_tools() {
        let env = test_env("rule-dedupe");
        fs::create_dir_all(&env.claude_root).unwrap();
        fs::create_dir_all(&env.codex_root).unwrap();
        fs::write(
            env.claude_root.join("CLAUDE.md"),
            "---\nname: Shared Rule\ndescription: Shared desc\n---\n\nShared body.",
        )
        .unwrap();
        fs::write(
            env.codex_root.join("AGENTS.md"),
            "---\nname: Shared Rule\ndescription: Shared desc\n---\n\nShared body.",
        )
        .unwrap();

        let preview = env.service.preview().expect("preview should work");
        let rule_candidates = preview
            .candidates
            .iter()
            .filter(|candidate| candidate.source_kind == "global_rule")
            .collect::<Vec<_>>();
        assert_eq!(rule_candidates.len(), 1);
        let candidate = rule_candidates[0];
        assert_eq!(candidate.name, "Shared Rule");
        assert!(candidate.source_path.ends_with("CLAUDE.md") || candidate.source_path.ends_with("AGENTS.md"));
        assert_eq!(candidate.metadata.get("sourceToolIds").and_then(|value| value.as_array()).map(|items| items.len()), Some(2));

        apply_selected(&env.service, vec![candidate.id.clone()], "rename");

        let db = env.context.open_db().expect("db should reopen");
        let resource = ResourceRepo::new(&db);
        let rule = resource
            .find_latest_rule_version_by_name("Shared Rule")
            .unwrap()
            .expect("rule should exist");
        let claude_binding = resource
            .tool_global_rule_binding(CLAUDE_TOOL_ID)
            .unwrap()
            .expect("claude should bind");
        let codex_binding = resource
            .tool_global_rule_binding(CODEX_TOOL_ID)
            .unwrap()
            .expect("codex should bind");
        assert_eq!(claude_binding.items.len(), 1);
        assert_eq!(codex_binding.items.len(), 1);
        assert_eq!(claude_binding.items[0].asset_id, rule.asset_id);
        assert_eq!(codex_binding.items[0].asset_id, rule.asset_id);
    }

    #[test]
    fn managed_global_rule_output_is_split_into_rule_candidates() {
        let env = test_env("managed-split");
        fs::create_dir_all(&env.codex_root).unwrap();
        fs::write(
            env.codex_root.join("AGENTS.md"),
            r#"---
name: "Codex"
scope: tool
managedBy: vt-hub-manager
---

## 1. Alpha Rule `v3`

Alpha body.

---

## 2. Beta Rule `v1`

Beta body.
"#,
        )
        .unwrap();

        let preview = env.service.preview().expect("preview should work");
        let rule_candidates = preview
            .candidates
            .iter()
            .filter(|candidate| candidate.source_kind == "global_rule")
            .collect::<Vec<_>>();
        assert_eq!(rule_candidates.len(), 2);
        assert!(rule_candidates.iter().any(|candidate| {
            candidate.name == "Alpha Rule" && candidate.content_preview == "Alpha body."
        }));
        assert!(rule_candidates.iter().any(|candidate| {
            candidate.name == "Beta Rule" && candidate.content_preview == "Beta body."
        }));

        let selected_ids = rule_candidates
            .iter()
            .map(|candidate| candidate.id.clone())
            .collect::<Vec<_>>();
        apply_selected(&env.service, selected_ids, "rename");

        let db = env.context.open_db().expect("db should reopen");
        let resource = ResourceRepo::new(&db);
        let alpha = resource
            .find_latest_rule_version_by_name("Alpha Rule")
            .unwrap()
            .expect("alpha rule should exist");
        let beta = resource
            .find_latest_rule_version_by_name("Beta Rule")
            .unwrap()
            .expect("beta rule should exist");
        let binding = resource
            .tool_global_rule_binding(crate::core::tool_registry::CODEX_TOOL_ID)
            .unwrap()
            .expect("rules should bind to source tool");
        assert_eq!(binding.items.len(), 2);
        assert_eq!(binding.items[0].asset_id, alpha.asset_id);
        assert_eq!(binding.items[1].asset_id, beta.asset_id);
    }

    #[test]
    fn conflict_strategy_rename_keeps_existing_rule_version() {
        let env = test_env("rename");
        fs::create_dir_all(&env.claude_root).unwrap();
        fs::write(
            env.claude_root.join("CLAUDE.md"),
            "---\nname: Shared Rule\ndescription: First\n---\n\nFirst body.",
        )
        .unwrap();

        let preview = env.service.preview().expect("first preview should work");
        let first_id = preview
            .candidates
            .iter()
            .find(|candidate| candidate.source_kind == "global_rule")
            .expect("rule candidate should exist")
            .id
            .clone();
        apply_selected(&env.service, vec![first_id], "rename");

        fs::write(
            env.claude_root.join("CLAUDE.md"),
            "---\nname: Shared Rule\ndescription: Second\n---\n\nSecond body.",
        )
        .unwrap();
        let preview = env.service.preview().expect("conflict preview should work");
        let candidate = preview
            .candidates
            .iter()
            .find(|candidate| candidate.source_kind == "global_rule")
            .expect("rule candidate should exist");
        assert_eq!(candidate.status, "conflict");
        assert_eq!(candidate.recommended_action, "rename");
        assert!(candidate.existing_id.is_some());

        let result = env
            .service
            .apply(FirstRunImportApplyInputDto {
                selected_ids: vec![candidate.id.clone()],
                conflict_strategy: Some("rename".to_string()),
                confirm: true,
            })
            .expect("renamed candidate should import");
        assert_eq!(result.renamed, 1);

        let db = env.context.open_db().expect("db should reopen");
        let resource = ResourceRepo::new(&db);
        let original = resource
            .find_latest_rule_version_by_name("Shared Rule")
            .unwrap()
            .expect("original rule should exist");
        let renamed = resource
            .find_latest_rule_version_by_name("Shared Rule (2)")
            .unwrap()
            .expect("renamed rule should exist");
        assert_eq!(original.body, "First body.");
        assert_eq!(renamed.body, "Second body.");
    }

    #[test]
    fn preview_reports_missing_tool_roots_without_fake_candidates() {
        let env = test_env("missing-roots");

        let preview = env.service.preview().expect("preview should work");

        for root in preview.roots {
            assert!(!root.exists);
            assert_eq!(root.candidate_count, 0);
        }
        assert!(preview.candidates.is_empty());
    }

    #[test]
    fn preview_without_candidates_marks_no_candidates_and_dismiss_disables_prompt() {
        let env = test_env("status");

        let status = env.service.status().expect("status should load");
        assert_eq!(status.status, "pending");
        assert!(status.should_prompt);

        let preview = env.service.preview().expect("empty preview should work");
        assert_eq!(preview.status, "no_candidates");
        let status = env.service.status().expect("status should reload");
        assert_eq!(status.status, "no_candidates");
        assert!(!status.should_prompt);

        let dismissed = env
            .service
            .dismiss("dismissed", Some("later"))
            .expect("dismiss should work");
        assert_eq!(dismissed.status, "dismissed");
        assert!(!dismissed.should_prompt);
        let status = env.service.status().expect("status should reload");
        assert_eq!(status.status, "dismissed");
        assert!(!status.should_prompt);
    }

    #[test]
    fn reset_status_makes_next_launch_prompt_again() {
        let env = test_env("reset-status");
        env.service
            .dismiss("dismissed", Some("later"))
            .expect("dismiss should work");

        let reset = env
            .service
            .reset_status()
            .expect("reset should work");
        assert_eq!(reset.status, "pending");
        assert!(reset.should_prompt);

        let status = env.service.status().expect("status should reload");
        assert_eq!(status.status, "pending");
        assert!(status.should_prompt);
    }
}
