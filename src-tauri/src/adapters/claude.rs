use std::path::PathBuf;

use crate::adapters::tool_adapter::{
    render_managed_rules_markdown, ProjectOutputBuildInput, ToolActionResult, ToolAdapter,
};
use crate::core::product::MANAGED_MARKER;
use crate::core::status_codes::{TARGET_STATE_MISSING, TARGET_STATE_PLANNED, TARGET_STATE_READY};
use crate::core::tool_registry::CLAUDE_TOOL_ID;
use crate::domain::tool::Tool;
use crate::infrastructure::claude_runtime_repo::ClaudeRuntimeRepo;

#[derive(Debug, Default, Clone, Copy)]
pub struct ClaudeAdapter;

impl ToolAdapter for ClaudeAdapter {
    fn tool(&self) -> Tool {
        Tool {
            id: CLAUDE_TOOL_ID,
            key: "claude",
            enabled: true,
        }
    }

    fn detect_installation(&self) -> bool {
        ClaudeRuntimeRepo::root().exists()
    }

    fn version(&self) -> String {
        "-".to_string()
    }

    fn live_config_path(&self) -> String {
        ClaudeRuntimeRepo::global_claude_path()
            .display()
            .to_string()
    }

    fn credential_state(&self) -> String {
        "managed_elsewhere".to_string()
    }

    fn credential_state_code(&self) -> i32 {
        TARGET_STATE_PLANNED
    }

    fn skill_state(&self) -> String {
        "tool_local".to_string()
    }

    fn skill_state_code(&self) -> i32 {
        TARGET_STATE_PLANNED
    }

    fn project_output_state(&self) -> String {
        if self.detect_installation() {
            "preview_ready".to_string()
        } else {
            "tool_missing".to_string()
        }
    }

    fn project_output_state_code(&self) -> i32 {
        if self.detect_installation() {
            TARGET_STATE_READY
        } else {
            TARGET_STATE_MISSING
        }
    }

    fn repair_state(&self) -> String {
        "manual_required".to_string()
    }

    fn repair_state_code(&self) -> i32 {
        TARGET_STATE_PLANNED
    }

    fn repair_hint(&self) -> String {
        "Claude Code global and project memory should be reviewed before overwrite.".to_string()
    }

    fn verify_credential(&self, _token: &str) -> ToolActionResult {
        ToolActionResult {
            ok: false,
            state: "unsupported".to_string(),
            detail: "Credential verification is not implemented for Claude Code here.".to_string(),
            manual_steps: vec![
                "Claude Code is hidden in V1 and has no supported credential flow.".to_string(),
            ],
        }
    }

    fn repair(&self) -> ToolActionResult {
        ToolActionResult {
            ok: false,
            state: "manual_required".to_string(),
            detail: "Claude Code repair requires manual review of the target memory file."
                .to_string(),
            manual_steps: vec![
                "Claude Code is hidden in V1 and has no supported repair flow.".to_string(),
            ],
        }
    }

    fn project_output_target_path(&self, project_root: &str) -> PathBuf {
        PathBuf::from(project_root).join("CLAUDE.md")
    }

    fn global_output_target_path(&self) -> Option<PathBuf> {
        Some(ClaudeRuntimeRepo::global_claude_path())
    }

    fn skill_runtime_root(&self) -> Option<PathBuf> {
        Some(ClaudeRuntimeRepo::root().join("skills"))
    }

    fn preset_config_path(&self) -> Option<PathBuf> {
        None
    }

    fn project_output_managed_marker(&self) -> &'static str {
        MANAGED_MARKER
    }

    fn render_project_output(&self, input: &ProjectOutputBuildInput) -> String {
        render_managed_rules_markdown(input)
    }
}
