import { defineStore } from 'pinia'
import type { MatrixRow } from '@/shared/types/ui'
import { storageKeys } from '@/shared/constants/storage'
import { getProjectContextSnapshot } from '@/shared/api/tauri'
import { getToolById, toolIds, type ToolId } from '@/shared/tool-registry'
import { entityStateBadgeKey, entityStateTone } from '@/shared/constants/status'
import { translateKey } from '@/shared/i18n/translate'
import { isTauriRuntime } from '@/shared/utils/runtime'
import { useProjectsStore } from './projects'
import type { ProjectOutputPreview, ProjectOutputScan } from '@/shared/api/client'

function scanStatusTone(status?: string): MatrixRow['tone'] {
  return status === 'missing' ? 'error' : 'ready'
}

function scanStatusBadgeKey(scan: ProjectOutputScan | null): string {
  return scan?.issues.includes('project_path_missing') ? 'common.abnormal' : 'common.normal'
}

function scanStatusLabel(scan: ProjectOutputScan | null): string {
  return scan?.issues.includes('project_path_missing') ? translateKey('common.abnormal') : translateKey('common.normal')
}

export const useToolContextStore = defineStore('tool-context', {
  state: () => ({
    activeProjectId: null as number | null,
    activeToolId: toolIds.codex as ToolId,
    activeAction: 'preview' as 'scan' | 'preview' | 'apply' | 'repair' | 'cleanup' | 'reset',
    outputPreview: null as ProjectOutputPreview | null,
    outputScan: null as ProjectOutputScan | null,
    outputStatus: '' as string,
  }),
  getters: {
    summaryRows(state): MatrixRow[] {
      const activeTool = getToolById(state.activeToolId)

      return [
        {
          label: translateKey('context.rows.tool'),
          value: activeTool ? `${activeTool.id} = ${activeTool.key}` : String(state.activeToolId),
          tone: entityStateTone(activeTool?.status ?? 'planned'),
          badgeLabel: activeTool ? activeTool.key.toUpperCase() : String(state.activeToolId),
        },
        {
          label: translateKey('context.rows.project'),
          value: state.activeProjectId ? String(state.activeProjectId) : translateKey('common.pending'),
          tone: 'planned',
          badgeKey: 'common.planned',
        },
        { label: translateKey('context.rows.flow'), value: translateKey('context.flowValue') },
        {
          label: translateKey('context.rows.action'),
          value: state.activeAction.toUpperCase(),
          tone: state.activeAction === 'apply' || state.activeAction === 'repair' || state.activeAction === 'cleanup' || state.activeAction === 'reset' ? 'warning' : 'planned',
          badgeKey: state.activeAction === 'apply' || state.activeAction === 'repair' || state.activeAction === 'cleanup' || state.activeAction === 'reset' ? 'common.warning' : 'common.planned',
        },
      ]
    },
    projectRows(state): MatrixRow[] {
      const project = useProjectsStore().items.find((item) => item.id === state.activeProjectId)
      if (!project) return []

      return [
        { label: translateKey('context.rows.projectName'), value: project.name },
        { label: translateKey('common.path'), value: project.path },
        { label: translateKey('context.rows.rules'), value: String(project.ruleBindings.length) },
      ]
    },
    presetRows(): MatrixRow[] {
      return []
    },
    ruleRows(state): MatrixRow[] {
      const project = useProjectsStore().items.find((item) => item.id === state.activeProjectId)
      if (!project) return []

      return project.ruleBindings.map((binding) => {
        return {
          label: binding.packName,
          value: `${binding.packType} v${binding.packVersionNo}`,
          tone: binding.enabled ? 'ready' : 'planned',
          badgeKey: binding.enabled ? 'common.ready' : 'common.planned',
        }
      })
    },
    previewRows(): MatrixRow[] {
      const preview = this.outputPreview
      const scan = this.outputScan
      return [
        { label: translateKey('context.rows.targetPath'), value: scan?.targetPath ?? preview?.targetPath ?? translateKey('feedback.noProjectScanYet') },
        { label: translateKey('context.rows.scanStatus'), value: scanStatusLabel(scan), tone: scanStatusTone(scan?.status), badgeKey: scanStatusBadgeKey(scan) },
        {
          label: translateKey('context.rows.managed'),
          value: scan ? (scan.managed ? translateKey('common.yes') : translateKey('common.no')) : preview ? (preview.managed ? translateKey('common.yes') : translateKey('common.no')) : translateKey('common.pending'),
          tone: scan ? (scan.managed ? 'ready' : 'warning') : preview ? (preview.managed ? 'ready' : 'warning') : 'planned',
          badgeKey: scan ? (scan.managed ? 'common.ready' : 'common.warning') : preview ? (preview.managed ? 'common.ready' : 'common.warning') : 'common.planned',
        },
        { label: translateKey('context.rows.ruleCount'), value: scan ? String(scan.ruleCount) : preview ? String(preview.ruleCount) : '0' },
        { label: translateKey('context.rows.statusNote'), value: this.outputStatus || translateKey('feedback.noActionYet') },
      ]
    },
    recentRows(state): MatrixRow[] {
      const project = useProjectsStore().items.find((item) => item.id === state.activeProjectId)
      if (!project) return []

      return [
        { label: translateKey('context.rows.lastOperation'), value: project.lastOperation },
        { label: translateKey('context.rows.latestBackup'), value: project.latestBackup },
      ]
    },
  },
  actions: {
    loadPersistedTool() {
      if (typeof window === 'undefined') return
      const raw = window.localStorage.getItem(storageKeys.activeToolId)
      const parsed = raw ? Number(raw) : NaN
      const tool = getToolById(parsed)
      if (tool?.enabled) {
        this.activeToolId = tool.id
        return true
      }
      return false
    },
    persistActiveTool() {
      if (typeof window === 'undefined') return
      window.localStorage.setItem(storageKeys.activeToolId, String(this.activeToolId))
    },
    async hydrateFromSnapshot() {
      try {
        const response = await getProjectContextSnapshot()
        if (!response.success || !response.data) {
          if (isTauriRuntime()) throw new Error(response.error?.message ?? translateKey('errors.projectContextSnapshotFailed'))
          return
        }

        this.activeProjectId = response.data.activeProjectId ?? response.data.projects[0]?.id ?? null
        const snapshotTool = getToolById(response.data.activeToolId)
        if (!getToolById(this.activeToolId)?.enabled && snapshotTool?.enabled) {
          this.activeToolId = snapshotTool.id
        }
        this.activeAction = 'preview'
        useProjectsStore().items = response.data.projects.map((project) => ({
          id: project.id,
          name: project.name,
          path: project.path,
          projectType: project.projectType as 201 | 202 | 203,
          ruleBindings: project.ruleBindings,
          lastOperation: project.lastOperation,
          latestBackup: project.latestBackup,
          outputScan: project.outputScan ?? null,
        }))
      } catch (error) {
        if (isTauriRuntime()) throw error
      }
    },
    setActiveProject(id: number | null) {
      this.activeProjectId = id
    },
    setActiveTool(id: ToolId) {
      const tool = getToolById(id)
      if (!tool?.enabled) return
      this.activeToolId = tool.id
      this.persistActiveTool()
    },
    setActiveAction(action: 'scan' | 'preview' | 'apply' | 'repair' | 'cleanup' | 'reset') {
      this.activeAction = action
    },
    setOutputPreview(preview: ProjectOutputPreview | null) {
      this.outputPreview = preview
    },
    setOutputScan(scan: ProjectOutputScan | null) {
      this.outputScan = scan
    },
    setOutputStatus(value: string) {
      this.outputStatus = value
    },
  },
})
