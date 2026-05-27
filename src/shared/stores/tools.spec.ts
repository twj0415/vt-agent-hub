import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useToolsStore } from './tools'
import { toolIds } from '@/shared/tool-registry'
import * as tauriApi from '@/shared/api/tauri'

describe('tools store global AGENTS flow', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.restoreAllMocks()
    vi.stubGlobal('performance', { now: () => 0 })
  })

  it('previews and applies global AGENTS output', async () => {
    vi.spyOn(tauriApi, 'previewGlobalOutput').mockResolvedValue({
      success: true,
      data: {
        toolId: 101,
        targetPath: 'C:\\Users\\Example\\.codex\\AGENTS.md',
        targetExists: false,
        managed: false,
        ruleCount: 1,
        backupRequired: false,
        canApply: true,
        warning: 'Applying will write Codex global AGENTS.md.',
        beforeContent: '',
        afterContent: 'managed global agents',
        diff: 'diff',
        issues: [],
      },
    })
    vi.spyOn(tauriApi, 'applyGlobalOutput').mockResolvedValue({
      success: true,
      data: {
        toolId: 101,
        operation: 'global.apply_agents',
        targetPath: 'C:\\Users\\Example\\.codex\\AGENTS.md',
        managed: true,
        created: true,
        message: 'Global AGENTS.md applied.',
      },
    })

    const store = useToolsStore()
    await store.loadGlobalPreview()
    await store.applyGlobalAgents()

    expect(store.globalPreview?.afterContent).toBe('managed global agents')
    expect(store.globalWriteResult?.operation).toBe('global.apply_agents')
  })

  it('hydrates real global rule ids and library diagnostics from snapshot', async () => {
    vi.spyOn(tauriApi, 'getToolsSnapshot').mockResolvedValue({
      success: true,
      data: {
        tools: [{ id: 101, name: 'Codex', enabled: true }],
        presets: [],
        globalRuleBinding: {
          packId: 3,
          packName: 'Codex Global Rules',
          packType: 'tool_global_rules',
          packVersionId: 3,
          packVersionNo: 1,
          updatePolicy: 'notify',
          enabled: true,
          items: [],
        },
        skillPackBinding: null,
        skillInstalls: [],
      },
    })
    vi.spyOn(tauriApi, 'getToolDiagnostics').mockResolvedValue({
      success: true,
      data: {
        installationDetected: true,
        liveConfigPath: 'C:\\Users\\Example\\.codex\\config.toml',
        credentialState: 'config_present',
        credentialStateCode: 502,
        skillState: 'installed',
        skillStateCode: 602,
        projectOutputState: 'preview_ready',
        projectOutputStateCode: 502,
        repairState: 'ready',
        repairStateCode: 502,
        repairHint: 'Use Workspace preview before apply.',
      },
    })
    vi.spyOn(tauriApi, 'scanLibraryDiagnostics').mockResolvedValue({
      success: true,
      data: {
        projectCount: 2,
        ruleCount: 4,
        skillCount: 1,
        libraryRoot: 'C:\\Users\\Example\\.vt-agent-hub\\library',
        createdPaths: [],
        existingPaths: ['C:\\Users\\Example\\.vt-agent-hub\\library\\rules'],
        issueCount: 1,
        healthState: 'attention',
        healthStateCode: 702,
        issues: [
          {
            scope: 'tool',
            key: 'credential',
            level: 'warning',
            levelCode: 703,
            detail: 'Credential was not verified against a remote provider.',
          },
        ],
      },
    })

    const store = useToolsStore()
    await store.hydrateFromSnapshot()

    expect(store.globalRuleBinding?.packVersionId).toBe(3)
    expect(store.libraryDiagnostics?.issueCount).toBe(1)
  })

  it('saves selected tool rules and applies global output immediately', async () => {
    vi.spyOn(tauriApi, 'saveToolGlobalRuleBindings').mockResolvedValue({
      success: true,
      data: true,
    })
    vi.spyOn(tauriApi, 'applyGlobalOutput').mockResolvedValue({
      success: true,
      data: {
        toolId: 101,
        operation: 'global.apply_agents',
        targetPath: 'C:\\Users\\Example\\.codex\\AGENTS.md',
        managed: true,
        created: false,
        message: 'Global AGENTS.md applied.',
      },
    })
    vi.spyOn(tauriApi, 'previewGlobalOutput').mockResolvedValue({
      success: true,
      data: {
        toolId: 101,
        targetPath: 'C:\\Users\\Example\\.codex\\AGENTS.md',
        targetExists: true,
        managed: true,
        ruleCount: 2,
        backupRequired: false,
        canApply: true,
        warning: '',
        beforeContent: '',
        afterContent: '## First rule\n\nBody\n\n---\n\n## Second rule\n\nBody\n',
        diff: '',
        issues: [],
      },
    })
    vi.spyOn(tauriApi, 'getToolsSnapshot').mockResolvedValue({
      success: true,
      data: {
        tools: [{ id: 101, name: 'Codex', enabled: true }],
        presets: [],
        globalRuleBinding: null,
        skillPackBinding: null,
        skillInstalls: [],
      },
    })
    vi.spyOn(tauriApi, 'getToolDiagnostics').mockResolvedValue({
      success: true,
      data: {
        installationDetected: true,
        liveConfigPath: 'C:\\Users\\Example\\.codex\\config.toml',
        credentialState: 'config_present',
        credentialStateCode: 502,
        skillState: 'installed',
        skillStateCode: 602,
        projectOutputState: 'preview_ready',
        projectOutputStateCode: 502,
        repairState: 'ready',
        repairStateCode: 502,
        repairHint: 'Ready.',
      },
    })
    vi.spyOn(tauriApi, 'scanLibraryDiagnostics').mockResolvedValue({
      success: true,
      data: {
        projectCount: 0,
        ruleCount: 0,
        skillCount: 0,
        libraryRoot: 'C:\\Users\\Example\\.vt-agent-hub\\library',
        createdPaths: [],
        existingPaths: [],
        issueCount: 0,
        healthState: 'normal',
        healthStateCode: 702,
        issues: [],
      },
    })
    vi.spyOn(tauriApi, 'getLibrarySnapshot').mockResolvedValue({
      success: true,
      data: { rules: [], skills: [] },
    })

    const store = useToolsStore()
    store.bindingDraft.selectedNewRuleIds = [1, 2]
    await store.saveRuleBindingAndApply()

    expect(tauriApi.saveToolGlobalRuleBindings).toHaveBeenCalledWith(101, [1, 2])
    expect(tauriApi.applyGlobalOutput).toHaveBeenCalledWith(101, true)
    expect(tauriApi.previewGlobalOutput).toHaveBeenCalledWith(101)
    expect(store.globalPreview?.afterContent).toContain('## First rule')
    expect(store.bindOpen).toBe(false)
  })

  it('saves selected Claude rules through the same global output flow', async () => {
    vi.spyOn(tauriApi, 'saveToolGlobalRuleBindings').mockResolvedValue({
      success: true,
      data: true,
    })
    vi.spyOn(tauriApi, 'applyGlobalOutput').mockResolvedValue({
      success: true,
      data: {
        toolId: toolIds.claude,
        operation: 'global.apply_agents',
        targetPath: 'C:\\Users\\Example\\.claude\\CLAUDE.md',
        managed: true,
        created: false,
        message: 'Global CLAUDE.md applied.',
      },
    })
    vi.spyOn(tauriApi, 'previewGlobalOutput').mockResolvedValue({
      success: true,
      data: {
        toolId: toolIds.claude,
        targetPath: 'C:\\Users\\Example\\.claude\\CLAUDE.md',
        targetExists: true,
        managed: true,
        ruleCount: 1,
        backupRequired: false,
        canApply: true,
        warning: '',
        beforeContent: '',
        afterContent: '## Claude rule\n\nBody\n',
        diff: '',
        issues: [],
      },
    })
    vi.spyOn(tauriApi, 'getToolsSnapshot').mockResolvedValue({
      success: true,
      data: {
        tools: [{ id: toolIds.claude, name: 'Claude', enabled: true }],
        presets: [],
        globalRuleBinding: null,
        skillPackBinding: null,
        skillInstalls: [],
      },
    })
    vi.spyOn(tauriApi, 'getToolDiagnostics').mockResolvedValue({
      success: true,
      data: {
        installationDetected: true,
        liveConfigPath: 'C:\\Users\\Example\\.claude\\CLAUDE.md',
        credentialState: 'managed_elsewhere',
        credentialStateCode: 602,
        skillState: 'tool_local',
        skillStateCode: 602,
        projectOutputState: 'preview_ready',
        projectOutputStateCode: 502,
        repairState: 'manual_required',
        repairStateCode: 602,
        repairHint: 'Review Claude memory before overwrite.',
      },
    })
    vi.spyOn(tauriApi, 'scanLibraryDiagnostics').mockResolvedValue({
      success: true,
      data: {
        projectCount: 0,
        ruleCount: 0,
        skillCount: 0,
        libraryRoot: 'C:\\Users\\Example\\.vt-agent-hub\\library',
        createdPaths: [],
        existingPaths: [],
        issueCount: 0,
        healthState: 'normal',
        healthStateCode: 702,
        issues: [],
      },
    })
    vi.spyOn(tauriApi, 'getLibrarySnapshot').mockResolvedValue({
      success: true,
      data: { rules: [], skills: [] },
    })

    const store = useToolsStore()
    store.activeId = toolIds.claude
    store.bindingDraft.selectedNewRuleIds = [3]
    await store.saveRuleBindingAndApply()

    expect(tauriApi.saveToolGlobalRuleBindings).toHaveBeenCalledWith(toolIds.claude, [3])
    expect(tauriApi.applyGlobalOutput).toHaveBeenCalledWith(toolIds.claude, true)
    expect(tauriApi.previewGlobalOutput).toHaveBeenCalledWith(toolIds.claude)
    expect(store.globalPreview?.afterContent).toContain('## Claude rule')
    expect(store.bindOpen).toBe(false)
  })

  it('keeps save credential separate from connection checks', async () => {
    Object.defineProperty(window, '__TAURI_INTERNALS__', { configurable: true, value: {} })
    vi.spyOn(tauriApi, 'saveToolCredentialState').mockResolvedValue({
      success: false,
      error: {
        code: 'credential_write_failed',
        message: 'Credential store unavailable.',
        i18nKey: 'errors.credentialWriteFailed',
      },
    })
    const verifySpy = vi.spyOn(tauriApi, 'verifyToolCredential')

    const store = useToolsStore()
    store.setDraftField('token', '12345678-token')
    await store.verifyCredential()

    expect(tauriApi.saveToolCredentialState).not.toHaveBeenCalled()
    expect(verifySpy).toHaveBeenCalledWith(101, '12345678-token')
  })

  it('reports credential save failures without running connection checks', async () => {
    Object.defineProperty(window, '__TAURI_INTERNALS__', { configurable: true, value: {} })
    vi.spyOn(tauriApi, 'saveToolCredentialState').mockResolvedValue({
      success: false,
      error: {
        code: 'credential_write_failed',
        message: 'Credential store unavailable.',
        i18nKey: 'errors.credentialWriteFailed',
      },
    })
    const verifySpy = vi.spyOn(tauriApi, 'verifyToolCredential')

    const store = useToolsStore()
    store.setDraftField('token', '12345678-token')
    await store.saveCredential()

    expect(verifySpy).not.toHaveBeenCalled()
    expect(store.verifyResult.ok).toBe(false)
    expect(store.verifyResult.state).toBe('credential_write_failed')
  })
})
