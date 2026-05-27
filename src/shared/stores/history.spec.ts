import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useHistoryStore } from './history'
import * as tauriApi from '@/shared/api/tauri'

describe('history store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.restoreAllMocks()
  })

  it('splits read-only history into operation, backup, repair, and diagnostic groups', () => {
    const store = useHistoryStore()

    expect(store.operations).toHaveLength(0)
    expect(store.backups).toHaveLength(0)
    expect(store.repairs).toHaveLength(0)
    expect(store.diagnostics).toHaveLength(0)
    expect(store.summaryRows).toHaveLength(0)
  })

  it('opens restore preview, restores backups, and exports diagnostics through backend commands', async () => {
    vi.spyOn(tauriApi, 'previewBackupRestore').mockResolvedValue({
      success: true,
      data: {
        backupId: 'backup-1',
        backupPath: 'C:\\Users\\Example\\.vt-agent-hub\\backups\\project-1\\AGENTS.md',
        targetPath: 'C:\\repo\\AGENTS.md',
        targetExists: true,
        beforeContent: 'before',
        afterContent: 'after',
        diff: 'diff',
        warning: 'Restoring overwrites the live target after confirmation.',
      },
    })
    vi.spyOn(tauriApi, 'restoreBackup').mockResolvedValue({
      success: true,
      data: {
        ok: true,
        message: 'Backup restored.',
        path: 'C:\\repo\\AGENTS.md',
      },
    })
    vi.spyOn(tauriApi, 'exportLibraryDiagnostics').mockResolvedValue({
      success: true,
      data: {
        path: 'C:\\Users\\Example\\.vt-agent-hub\\snapshots\\library-diagnostics.json',
        issueCount: 0,
        message: 'Diagnostics exported.',
      },
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
    vi.spyOn(tauriApi, 'getBackupSnapshot').mockResolvedValue({
      success: true,
      data: {
        entries: [
          {
            id: 'backup-1',
            scope: 'project',
            projectId: 1,
            fileName: 'AGENTS.md',
            path: 'C:\\Users\\Example\\.vt-agent-hub\\backups\\project-1\\AGENTS.md',
            targetPath: 'C:\\repo\\AGENTS.md',
            createdAt: '1710000000',
            size: 128,
          },
        ],
      },
    })
    vi.spyOn(tauriApi, 'scanLibraryDiagnostics').mockResolvedValue({
      success: true,
      data: {
        projectCount: 1,
        ruleCount: 2,
        skillCount: 1,
        libraryRoot: 'C:\\Users\\Example\\.vt-agent-hub\\library',
        createdPaths: [],
        existingPaths: [],
        issueCount: 0,
        healthState: 'normal',
        healthStateCode: 701,
        issues: [],
      },
    })

    const store = useHistoryStore()

    await store.openRestorePreview('backup-1')
    expect(store.previewOpen).toBe(true)
    expect(store.restorePreview?.backupId).toBe('backup-1')

    await store.confirmRestore(true)
    expect(store.lastBackupAction?.message).toBe('Backup restored.')
    expect(store.previewOpen).toBe(false)

    await store.exportDiagnostics()
    expect(store.lastExportResult?.message).toBe('Diagnostics exported.')
  })
})
