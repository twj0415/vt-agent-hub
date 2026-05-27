import { defineStore } from 'pinia'
import type {
  BackupActionResult,
  BackupEntry,
  BackupRestorePreview,
  DiagnosticExportResult,
  LibraryDiagnostics,
  HistorySnapshot,
} from '@/shared/api/client'
import type { MatrixRow } from '@/shared/types/ui'
import {
  deleteBackup,
  exportLibraryDiagnostics,
  getBackupSnapshot,
  getHistorySnapshot,
  previewBackupRestore,
  restoreBackup,
  scanLibraryDiagnostics,
} from '@/shared/api/tauri'
import { isTauriRuntime } from '@/shared/utils/runtime'
import { notifyError, notifySuccess } from '@/shared/utils/notify'

type HistoryLevel = 'healthy' | 'attention' | 'risk'
type HistoryKind = 'operation' | 'backup' | 'repair' | 'diagnostic'

export type HistoryItem = {
  id: number
  projectId?: number | null
  toolId?: number | null
  relatedRuleId?: number | null
  kind: HistoryKind
  title: string
  createdAt: string
  action: string
  result: string
  level: HistoryLevel
  detail: string
  relatedPath: string
  navigationTarget: string
}

function toRows(items: HistoryItem[]): MatrixRow[] {
  return items.map((item) => ({
    label: item.title,
    value: item.createdAt,
    tone:
      item.level === 'healthy'
        ? 'ready'
        : item.level === 'attention'
          ? 'warning'
          : 'error',
    badgeKey:
      item.level === 'healthy'
        ? 'common.ready'
        : item.level === 'attention'
          ? 'common.warning'
          : 'common.error',
  }))
}

export const useHistoryStore = defineStore('history', {
  state: () => ({
    items: [] as HistoryItem[],
    filters: {
      projectIds: [] as number[],
      toolIds: [] as number[],
      kinds: [] as string[],
      results: [] as string[],
    },
    activeProjectFilter: 'all' as 'all' | number,
    activeToolFilter: 'all' as 'all' | number,
    activeKindFilter: 'all' as 'all' | string,
    activeResultFilter: 'all' as 'all' | string,
    backupEntries: [] as BackupEntry[],
    libraryDiagnostics: null as LibraryDiagnostics | null,
    restorePreview: null as BackupRestorePreview | null,
    lastBackupAction: null as BackupActionResult | null,
    lastExportResult: null as DiagnosticExportResult | null,
    backupError: '',
    diagnosticsError: '',
    previewOpen: false,
    detailOpen: false,
    activeHistoryId: null as number | null,
  }),
  getters: {
    operations(state) {
      return state.items.filter((item) => item.kind === 'operation')
    },
    backups(state) {
      return state.items.filter((item) => item.kind === 'backup')
    },
    repairs(state) {
      return state.items.filter((item) => item.kind === 'repair')
    },
    diagnostics(state) {
      return state.items.filter((item) => item.kind === 'diagnostic')
    },
    summaryRows(state): MatrixRow[] {
      return toRows(this.filteredItems)
    },
    operationRows(): MatrixRow[] {
      return toRows(this.operations)
    },
    backupRows(): MatrixRow[] {
      return toRows(this.backups)
    },
    repairRows(): MatrixRow[] {
      return toRows(this.repairs)
    },
    diagnosticRows(): MatrixRow[] {
      return toRows(this.diagnostics)
    },
    backupDetailRows(state): MatrixRow[] {
      return state.backupEntries.map((entry) => ({
        label: entry.fileName,
        value: `${entry.scope} -> ${entry.targetPath}`,
        tone: entry.scope === 'project' ? 'ready' : 'warning',
        badgeLabel: entry.scope,
      }))
    },
    filteredItems(): HistoryItem[] {
      return this.items.filter((item) => {
        const matchProject = this.activeProjectFilter === 'all' || item.projectId === this.activeProjectFilter
        const matchTool = this.activeToolFilter === 'all' || item.toolId === this.activeToolFilter
        const matchKind = this.activeKindFilter === 'all' || item.kind === this.activeKindFilter
        const matchResult = this.activeResultFilter === 'all' || item.result === this.activeResultFilter
        return matchProject && matchTool && matchKind && matchResult
      })
    },
    activeItem(state): HistoryItem | null {
      return state.items.find((item) => item.id === state.activeHistoryId) ?? null
    },
  },
  actions: {
    async hydrateFromSnapshot() {
      try {
        const response = await getHistorySnapshot()
        if (!response.success || !response.data) {
          if (isTauriRuntime()) throw new Error(response.error?.message ?? 'History snapshot failed.')
          return
        }

        this.items = response.data.entries.map((entry) => ({
          id: entry.id,
          projectId: entry.projectId ?? null,
          toolId: entry.toolId ?? null,
          relatedRuleId: entry.relatedRuleId ?? null,
          kind: entry.kind as HistoryKind,
          title: entry.title,
          createdAt: entry.createdAt,
          action: entry.action,
          result: entry.result,
          level:
            entry.level === 'healthy'
              ? 'healthy'
              : entry.level === 'attention'
                ? 'attention'
                : 'risk',
          detail: entry.detail,
          relatedPath: entry.relatedPath,
          navigationTarget: entry.navigationTarget,
        }))
        this.filters = response.data.filters
      } catch (error) {
        if (isTauriRuntime()) throw error
      }

      try {
        const backupSnapshot = await getBackupSnapshot()
        if (!backupSnapshot.success || !backupSnapshot.data) {
          if (isTauriRuntime()) throw new Error(backupSnapshot.error?.message ?? 'Backup snapshot failed.')
        } else {
          this.backupEntries = backupSnapshot.data.entries
          this.backupError = ''
        }
      } catch (error) {
        this.backupError = error instanceof Error ? error.message : 'Backup snapshot failed.'
        if (isTauriRuntime()) throw error
      }

      try {
        const diagnostics = await scanLibraryDiagnostics()
        if (diagnostics.success && diagnostics.data) {
          this.libraryDiagnostics = diagnostics.data
          this.diagnosticsError = ''
        } else if (isTauriRuntime()) {
          throw new Error(diagnostics.error?.message ?? 'Library diagnostics failed.')
        }
      } catch (error) {
        this.diagnosticsError = error instanceof Error ? error.message : 'Library diagnostics failed.'
        if (isTauriRuntime()) throw error
      }
    },
    setPreviewOpen(value: boolean) {
      this.previewOpen = value
    },
    setDetailOpen(value: boolean) {
      this.detailOpen = value
    },
    openHistoryDetail(id: number) {
      this.activeHistoryId = id
      this.detailOpen = true
    },
    async openRestorePreview(backupId: string) {
      try {
        const response = await previewBackupRestore(backupId)
        if (!response.success || !response.data) {
          this.backupError = response.error?.message ?? 'Backup restore preview failed.'
          notifyError(this.backupError)
          return
        }

        this.restorePreview = response.data
        this.previewOpen = true
        this.backupError = ''
      } catch (error) {
        this.backupError = error instanceof Error ? error.message : 'Backup restore preview failed.'
        notifyError(this.backupError)
      }
    },
    async confirmRestore(confirmRisk = true) {
      if (!this.restorePreview) return

      try {
        const response = await restoreBackup(this.restorePreview.backupId, confirmRisk)
        if (!response.success || !response.data) {
          this.backupError = response.error?.message ?? 'Backup restore failed.'
          notifyError(this.backupError)
          return
        }

        this.lastBackupAction = response.data
        this.previewOpen = false
        this.backupError = response.data.message
        notifySuccess(this.backupError)
        await this.hydrateFromSnapshot()
      } catch (error) {
        this.backupError = error instanceof Error ? error.message : 'Backup restore failed.'
        notifyError(this.backupError)
      }
    },
    async removeBackup(backupId: string) {
      try {
        const response = await deleteBackup(backupId)
        if (!response.success || !response.data) {
          this.backupError = response.error?.message ?? 'Backup delete failed.'
          notifyError(this.backupError)
          return
        }

        this.lastBackupAction = response.data
        this.backupError = response.data.message
        notifySuccess(this.backupError)
        await this.hydrateFromSnapshot()
      } catch (error) {
        this.backupError = error instanceof Error ? error.message : 'Backup delete failed.'
        notifyError(this.backupError)
      }
    },
    async exportDiagnostics() {
      try {
        const response = await exportLibraryDiagnostics()
        if (!response.success || !response.data) {
          this.diagnosticsError = response.error?.message ?? 'Diagnostics export failed.'
          notifyError(this.diagnosticsError)
          return
        }

        this.lastExportResult = response.data
        this.diagnosticsError = response.data.message
        notifySuccess(`${response.data.message} ${response.data.path}`)
        await this.hydrateFromSnapshot()
      } catch (error) {
        this.diagnosticsError = error instanceof Error ? error.message : 'Diagnostics export failed.'
        notifyError(this.diagnosticsError)
      }
    },
    setProjectFilter(value: 'all' | number) {
      this.activeProjectFilter = value
    },
    setToolFilter(value: 'all' | number) {
      this.activeToolFilter = value
    },
    setKindFilter(value: 'all' | string) {
      this.activeKindFilter = value
    },
    setResultFilter(value: 'all' | string) {
      this.activeResultFilter = value
    },
  },
})
