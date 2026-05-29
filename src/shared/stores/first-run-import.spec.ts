import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useFirstRunImportStore } from './first-run-import'
import { useHistoryStore } from './history'
import { useProvidersStore } from './providers'
import { useRuleStore } from './rules'
import { useSettingsStore } from './settings'
import { useSkillStore } from './skills'
import type { FirstRunImportCandidate, FirstRunImportPreview } from '@/shared/api/client'
import * as tauriApi from '@/shared/api/tauri'

function candidate(partial: Partial<FirstRunImportCandidate>): FirstRunImportCandidate {
  return {
    id: 'candidate',
    assetType: 'rule',
    targetAssetType: 'rule',
    sourceToolId: 101,
    sourceTool: 'codex',
    sourceKind: 'global_rule',
    name: 'Global Rule',
    summary: 'Summary',
    sourcePath: 'C:/Users/Example/.codex/AGENTS.md',
    relativePath: 'AGENTS.md',
    status: 'ready',
    conflict: null,
    existingId: null,
    defaultSelected: true,
    selectable: true,
    recommendedAction: 'create',
    contentPreview: 'Body',
    warnings: [],
    metadata: {},
    ...partial,
  }
}

function preview(candidates: FirstRunImportCandidate[], status = 'pending'): FirstRunImportPreview {
  return {
    status,
    scanVersion: 'global-import-v1',
    roots: [
      {
        tool: 'claude',
        path: 'C:/Users/Example/.claude',
        exists: true,
        candidateCount: candidates.filter((item) => item.sourceTool === 'claude').length,
      },
      {
        tool: 'codex',
        path: 'C:/Users/Example/.codex',
        exists: true,
        candidateCount: candidates.filter((item) => item.sourceTool === 'codex').length,
      },
    ],
    candidates,
    warnings: [],
    credentialPolicy: 'No secret values are imported.',
  }
}

function enableTauriRuntime() {
  Object.defineProperty(window, '__TAURI_INTERNALS__', {
    value: {},
    configurable: true,
  })
}

describe('first-run import store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.restoreAllMocks()
    Reflect.deleteProperty(window, '__TAURI_INTERNALS__')
  })

  it('opens after bootstrap only for pending imports and selects recommended importable candidates', async () => {
    enableTauriRuntime()
    const readyRule = candidate({ id: 'rule', name: 'Rule' })
    const unsupportedCommand = candidate({
      id: 'command',
      assetType: 'command',
      targetAssetType: 'none',
      sourceTool: 'claude',
      sourceToolId: 102,
      sourceKind: 'claude_command',
      name: 'Command',
      status: 'unsupported',
      defaultSelected: false,
      selectable: false,
      recommendedAction: 'unavailable',
      warnings: ['该资源类型暂未开发，仅展示来源。'],
    })
    vi.spyOn(tauriApi, 'getFirstRunImportStatus').mockResolvedValue({
      success: true,
      data: { status: 'pending', shouldPrompt: true },
    })
    vi.spyOn(tauriApi, 'previewFirstRunImport').mockResolvedValue({
      success: true,
      data: preview([readyRule, unsupportedCommand]),
    })

    const store = useFirstRunImportStore()
    await store.maybeOpenAfterBootstrap()

    expect(store.open).toBe(true)
    expect(store.selectedIds).toEqual(['rule'])
    expect(store.activeId).toBe('rule')
    store.toggleCandidate('command')
    expect(store.selectedIds).toEqual(['rule'])
  })

  it('does not auto-open outside Tauri or after the first session check', async () => {
    const statusSpy = vi.spyOn(tauriApi, 'getFirstRunImportStatus')
    const store = useFirstRunImportStore()

    await store.maybeOpenAfterBootstrap()

    expect(statusSpy).not.toHaveBeenCalled()
    expect(store.open).toBe(false)
  })

  it('sends only selected importable ids when applying', async () => {
    const readySkill = candidate({ id: 'skill', assetType: 'skill', targetAssetType: 'skill', name: 'Skill' })
    const unsupportedPrompt = candidate({
      id: 'prompt',
      assetType: 'prompt',
      targetAssetType: 'none',
      sourceKind: 'codex_prompt',
      name: 'Prompt',
      status: 'unsupported',
      defaultSelected: false,
      selectable: false,
      recommendedAction: 'unavailable',
      warnings: ['该资源类型暂未开发，仅展示来源。'],
    })
    vi.spyOn(tauriApi, 'previewFirstRunImport').mockResolvedValue({
      success: true,
      data: preview([readySkill, unsupportedPrompt]),
    })
    vi.spyOn(tauriApi, 'applyFirstRunImport').mockResolvedValue({
      success: true,
      data: {
        importedRules: 0,
        importedSkills: 1,
        importedProviders: 0,
        skipped: 0,
        renamed: 0,
        overwritten: 0,
        assets: [],
        warnings: [],
      },
    })
    vi.spyOn(useRuleStore(), 'hydrateFromSnapshot').mockResolvedValue()
    vi.spyOn(useSkillStore(), 'hydrateFromSnapshot').mockResolvedValue()
    vi.spyOn(useProvidersStore(), 'hydrate').mockResolvedValue()
    vi.spyOn(useSettingsStore(), 'hydrateFromSnapshot').mockResolvedValue()
    vi.spyOn(useHistoryStore(), 'hydrateFromSnapshot').mockResolvedValue()

    const store = useFirstRunImportStore()
    await store.loadPreview()
    store.selectAllInGroup('all')
    await store.applySelected()

    expect(tauriApi.applyFirstRunImport).toHaveBeenCalledWith({
      selectedIds: ['skill'],
      conflictStrategy: 'rename',
      confirm: true,
    })
    expect(store.open).toBe(false)
    expect(store.status).toEqual({ status: 'completed', shouldPrompt: false })
  })

  it('keeps completed and dismissed status from opening automatically', async () => {
    enableTauriRuntime()
    vi.spyOn(tauriApi, 'getFirstRunImportStatus').mockResolvedValue({
      success: true,
      data: { status: 'dismissed', shouldPrompt: false },
    })
    const previewSpy = vi.spyOn(tauriApi, 'previewFirstRunImport')

    const store = useFirstRunImportStore()
    await store.maybeOpenAfterBootstrap()

    expect(previewSpy).not.toHaveBeenCalled()
    expect(store.open).toBe(false)
  })

  it('resets first-run prompt state for launch testing', async () => {
    vi.spyOn(tauriApi, 'resetFirstRunImportStatus').mockResolvedValue({
      success: true,
      data: { status: 'pending', shouldPrompt: true },
    })

    const store = useFirstRunImportStore()
    store.hasCheckedThisSession = true
    await store.resetPromptStatus()

    expect(tauriApi.resetFirstRunImportStatus).toHaveBeenCalled()
    expect(store.status).toEqual({ status: 'pending', shouldPrompt: true })
    expect(store.hasCheckedThisSession).toBe(false)
  })
})
