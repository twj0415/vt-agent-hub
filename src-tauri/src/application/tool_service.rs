use crate::adapters::claude::ClaudeAdapter;
use crate::adapters::codex::CodexAdapter;
use crate::adapters::cursor::CursorAdapter;
use std::path::PathBuf;

use crate::adapters::tool_adapter::{
    PresetConfigBuildInput, ProjectOutputBuildInput, ProviderConfigImport, ToolActionResult,
    ToolAdapter, ToolDiagnostics,
};
use crate::core::tool_registry::{
    get_tool, ToolCapabilitySet, CLAUDE_TOOL_ID, CODEX_TOOL_ID, CURSOR_TOOL_ID,
};

#[derive(Debug, Default)]
pub struct ToolService {
    codex_adapter: CodexAdapter,
    claude_adapter: ClaudeAdapter,
    cursor_adapter: CursorAdapter,
}

impl ToolService {
    pub fn new() -> Self {
        Self::default()
    }

    fn ensure_enabled_tool(&self, tool_id: i32) -> Result<i32, String> {
        let tool = get_tool(tool_id).ok_or_else(|| format!("Unknown tool id: {tool_id}"))?;

        if !tool.enabled {
            return Err(format!("Tool {} is not enabled yet", tool.key));
        }

        Ok(tool.id)
    }

    fn with_adapter<T>(
        &self,
        tool_id: i32,
        unsupported_message: &str,
        operation: impl FnOnce(&dyn ToolAdapter) -> T,
    ) -> Result<T, String> {
        let tool_id = self.ensure_enabled_tool(tool_id)?;

        match tool_id {
            CODEX_TOOL_ID => Ok(operation(&self.codex_adapter)),
            CLAUDE_TOOL_ID => Ok(operation(&self.claude_adapter)),
            CURSOR_TOOL_ID => Ok(operation(&self.cursor_adapter)),
            _ => Err(format!("{unsupported_message} {tool_id}")),
        }
    }

    pub fn get_diagnostics(&self, tool_id: i32) -> Result<ToolDiagnostics, String> {
        self.with_adapter(
            tool_id,
            "Diagnostics adapter is not implemented for tool id",
            |adapter| adapter.diagnostics(),
        )
    }

    pub fn capabilities(&self, tool_id: i32) -> Result<ToolCapabilitySet, String> {
        Ok(get_tool(tool_id)
            .ok_or_else(|| format!("Unknown tool id: {tool_id}"))?
            .capabilities)
    }

    pub fn verify_credential(&self, tool_id: i32, token: &str) -> Result<ToolActionResult, String> {
        self.with_adapter(
            tool_id,
            "Credential verification adapter is not implemented for tool id",
            |adapter| adapter.verify_credential(token),
        )
    }

    pub fn repair(&self, tool_id: i32) -> Result<ToolActionResult, String> {
        self.with_adapter(
            tool_id,
            "Repair adapter is not implemented for tool id",
            |adapter| adapter.repair(),
        )
    }

    pub fn project_output_target_path(
        &self,
        tool_id: i32,
        project_root: &str,
    ) -> Result<PathBuf, String> {
        self.with_adapter(
            tool_id,
            "Project output target path adapter is not implemented for tool id",
            |adapter| adapter.project_output_target_path(project_root),
        )
    }

    pub fn global_output_target_path(&self, tool_id: i32) -> Result<PathBuf, String> {
        self.with_adapter(
            tool_id,
            "Global output target path adapter is not implemented for tool id",
            |adapter| adapter.global_output_target_path(),
        )?
        .ok_or_else(|| format!("Tool {} does not support global managed output.", tool_id))
    }

    pub fn skill_runtime_root(&self, tool_id: i32) -> Result<PathBuf, String> {
        self.with_adapter(
            tool_id,
            "Skill runtime adapter is not implemented for tool id",
            |adapter| adapter.skill_runtime_root(),
        )?
        .ok_or_else(|| format!("Tool {} does not support skill installation.", tool_id))
    }

    pub fn preset_config_path(&self, tool_id: i32) -> Result<PathBuf, String> {
        self.with_adapter(
            tool_id,
            "Preset config adapter is not implemented for tool id",
            |adapter| adapter.preset_config_path(),
        )?
        .ok_or_else(|| format!("Tool {} does not support managed preset config.", tool_id))
    }

    pub fn project_output_managed_marker(&self, tool_id: i32) -> Result<&'static str, String> {
        self.with_adapter(
            tool_id,
            "Project output marker adapter is not implemented for tool id",
            |adapter| adapter.project_output_managed_marker(),
        )
    }

    pub fn render_project_output(
        &self,
        tool_id: i32,
        input: &ProjectOutputBuildInput,
    ) -> Result<String, String> {
        self.with_adapter(
            tool_id,
            "Project output render adapter is not implemented for tool id",
            |adapter| adapter.render_project_output(input),
        )
    }

    pub fn render_preset_config(
        &self,
        tool_id: i32,
        input: &PresetConfigBuildInput,
        existing: &str,
    ) -> Result<String, String> {
        self.with_adapter(
            tool_id,
            "Preset config render adapter is not implemented for tool id",
            |adapter| adapter.render_preset_config(input, existing),
        )?
    }

    pub fn import_live_preset(
        &self,
        tool_id: i32,
        content: &str,
    ) -> Result<PresetConfigBuildInput, String> {
        self.with_adapter(
            tool_id,
            "Preset live import adapter is not implemented for tool id",
            |adapter| adapter.import_live_preset(content),
        )?
    }

    pub fn import_provider_config(
        &self,
        tool_id: i32,
        input: &crate::dto::ProviderImportInputDto,
    ) -> Result<ProviderConfigImport, String> {
        self.with_adapter(
            tool_id,
            "Provider config import adapter is not implemented for tool id",
            |adapter| adapter.import_provider_config(input),
        )?
    }
}

