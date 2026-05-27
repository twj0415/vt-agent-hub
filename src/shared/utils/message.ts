import { translateIfExists } from '@/shared/i18n/translate'

const projectMessageKeys: Record<string, string> = {
  'Project imported.': 'feedback.projectImported',
  'Project saved.': 'feedback.projectSaved',
  'Project deleted.': 'feedback.projectDeleted',
  'Project rule bindings saved.': 'feedback.projectRuleBindingsSaved',
  'Project scan completed.': 'feedback.projectScanCompleted',
  'Project AGENTS.md applied.': 'feedback.projectAgentsApplied',
  'Project AGENTS.md repaired.': 'feedback.projectAgentsRepaired',
  'Project AGENTS.md removed.': 'feedback.projectAgentsRemoved',
  'Project AGENTS.md reset to unmanaged state.': 'feedback.projectAgentsReset',
  'Project AGENTS cleanup completed.': 'feedback.projectAgentsCleanupCompleted',
  'Project AGENTS reset completed.': 'feedback.projectAgentsResetCompleted',
  'Rule file picker failed.': 'errors.ruleFilePickerFailed',
  'Project input is invalid.': 'errors.projectInputInvalid',
  'Project path is required.': 'errors.projectPathRequired',
  'Project type is not supported.': 'errors.projectTypeUnsupported',
  'Project save failed.': 'errors.projectSaveFailed',
  'Project delete failed.': 'errors.projectDeleteFailed',
  'Project rule bindings failed.': 'errors.projectRuleBindingsFailed',
  'Project output scan failed.': 'errors.projectOutputScanFailed',
  'Project output preview failed.': 'errors.projectOutputPreviewFailed',
  'Project output write failed.': 'errors.projectOutputWriteFailed',
  'Workspace snapshot failed.': 'errors.projectContextSnapshotFailed',
  'Folder picker failed.': 'errors.folderPickerFailed',
  'Project import from git failed.': 'errors.projectImportFromGitFailed',
  'Repository URL is required.': 'errors.gitRepositoryRequired',
  'Target path is required.': 'errors.gitTargetPathRequired',
  'Target path already exists.': 'errors.gitTargetPathExists',
  'Git clone target path is required.': 'errors.gitTargetPathRequired',
  'project.path must exist when importing a project.': 'errors.projectPathMissing',
  'project.path must be a directory when importing a project.': 'errors.projectPathNotDirectory',
  'Project has no bound rules for the current tool view.': 'errors.projectNoBoundRules',
  'Tool global output has no bound rules.': 'errors.toolGlobalNoBoundRules',
  'Global AGENTS.md is not managed and requires repair confirmation.': 'errors.globalAgentsRepairConfirmationRequired',
  'Global AGENTS.md is not managed. Use repair instead.': 'errors.globalAgentsUseRepair',
  'Global AGENTS.md is already absent.': 'errors.globalAgentsAlreadyAbsent',
  'Global AGENTS.md is not a VT Hub Manager managed file. Use repair instead of cleanup.': 'errors.globalAgentsCleanupUnmanaged',
  'Global AGENTS.md removed.': 'feedback.globalAgentsRemoved',
  'Global AGENTS.md repaired.': 'feedback.globalAgentsRepaired',
  'Global AGENTS.md applied.': 'feedback.globalAgentsApplied',
  'Global CLAUDE.md removed.': 'feedback.globalAgentsRemoved',
  'Global CLAUDE.md repaired.': 'feedback.globalAgentsRepaired',
  'Global CLAUDE.md applied.': 'feedback.globalAgentsApplied',
  'Skill saved.': 'feedback.skillSaved',
  'Skill deleted.': 'feedback.skillDeleted',
  'Preset saved.': 'feedback.presetSaved',
  'Preset deleted.': 'feedback.presetDeleted',
  'Preset imported from live config.': 'feedback.presetImportedFromLiveConfig',
  'Credential cleared.': 'feedback.credentialCleared',
  'Diagnostics scan completed.': 'feedback.diagnosticsScanCompleted',
  'Skill bindings saved.': 'feedback.skillBindingsSaved',
  'Check connection': 'feedback.checkConnection',
  'Settings snapshot failed.': 'errors.settingsSnapshotFailed',
  'Application reset failed.': 'errors.applicationResetFailed',
  'Tools snapshot failed.': 'errors.toolsSnapshotFailed',
  'Tool diagnostics failed.': 'errors.toolDiagnosticsFailed',
  'Library diagnostics failed.': 'errors.libraryDiagnosticsFailed',
  'Credential input is invalid.': 'errors.credentialInputInvalid',
  'Credential verification failed.': 'errors.credentialVerificationFailed',
  'Credential persistence failed.': 'errors.credentialPersistenceFailed',
  'Repair failed.': 'errors.toolRepairFailed',
  'Global AGENTS preview failed.': 'errors.globalAgentsPreviewFailed',
  'Global AGENTS apply failed.': 'errors.globalAgentsApplyFailed',
  'Global AGENTS repair failed.': 'errors.globalAgentsRepairFailed',
  'Credential clear failed.': 'errors.credentialClearFailed',
  'Diagnostics export failed.': 'errors.diagnosticsExportFailed',
  'Risk confirmation is required before writing AGENTS.md.': 'errors.projectWriteRiskRequired',
  'Risk confirmation is required before deleting AGENTS.md.': 'errors.projectDeleteRiskRequired',
  'Target AGENTS.md is not a VT Hub Manager managed file. Use repair instead of cleanup/reset.': 'errors.projectTargetUnmanaged',
  'Preset asset picker failed.': 'errors.presetAssetPickerFailed',
}

const dynamicProjectMessages = [
  {
    pattern: /^Project path already exists:\s*(.+)$/i,
    key: 'errors.projectPathExists',
    params: (match: RegExpMatchArray) => ({ path: match[1] }),
  },
  {
    pattern: /^git clone failed:\s*(.+)$/i,
    key: 'errors.gitCloneFailed',
    params: (match: RegExpMatchArray) => ({ detail: match[1] }),
  },
  {
    pattern: /^Global (AGENTS|CLAUDE)\.md is not managed and requires repair confirmation\.$/i,
    key: 'errors.globalAgentsRepairConfirmationRequired',
  },
  {
    pattern: /^Global (AGENTS|CLAUDE)\.md is not managed\. Use repair instead\.$/i,
    key: 'errors.globalAgentsUseRepair',
  },
  {
    pattern: /^Global (AGENTS|CLAUDE)\.md is already absent\.$/i,
    key: 'errors.globalAgentsAlreadyAbsent',
  },
  {
    pattern: /^Global (AGENTS|CLAUDE)\.md is not a VT Hub Manager managed file\. Use repair instead of cleanup\.$/i,
    key: 'errors.globalAgentsCleanupUnmanaged',
  },
]

export function localizeMessage(message: string) {
  const normalized = message.trim()
  if (!normalized) return ''
  for (const item of dynamicProjectMessages) {
    const match = normalized.match(item.pattern)
    if (match) return translateIfExists(item.key, normalized, item.params?.(match))
  }
  return translateIfExists(projectMessageKeys[normalized], normalized)
}
