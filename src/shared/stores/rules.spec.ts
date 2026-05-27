import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useRuleStore } from './rules'
import * as tauriApi from '@/shared/api/tauri'

describe('rules store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.restoreAllMocks()
  })

  it('prefers preview name over file name without auto-selecting category when importing rules', async () => {
    vi.spyOn(tauriApi, 'previewRuleImport').mockResolvedValue({
      success: true,
      data: {
        sourcePath: 'D:\\temp\\example-rule.md',
        name: 'Previewed Rule Name',
        summary: 'Previewed summary',
        categoryCode: 305,
        body: 'Previewed body',
      },
    })

    const store = useRuleStore()
    store.setImportField('sourcePath', 'D:\\temp\\example-rule.md')

    await vi.waitFor(() => {
      expect(store.importDraft.name).toBe('Previewed Rule Name')
      expect(store.importDraft.summary).toBe('Previewed summary')
      expect(store.importDraft.categoryCode).toBeNull()
      expect(store.importDraft.body).toBe('Previewed body')
    })
  })

  it('keeps missing frontmatter fields empty when previewing imports', async () => {
    vi.spyOn(tauriApi, 'previewRuleImport').mockResolvedValue({
      success: true,
      data: {
        sourcePath: 'D:\\temp\\example-rule.md',
        name: '',
        summary: '',
        categoryCode: null,
        body: 'Body only',
      },
    })

    const store = useRuleStore()
    store.setImportField('sourcePath', 'D:\\temp\\example-rule.md')

    await vi.waitFor(() => {
      expect(store.importDraft.name).toBe('')
      expect(store.importDraft.summary).toBe('')
      expect(store.importDraft.categoryCode).toBeNull()
      expect(store.importDraft.body).toBe('Body only')
    })
  })

  it('does not overwrite manually edited import fields when preview returns late', async () => {
    let resolvePreview: (value: Awaited<ReturnType<typeof tauriApi.previewRuleImport>>) => void = () => {}
    vi.spyOn(tauriApi, 'previewRuleImport').mockReturnValue(new Promise((resolve) => {
      resolvePreview = resolve
    }))

    const store = useRuleStore()
    store.setImportField('sourcePath', 'D:\\temp\\example-rule.md')
    store.setImportField('name', 'Manual name')
    store.setImportField('summary', 'Manual summary')
    store.setImportField('categoryCode', 306)

    resolvePreview({
      success: true,
      data: {
        sourcePath: 'D:\\temp\\example-rule.md',
        name: '',
        summary: '',
        categoryCode: null,
        body: 'Previewed body',
      },
    })

    await vi.waitFor(() => {
      expect(store.importDraft.body).toBe('Previewed body')
    })
    expect(store.importDraft.name).toBe('Manual name')
    expect(store.importDraft.summary).toBe('Manual summary')
    expect(store.importDraft.categoryCode).toBe(306)
  })

  it('shows the first missing import field before importing', async () => {
    const importSpy = vi.spyOn(tauriApi, 'importRuleAsset')

    const store = useRuleStore()
    store.importDraft = {
      sourcePath: 'D:\\temp\\example-rule.md',
      name: '',
      summary: 'Manual summary',
      categoryCode: Number.NaN,
      body: 'Rule body',
      conflictStrategy: 'skip',
    }

    await store.applyImport()

    expect(importSpy).not.toHaveBeenCalled()
    expect(store.actionError).toBe('请输入规则名称。')
  })

  it('shows backend import error details instead of the generic markdown hint', async () => {
    vi.spyOn(tauriApi, 'importRuleAsset').mockResolvedValue({
      success: false,
      error: {
        code: 'rule_import_failed',
        message: 'UNIQUE constraint failed: rule_assets.asset_key',
        i18nKey: 'errors.ruleImportFailed',
      },
    })

    const store = useRuleStore()
    store.importDraft = {
      sourcePath: 'D:\\temp\\example-rule.md',
      name: 'Manual name',
      summary: 'Manual summary',
      categoryCode: 301,
      body: 'Rule body',
      conflictStrategy: 'skip',
    }

    await store.applyImport()

    expect(store.actionError).toBe('UNIQUE constraint failed: rule_assets.asset_key')
  })
})
