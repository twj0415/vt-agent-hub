import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useAppStore } from './app'
import * as tauriApi from '@/shared/api/tauri'

describe('app store bootstrap', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.restoreAllMocks()
  })

  it('loads core snapshots from one bootstrap action', async () => {
    vi.spyOn(tauriApi, 'getAppBootstrap').mockResolvedValue({
      success: true,
      data: {
        appName: 'VT Hub Manager',
        state: 'Planned',
        activeToolId: 101,
      },
    })
    vi.spyOn(tauriApi, 'getProjectContextSnapshot').mockResolvedValue({
      success: true,
      data: { activeProjectId: 1, activeToolId: 101, projects: [] },
    })
    vi.spyOn(tauriApi, 'getToolsSnapshot').mockResolvedValue({
      success: true,
      data: { tools: [{ id: 101, name: 'Codex', enabled: true }], presets: [], globalRuleBinding: null, skillPackBinding: null, skillInstalls: [] },
    })
    vi.spyOn(tauriApi, 'getToolDiagnostics').mockResolvedValue({
      success: true,
      data: {
        installationDetected: true,
        liveConfigPath: 'C:\\Users\\Example\\.codex\\config.toml',
        credentialState: 'present',
        credentialStateCode: 502,
        skillState: 'installed',
        skillStateCode: 602,
        projectOutputState: 'ready',
        projectOutputStateCode: 502,
        repairState: 'ready',
        repairStateCode: 502,
        repairHint: '',
      },
    })
    vi.spyOn(tauriApi, 'getLibrarySnapshot').mockResolvedValue({
      success: true,
      data: { rules: [], skills: [] },
    })
    vi.spyOn(tauriApi, 'getHistorySnapshot').mockResolvedValue({
      success: true,
      data: {
        entries: [],
        filters: {
          projectIds: [],
          toolIds: [],
          kinds: [],
          results: [],
        },
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
        healthStateCode: 502,
        issues: [],
      },
    })
    vi.spyOn(tauriApi, 'getSettingsSnapshot').mockResolvedValue({
      success: true,
      data: { items: [], paths: [], truthSources: [] },
    })

    const store = useAppStore()
    await store.bootstrapAll()

    expect(store.ready).toBe(true)
    expect(store.error).toBe('')
    expect(tauriApi.getProjectContextSnapshot).toHaveBeenCalledTimes(1)
    expect(tauriApi.getLibrarySnapshot).toHaveBeenCalledTimes(2)
    expect(tauriApi.getToolsSnapshot).toHaveBeenCalledTimes(1)
  })

  it('records startup errors instead of leaving the app in an unknown state', async () => {
    vi.spyOn(tauriApi, 'getAppBootstrap').mockResolvedValue({
      success: false,
      error: { code: 'boom', message: 'backend failed', i18nKey: 'errors.appBootstrapFailed' },
    })

    const store = useAppStore()
    await store.bootstrapAll()

    expect(store.ready).toBe(true)
    expect(store.error).toBe('backend failed')
    expect(store.bootstrapErrors).toContain('backend failed')
  })
})
