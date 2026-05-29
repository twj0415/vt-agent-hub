#[cfg(test)]
mod tests {
    use crate::adapters::codex::CodexAdapter;
    use crate::adapters::tool_adapter::{
        ProjectOutputBuildInput, ProjectOutputRule, ProjectOutputScope, ToolAdapter,
    };
    use crate::dto::{ProviderImportInputDto, ProviderImportInputPartDto};
    use crate::application::tool_service::ToolService;
    use crate::core::status_codes::{
        SKILL_INSTALL_INSTALLED, SKILL_INSTALL_NOT_INSTALLED, SKILL_INSTALL_SOURCE_MISSING,
        SKILL_INSTALL_STALE, TARGET_STATE_ERROR, TARGET_STATE_MISSING, TARGET_STATE_PLANNED,
        TARGET_STATE_READY,
    };
    use crate::core::tool_registry::{CLAUDE_TOOL_ID, CODEX_TOOL_ID, CURSOR_TOOL_ID};

    #[test]
    fn exposes_codex_diagnostics_through_adapter_boundary() {
        let service = ToolService::new();
        let diagnostics = service
            .get_diagnostics(CODEX_TOOL_ID)
            .expect("codex diagnostics should be available");

        assert!(diagnostics.installation_detected);
        assert!(!diagnostics.credential_state.is_empty());
        assert!(!diagnostics.skill_state.is_empty());
        assert!(!diagnostics.project_output_state.is_empty());
        assert!(!diagnostics.repair_state.is_empty());
    }

    #[test]
    fn exposes_diagnostics_for_enabled_tool_slots() {
        let service = ToolService::new();

        assert!(service.get_diagnostics(CODEX_TOOL_ID).is_ok());
        assert!(service.get_diagnostics(CLAUDE_TOOL_ID).is_ok());
        assert!(service.get_diagnostics(CURSOR_TOOL_ID).is_err());
        assert!(service.get_diagnostics(999).is_err());
    }

    #[test]
    fn verifies_credentials_through_adapter_boundary() {
        let service = ToolService::new();
        let result = service
            .verify_credential(CODEX_TOOL_ID, "demo-token")
            .expect("credential verification should route through the adapter");

        assert_eq!(result.state, "local_valid_remote_unavailable");
        assert!(!result.ok);
        assert!(!result.manual_steps.is_empty());
    }

    #[test]
    fn reports_missing_credentials() {
        let service = ToolService::new();
        let result = service
            .verify_credential(CODEX_TOOL_ID, "")
            .expect("credential verification should return a local result");

        assert_eq!(result.state, "local_invalid");
        assert!(!result.ok);
    }

    #[test]
    fn codex_adapter_contract_returns_stable_tool_identity() {
        let adapter = CodexAdapter;
        let tool = adapter.tool();

        assert_eq!(tool.id, CODEX_TOOL_ID);
        assert_eq!(tool.key, "codex");
        assert!(tool.enabled);
    }

    #[test]
    fn tool_capabilities_are_exposed_for_all_supported_slots() {
        let service = ToolService::new();
        let capabilities = service
            .capabilities(CODEX_TOOL_ID)
            .expect("codex should expose registry metadata");

        assert!(capabilities.rules);
        assert!(capabilities.skill_install);
        assert!(capabilities.agents_output);
        assert!(service.project_output_managed_marker(CODEX_TOOL_ID).is_ok());
        assert!(service
            .project_output_managed_marker(CLAUDE_TOOL_ID)
            .is_ok());
        assert!(service
            .project_output_managed_marker(CURSOR_TOOL_ID)
            .is_err());
        assert!(service.project_output_managed_marker(999).is_err());
    }

    #[test]
    fn project_output_path_and_render_are_adapter_owned() {
        let service = ToolService::new();
        let target_path = service
            .project_output_target_path(CODEX_TOOL_ID, "C:\\demo")
            .expect("codex should provide output target path");
        let marker = service
            .project_output_managed_marker(CODEX_TOOL_ID)
            .expect("codex should provide managed marker");
        let output = service
            .render_project_output(
                CODEX_TOOL_ID,
                &ProjectOutputBuildInput {
                    project_name: "Demo".to_string(),
                    scope: ProjectOutputScope::Project,
                    rules: vec![ProjectOutputRule {
                        id: 1,
                        version_no: 2,
                        code: 301,
                        category_code: 301,
                        sort_order: 0,
                        name: "Personal".to_string(),
                        body: "Rule body".to_string(),
                    }],
                },
            )
            .expect("codex should render project output");

        assert!(target_path.ends_with("AGENTS.md"));
        assert!(output.contains(marker));
        assert!(output.contains("scope: project"));
        assert!(!output.contains("description:"));
        assert!(!output.contains("## 合并规则清单"));
        assert!(!output.contains("Personal(v2)"));
        assert!(output.contains("## 1. Personal `v2`"));
        assert!(output.contains("Rule body"));
    }

    #[test]
    fn runtime_targets_are_adapter_owned() {
        let service = ToolService::new();

        assert!(service
            .global_output_target_path(CODEX_TOOL_ID)
            .expect("codex global target should resolve")
            .ends_with("AGENTS.md"));
        assert!(service
            .skill_runtime_root(CODEX_TOOL_ID)
            .expect("codex skill root should resolve")
            .ends_with("skills"));
        assert!(service
            .preset_config_path(CODEX_TOOL_ID)
            .expect("codex preset config should resolve")
            .ends_with("config.toml"));
    }

    #[test]
    fn imports_provider_configs_through_adapter_boundary() {
        let service = ToolService::new();
        let codex = service
            .import_provider_config(
                CODEX_TOOL_ID,
                &ProviderImportInputDto {
                    tool_id: CODEX_TOOL_ID,
                    parts: vec![
                        ProviderImportInputPartDto {
                            role: "config".to_string(),
                            content: r#"
model_provider = "OpenAI"
model = "gpt-5.5"
review_model = "gpt-5.4"
model_reasoning_effort = "medium"
base_url = "https://api.openai.com/v1"
"#
                                .to_string(),
                        },
                        ProviderImportInputPartDto {
                            role: "auth".to_string(),
                            content: r#"{ "OPENAI_API_KEY": "codex-token" }"#.to_string(),
                        },
                    ],
                },
            )
            .expect("codex import should succeed");

        assert_eq!(codex.provider_name, "OpenAI");
        assert_eq!(codex.category, "official");
        assert_eq!(codex.display_name, "OpenAI");
        assert_eq!(codex.credential_token.as_deref(), Some("codex-token"));
        assert_eq!(codex.config_json.get("wireApi").and_then(serde_json::Value::as_str), Some("responses"));

        let claude = service
            .import_provider_config(
                CLAUDE_TOOL_ID,
                &ProviderImportInputDto {
                    tool_id: CLAUDE_TOOL_ID,
                    parts: vec![ProviderImportInputPartDto {
                        role: "config".to_string(),
                        content: r#"
{
  "env": {
    "CLAUDE_CODE_USE_BEDROCK": "true",
    "AWS_REGION": "eu-west-1",
    "ANTHROPIC_API_KEY": "claude-token"
  }
}
"#
                            .to_string(),
                    }],
                },
            )
            .expect("claude import should succeed");

        assert_eq!(claude.provider_name, "Claude Bedrock");
        assert_eq!(claude.category, "custom_gateway");
        assert_eq!(claude.base_url, "bedrock://eu-west-1");
        assert_eq!(claude.credential_token.as_deref(), Some("claude-token"));
        assert_eq!(claude.config_json.get("providerKind").and_then(serde_json::Value::as_str), Some("bedrock"));
    }

    #[test]
    fn preset_rendering_is_adapter_owned() {
        let service = ToolService::new();
        let existing = "keep = \"yes\"\n";
        let rendered = service
            .render_preset_config(
                CODEX_TOOL_ID,
                &crate::adapters::tool_adapter::PresetConfigBuildInput {
                    name: "Demo".to_string(),
                    provider: "OpenAI".to_string(),
                    model: "gpt-5.5".to_string(),
                    reasoning: "medium".to_string(),
                    base_url: "https://api.openai.com/v1".to_string(),
                    credential_token: None,
                    config_json: serde_json::Value::Null,
                },
                existing,
            )
            .expect("codex preset render should succeed");

        assert!(rendered.contains("keep = \"yes\""));
        assert!(rendered.contains("model = \"gpt-5.5\""));
        assert!(rendered.contains("model_provider = \"OpenAI\""));
        assert!(rendered.contains("[model_providers.OpenAI]"));
    }

    #[test]
    fn adapter_status_codes_stay_in_known_ranges() {
        let diagnostics = ToolService::new()
            .get_diagnostics(CODEX_TOOL_ID)
            .expect("codex diagnostics should be available");
        let target_codes = [
            TARGET_STATE_MISSING,
            TARGET_STATE_READY,
            TARGET_STATE_ERROR,
            TARGET_STATE_PLANNED,
        ];
        let skill_codes = [
            SKILL_INSTALL_NOT_INSTALLED,
            SKILL_INSTALL_INSTALLED,
            SKILL_INSTALL_STALE,
            SKILL_INSTALL_SOURCE_MISSING,
        ];

        assert!(target_codes.contains(&diagnostics.credential_state_code));
        assert!(skill_codes.contains(&diagnostics.skill_state_code));
        assert!(target_codes.contains(&diagnostics.project_output_state_code));
        assert!(target_codes.contains(&diagnostics.repair_state_code));
    }
}
