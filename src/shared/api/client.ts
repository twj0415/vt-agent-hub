export type AppError = {
  code: string
  message: string
  i18nKey: string
  details?: Record<string, unknown>
}

export type AppResponse<T> = {
  success: boolean
  data?: T
  error?: AppError
}

export type AppBootstrap = {
  appName: string
  projectName?: string
  state: string
  activeToolId: number
}

export type ProjectRuleBinding = {
  toolId?: number | null
  packId: number
  packName: string
  packType: string
  packVersionId: number
  packVersionNo: number
  updatePolicy: string
  enabled: boolean
  items: PackItem[]
}

export type WorkspaceProject = {
  id: number
  name: string
  path: string
  projectType: number
  ruleBindings: ProjectRuleBinding[]
  lastOperation: string
  latestBackup: string
  outputScan?: ProjectOutputScan | null
}

export type ProjectContextSnapshot = {
  activeProjectId: number | null
  activeToolId: number
  projects: WorkspaceProject[]
}

export type RuleSummary = {
  assetId: number
  versionId: number
  versionNo: number
  key: string
  code: number
  name: string
  categoryCode: number
  state: number
  sortOrder: number
  summary: string
  body: string
}

export type RuleImpact = {
  ruleAssetId: number
  ruleName: string
  boundProjectCount: number
  boundToolCount: number
  projectNames: string[]
  toolIds: number[]
  projectToolIds: number[]
  globalToolIds: number[]
  requiresProjectRegeneration: boolean
}

export type RuleImportResult = {
  rule: RuleSummary
  sourcePath: string
  importedName: string
  operation: string
  warnings: string[]
}

export type RuleImportPreview = {
  sourcePath: string
  name: string
  summary: string
  categoryCode?: number | null
  body: string
}

export type RepositoryImportAsset = {
  assetType: string
  name: string
  sourcePath: string
  status: string
  conflict: boolean
}

export type RepositoryImportReport = {
  source: string
  branch: string
  conflictStrategy: string
  previewOnly: boolean
  importedRules: number
  importedSkills: number
  detectedPresets: number
  skipped: number
  overwritten: number
  renamed: number
  assets: RepositoryImportAsset[]
  warnings: string[]
}

export type SkillFileNode = {
  path: string
  isDir: boolean
}

export type SkillRuntime = {
  platformRoot: string
  libraryPath: string
  librarySkillMdPath: string
  runtimePath: string
  runtimeSkillMdPath: string
  libraryExists: boolean
  runtimeExists: boolean
  skillMdValid: boolean
  installState: number
  statusDetail: string
  libraryBody: string
  runtimeBody: string
  libraryTree: SkillFileNode[]
  runtimeTree: SkillFileNode[]
  installActionReady: boolean
  uninstallActionReady: boolean
  repairActionReady: boolean
  markStaleActionReady: boolean
}

export type SkillSummary = {
  assetId: number
  versionId: number
  versionNo: number
  key: string
  code: number
  name: string
  categoryCode: number
  state: number
  summary: string
  body: string
  runtime: SkillRuntime
  toolIds?: number[]
}

export type PackItem = {
  itemType: string
  assetId: number
  assetVersionId: number
  assetVersionNo: number
  sortOrder: number
  required: boolean
}

export type LibrarySnapshot = {
  rules: RuleSummary[]
  skills: SkillSummary[]
}

export type ToolSnapshot = {
  id: number
  name: string
  enabled: boolean
}

export type ToolRulePackBinding = {
  packId: number
  packName: string
  packType: string
  packVersionId: number
  packVersionNo: number
  updatePolicy: string
  enabled: boolean
  items: PackItem[]
}

export type ToolSkillInstall = {
  skillAssetId: number
  requiredVersionId?: number | null
  installedVersionId?: number | null
  state: string
  updatedAt: string
}

export type ToolsSnapshot = {
  tools: ToolSnapshot[]
  globalRuleBinding?: ToolRulePackBinding | null
  skillPackBinding?: ToolRulePackBinding | null
  skillInstalls: ToolSkillInstall[]
}

export type ProjectDetail = {
  id: number
  name: string
  path: string
  projectType: number
  ruleBindings: ProjectRuleBinding[]
  lastOperation: string
  latestBackup: string
}

export type HistoryEntry = {
  id: number
  projectId?: number | null
  toolId?: number | null
  relatedRuleId?: number | null
  kind: string
  title: string
  createdAt: string
  action: string
  result: string
  level: string
  levelCode: number
  detail: string
  relatedPath: string
  navigationTarget: string
}

export type HistoryFilters = {
  projectIds: number[]
  toolIds: number[]
  kinds: string[]
  results: string[]
}

export type HistorySnapshot = {
  entries: HistoryEntry[]
  filters: HistoryFilters
}

export type BackupEntry = {
  id: string
  scope: string
  projectId?: number
  fileName: string
  path: string
  targetPath: string
  createdAt: string
  size: number
}

export type BackupSnapshot = {
  entries: BackupEntry[]
}

export type BackupRestorePreview = {
  backupId: string
  backupPath: string
  targetPath: string
  targetExists: boolean
  beforeContent: string
  afterContent: string
  diff: string
  warning?: string
}

export type BackupActionResult = {
  ok: boolean
  message: string
  path: string
}

export type DiagnosticExportResult = {
  path: string
  issueCount: number
  message: string
}

export type SettingItem = {
  id: number
  name: string
  value: string
}

export type SettingsPath = {
  key: string
  path: string
  note: string
}

export type SettingsTruthSource = {
  key: string
  canonical: string
  mirrors: string[]
  note: string
}

export type SettingsSnapshot = {
  items: SettingItem[]
  paths: SettingsPath[]
  truthSources: SettingsTruthSource[]
}

export type ToolDiagnostics = {
  installationDetected: boolean
  version: string
  liveConfigPath: string
  credentialState: string
  credentialStateCode: number
  skillState: string
  skillStateCode: number
  projectOutputState: string
  projectOutputStateCode: number
  repairState: string
  repairStateCode: number
  repairHint: string
}

export type ToolActionResult = {
  ok: boolean
  state: string
  detail: string
  manualSteps: string[]
}

export type ProjectOutputScan = {
  projectId: number
  toolId: number
  projectName: string
  targetPath: string
  targetExists: boolean
  managed: boolean
  ruleCount: number
  status: string
  statusCode: number
  issues: string[]
}

export type ProjectOutputPreview = {
  projectId: number
  toolId: number
  projectName: string
  targetPath: string
  targetExists: boolean
  managed: boolean
  ruleCount: number
  backupRequired: boolean
  canApply: boolean
  warning?: string
  beforeContent: string
  afterContent: string
  diff: string
  issues: string[]
}

export type ProjectOutputWriteResult = {
  projectId: number
  toolId: number
  operation: string
  targetPath: string
  backupPath?: string
  managed: boolean
  created: boolean
  message: string
}

export type GlobalOutputPreview = {
  toolId: number
  targetPath: string
  targetExists: boolean
  managed: boolean
  ruleCount: number
  backupRequired: boolean
  canApply: boolean
  warning?: string
  beforeContent: string
  afterContent: string
  diff: string
  issues: string[]
}

export type GlobalOutputWriteResult = {
  toolId: number
  operation: string
  targetPath: string
  backupPath?: string
  managed: boolean
  created: boolean
  message: string
}

export type LibraryDiagnosticIssue = {
  scope: string
  key: string
  level: string
  levelCode: number
  detail: string
  relatedPath?: string
}

export type LibraryDiagnostics = {
  projectCount: number
  ruleCount: number
  skillCount: number
  libraryRoot: string
  createdPaths: string[]
  existingPaths: string[]
  issueCount: number
  healthState: string
  healthStateCode: number
  issues: LibraryDiagnosticIssue[]
}

export type SaveProjectInput = {
  id?: number
  name: string
  path: string
  projectType: number
  importMode: boolean
}

export type GitProjectImportInput = {
  repoUrl: string
  targetPath: string
  name?: string
  branch?: string
  projectType: number
}

export type ProviderToolConfigInput = {
  id?: number | null
  toolId: number
  schemaVersion: number
  displayName: string
  model: string
  reasoning: string
  baseUrl: string
  credentialRef?: string | null
  credentialToken?: string | null
  configJson?: Record<string, unknown> | null
}

export type ProviderSaveInput = {
  id?: number | null
  name: string
  category: string
  website: string
  note: string
  toolConfigs: ProviderToolConfigInput[]
}

export type ProviderImportInputPart = {
  role: string
  content: string
}

export type ProviderImportInput = {
  toolId: number
  parts: ProviderImportInputPart[]
}

export type ProviderToolConfig = {
  id: number
  providerId: number
  toolId: number
  schemaVersion: number
  displayName: string
  model: string
  reasoning: string
  baseUrl: string
  credentialRef: string
  hasCredential: boolean
  configJson: Record<string, unknown> | null
  isActive: boolean
  state: number
  lastCheckStatus: string
  lastCheckLatencyMs?: number | null
  lastCheckMessage: string
  lastCheckedAt: string
}

export type ProviderSummary = {
  id: number
  name: string
  category: string
  website: string
  note: string
  sortOrder: number
  configs: ProviderToolConfig[]
}

export type ProviderImportDraft = {
  sourceKind: string
  detectedParts: string[]
  toolId: number
  schemaVersion: number
  name: string
  category: string
  website: string
  note: string
  displayName: string
  model: string
  reasoning: string
  baseUrl: string
  credentialRef: string
  hasCredential: boolean
  credentialToken?: string | null
  configJson: Record<string, unknown> | null
}

export type ProviderApplyFilePreview = {
  label: string
  targetPath: string
  targetExists: boolean
  backupRequired: boolean
  beforeContent: string
  afterContent: string
  diff: string
}

export type ProviderApplyPreview = {
  toolId: number
  providerId: number
  configId: number
  providerName: string
  targetPath: string
  targetExists: boolean
  backupRequired: boolean
  beforeContent: string
  afterContent: string
  diff: string
  files: ProviderApplyFilePreview[]
  warning?: string
}

export type ProviderApplyResult = {
  toolId: number
  providerId: number
  configId: number
  operation: string
  targetPath: string
  backupPath?: string
  targetPaths: string[]
  backupPaths: string[]
  message: string
}

