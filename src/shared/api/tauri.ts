import { invoke } from '@tauri-apps/api/core'
import type {
  AppBootstrap,
  LibraryDiagnostics,
  AppResponse,
  BackupActionResult,
  BackupRestorePreview,
  BackupSnapshot,
  LibrarySnapshot,
  DiagnosticExportResult,
  GlobalOutputPreview,
  GlobalOutputWriteResult,
  HistorySnapshot,
  ProjectOutputPreview,
  ProjectOutputScan,
  ProjectOutputWriteResult,
  ProjectDetail,
  ProviderApplyPreview,
  ProviderApplyResult,
  ProviderImportDraft,
  ProviderImportInput,
  ProviderSaveInput,
  ProviderSummary,
  RepositoryImportReport,
  RuleImpact,
  RuleImportPreview,
  RuleImportResult,
  RuleSummary,
  SaveProjectInput,
  SettingsSnapshot,
  SkillRuntime,
  SkillSummary,
  ToolActionResult,
  ToolDiagnostics,
  ToolsSnapshot,
  ProjectContextSnapshot,
  GitProjectImportInput,
} from './client'

export function getAppBootstrap() {
  return invoke<AppResponse<AppBootstrap>>('get_app_bootstrap')
}

export function resetAppData(confirmRisk: boolean) {
  return invoke<AppResponse<string>>('reset_app_data', { confirmRisk })
}

export function getProjectContextSnapshot() {
  return invoke<AppResponse<ProjectContextSnapshot>>('get_project_context_snapshot')
}

export function getLibrarySnapshot() {
  return invoke<AppResponse<LibrarySnapshot>>('get_library_snapshot')
}

export const getWorkspaceSnapshot = getProjectContextSnapshot
export const getCatalogSnapshot = getLibrarySnapshot

export function getToolsSnapshot(toolId?: number | null) {
  return invoke<AppResponse<ToolsSnapshot>>('get_tools_snapshot', { toolId: toolId ?? null })
}

export function getProjectDetail(projectId: number) {
  return invoke<AppResponse<ProjectDetail>>('get_project_detail', { projectId })
}

export function getHistorySnapshot() {
  return invoke<AppResponse<HistorySnapshot>>('get_history_snapshot')
}

export function getBackupSnapshot() {
  return invoke<AppResponse<BackupSnapshot>>('get_backup_snapshot')
}

export function previewBackupRestore(backupId: string) {
  return invoke<AppResponse<BackupRestorePreview>>('preview_backup_restore', { backupId })
}

export function restoreBackup(backupId: string, confirmRisk: boolean) {
  return invoke<AppResponse<BackupActionResult>>('restore_backup', { backupId, confirmRisk })
}

export function deleteBackup(backupId: string) {
  return invoke<AppResponse<BackupActionResult>>('delete_backup', { backupId })
}

export function exportLibraryDiagnostics() {
  return invoke<AppResponse<DiagnosticExportResult>>('export_library_diagnostics')
}

export function getSettingsSnapshot() {
  return invoke<AppResponse<SettingsSnapshot>>('get_settings_snapshot')
}

export function getToolDiagnostics(toolId: number) {
  return invoke<AppResponse<ToolDiagnostics>>('get_tool_diagnostics', { toolId })
}

export function setToolEnabled(toolId: number, enabled: boolean) {
  return invoke<AppResponse<boolean>>('set_tool_enabled', { toolId, enabled })
}

export function verifyToolCredential(toolId: number, token: string) {
  return invoke<AppResponse<ToolActionResult>>('verify_tool_credential', { toolId, token })
}

export function repairTool(toolId: number) {
  return invoke<AppResponse<ToolActionResult>>('repair_tool', { toolId })
}

export function scanProjectOutput(projectId: number, toolId: number) {
  return invoke<AppResponse<ProjectOutputScan>>('scan_project_output', { projectId, toolId })
}

export function previewProjectOutput(projectId: number, toolId: number) {
  return invoke<AppResponse<ProjectOutputPreview>>('preview_project_output', { projectId, toolId })
}

export function applyProjectOutput(projectId: number, toolId: number, confirmRisk: boolean) {
  return invoke<AppResponse<ProjectOutputWriteResult>>('apply_project_output', { projectId, toolId, confirmRisk })
}

export function repairProjectOutput(projectId: number, toolId: number, confirmRisk: boolean) {
  return invoke<AppResponse<ProjectOutputWriteResult>>('repair_project_output', { projectId, toolId, confirmRisk })
}

export function cleanupProjectOutput(projectId: number, toolId: number, confirmRisk: boolean) {
  return invoke<AppResponse<ProjectOutputWriteResult>>('cleanup_project_output', { projectId, toolId, confirmRisk })
}

export function resetProjectOutput(projectId: number, toolId: number, confirmRisk: boolean) {
  return invoke<AppResponse<ProjectOutputWriteResult>>('reset_project_output', { projectId, toolId, confirmRisk })
}

export function scanLibraryDiagnostics() {
  return invoke<AppResponse<LibraryDiagnostics>>('scan_library_diagnostics')
}

export function saveProjectEntity(input: SaveProjectInput) {
  return invoke<AppResponse<ProjectDetail>>('save_project_entity', input)
}

export function importProjectFromGit(input: GitProjectImportInput) {
  return invoke<AppResponse<ProjectDetail>>('import_project_from_git', input)
}

export function deleteProjectEntity(projectId: number) {
  return invoke<AppResponse<boolean>>('delete_project_entity', { projectId })
}

export function saveProjectRuleBindings(projectId: number, toolId: number | null, ruleIds: number[]) {
  return invoke<AppResponse<boolean>>('save_project_rule_bindings', { projectId, toolId, ruleIds })
}

export function saveToolGlobalRuleBindings(toolId: number, ruleIds: number[]) {
  return invoke<AppResponse<boolean>>('save_tool_global_rule_bindings', { toolId, ruleIds })
}

export function saveToolSkillBindings(toolId: number, skillIds: number[]) {
  return invoke<AppResponse<boolean>>('save_tool_skill_bindings', { toolId, skillIds })
}

export function previewGlobalOutput(toolId: number) {
  return invoke<AppResponse<GlobalOutputPreview>>('preview_global_output', { toolId })
}

export function applyGlobalOutput(toolId: number, confirmRisk: boolean) {
  return invoke<AppResponse<GlobalOutputWriteResult>>('apply_global_output', { toolId, confirmRisk })
}

export function repairGlobalOutput(toolId: number, confirmRisk: boolean) {
  return invoke<AppResponse<GlobalOutputWriteResult>>('repair_global_output', { toolId, confirmRisk })
}

export function cleanupGlobalOutput(toolId: number, confirmRisk: boolean) {
  return invoke<AppResponse<GlobalOutputWriteResult>>('cleanup_global_output', { toolId, confirmRisk })
}

export function saveRuleAsset(
  id: number | null,
  code: number,
  name: string,
  categoryCode: number,
  state: number,
  summary: string,
  body: string,
) {
  return invoke<AppResponse<RuleSummary>>('save_rule_asset', {
    id,
    code,
    name,
    categoryCode,
    state,
    summary,
    body,
  })
}

export function deleteRuleAsset(ruleId: number) {
  return invoke<AppResponse<boolean>>('delete_rule_asset', { ruleId })
}

export function previewRuleImpact(ruleId: number) {
  return invoke<AppResponse<RuleImpact>>('preview_rule_impact', { ruleId })
}

export function previewRuleImport(sourcePath: string) {
  return invoke<AppResponse<RuleImportPreview>>('preview_rule_import', { sourcePath })
}

export function importRuleAsset(sourcePath: string, name: string, categoryCode: number, summary: string, conflictStrategy: string) {
  return invoke<AppResponse<RuleImportResult>>('import_rule_asset', {
    sourcePath,
    name,
    categoryCode,
    summary,
    conflictStrategy,
  })
}

export function moveRuleAsset(ruleId: number, categoryCode: number, sortOrder: number) {
  return invoke<AppResponse<RuleSummary>>('move_rule_asset', { ruleId, categoryCode, sortOrder })
}

export function saveSkillAsset(
  id: number | null,
  code: number,
  name: string,
  categoryCode: number,
  state: number,
  installState: number,
  summary: string,
  body: string,
) {
  return invoke<AppResponse<SkillSummary>>('save_skill_asset', {
    id,
    code,
    name,
    categoryCode,
    state,
    installState,
    summary,
    body,
  })
}

export function deleteSkillAsset(skillId: number) {
  return invoke<AppResponse<boolean>>('delete_skill_asset', { skillId })
}

export function installSkillAsset(toolId: number, skillId: number) {
  return invoke<AppResponse<SkillRuntime>>('install_skill_asset', { toolId, skillId })
}

export function uninstallSkillAsset(toolId: number, skillId: number) {
  return invoke<AppResponse<SkillRuntime>>('uninstall_skill_asset', { toolId, skillId })
}

export function repairSkillAsset(toolId: number, skillId: number) {
  return invoke<AppResponse<SkillRuntime>>('repair_skill_asset', { toolId, skillId })
}

export function markSkillAssetStale(toolId: number, skillId: number) {
  return invoke<AppResponse<SkillRuntime>>('mark_skill_asset_stale', { toolId, skillId })
}

export function listProviders(toolId?: number | null) {
  return invoke<AppResponse<ProviderSummary[]>>('list_providers', { toolId })
}

export function saveProvider(payload: ProviderSaveInput) {
  return invoke<AppResponse<ProviderSummary>>('save_provider', { payload })
}

export function importProviderConfig(payload: ProviderImportInput) {
  return invoke<AppResponse<ProviderImportDraft>>('import_provider_config', { payload })
}

export function deleteProvider(providerId: number) {
  return invoke<AppResponse<boolean>>('delete_provider', { providerId })
}

export function duplicateProvider(providerId: number) {
  return invoke<AppResponse<ProviderSummary>>('duplicate_provider', { providerId })
}

export function previewProviderApply(configId: number) {
  return invoke<AppResponse<ProviderApplyPreview>>('preview_provider_apply', { configId })
}

export function applyProviderToLiveConfig(configId: number, confirmRisk: boolean) {
  return invoke<AppResponse<ProviderApplyResult>>('apply_provider_to_live_config', { configId, confirmRisk })
}

export function saveToolCredentialState(toolId: number, token: string) {
  return invoke<AppResponse<boolean>>('save_tool_credential_state', { toolId, token })
}

export function clearToolCredentialState(toolId: number) {
  return invoke<AppResponse<boolean>>('clear_tool_credential_state', { toolId })
}

export function previewRepositoryImport(source: string, branch: string, conflictStrategy: string) {
  return invoke<AppResponse<RepositoryImportReport>>('preview_repository_import', {
    source,
    branch,
    conflictStrategy,
  })
}

export function applyRepositoryImport(source: string, branch: string, conflictStrategy: string) {
  return invoke<AppResponse<RepositoryImportReport>>('apply_repository_import', {
    source,
    branch,
    conflictStrategy,
  })
}

export function pickFolderPath() {
  return invoke<AppResponse<string | null>>('pick_folder_path')
}

export function pickFilePath(kind: 'markdown' | 'json' | 'all' = 'all') {
  return invoke<AppResponse<string | null>>('pick_file_path', { kind })
}
