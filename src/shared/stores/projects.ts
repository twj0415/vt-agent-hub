import { defineStore } from 'pinia'
import {
  applyProjectOutput,
  cleanupProjectOutput,
  deleteProjectEntity,
  getProjectContextSnapshot,
  importProjectFromGit,
  previewProjectOutput,
  resetProjectOutput,
  repairProjectOutput,
  scanProjectOutput,
  saveProjectEntity,
  saveProjectRuleBindings,
} from '@/shared/api/tauri'
import { useRuleStore } from '@/shared/stores/rules'
import { isTauriRuntime } from '@/shared/utils/runtime'
import { notifyError, notifySuccess, notifyWarning } from '@/shared/utils/notify'
import { translateKey } from '@/shared/i18n/translate'
import { localizeMessage } from '@/shared/utils/message'
import { pathName, repoName } from '@/shared/utils/path'
import { toolIds, type ToolId } from '@/shared/tool-registry'
import { firstIssue, projectDraftSchema } from '@/shared/validation/forms'
import {
  createOutputPreview,
  createOutputScan,
  createOutputWriteResult,
  createProjectDraft,
  createRuleBindingSelection,
  type ProjectDraft,
  type ProjectFormIntent,
  type ProjectImportMode,
  type ProjectItem,
  type ProjectType,
  type RuleBindingSelection,
} from './projects-model'

export function localizeProjectMessage(message: string) {
  return localizeMessage(message)
}

type ProjectRuleBindingSaveResult = 'applied' | 'skipped' | 'failed'

function normalizeProjectPath(value: string) {
  return value.trim().replace(/[\\/]+$/, '').replace(/\\/g, '/').toLowerCase()
}

function joinProjectPath(parent: string, child: string) {
  const base = parent.trim().replace(/[\\/]+$/, '')
  const name = child.trim().replace(/^[\\/]+|[\\/]+$/g, '')
  if (!base) return name
  if (!name) return base
  return `${base}\\${name}`
}

export const useProjectsStore = defineStore('projects', {
  state: () => ({
    items: [] as ProjectItem[],
    activeId: 0,
    detailOpen: false,
    formOpen: false,
    bindOpen: false,
    previewOpen: false,
    listLoading: false,
    snapshotReady: false,
    scanLoading: false,
    previewLoading: false,
    bindLoading: false,
    deleteLoading: false,
    outputAction: 'preview' as 'preview' | 'apply' | 'repair' | 'cleanup' | 'reset',
    formIntent: 'import' as ProjectFormIntent,
    importMode: 'local' as ProjectImportMode,
    importLoading: false,
    draftNameTouched: false,
    draft: createProjectDraft(),
    ruleSearch: '',
    bindingDraft: createRuleBindingSelection() as RuleBindingSelection,
    outputScan: createOutputScan(),
    outputPreview: createOutputPreview(),
    outputResult: createOutputWriteResult(),
    workflowError: '',
  }),
  getters: {
    activeItem(state) {
      return state.items.find((item) => item.id === state.activeId) ?? null
    },
    selectedRuleCount(state) {
      return state.bindingDraft.selectedRuleIds.length
    },
  },
  actions: {
    async hydrateFromSnapshot() {
      this.listLoading = true
      try {
        const response = await getProjectContextSnapshot()
        if (!response.success || !response.data) {
          if (isTauriRuntime()) throw new Error(localizeProjectMessage(response.error?.message ?? 'Workspace snapshot failed.'))
          return
        }

        this.items = response.data.projects.map((project) => ({
          id: project.id,
          name: project.name,
          path: project.path,
          projectType: project.projectType as ProjectType,
          ruleBindings: project.ruleBindings,
          lastOperation: project.lastOperation,
          latestBackup: project.latestBackup,
          outputScan: project.outputScan ?? null,
        }))
        this.activeId = response.data.activeProjectId ?? this.items[0]?.id ?? 0
        this.outputScan = this.activeItem?.outputScan ?? createOutputScan()
      } catch (error) {
        if (isTauriRuntime()) throw error
      } finally {
        this.snapshotReady = true
        this.listLoading = false
      }
    },
    select(id: number) {
      this.activeId = id
      this.outputScan = this.activeItem?.outputScan ?? createOutputScan()
      this.outputPreview = createOutputPreview()
      this.outputResult = createOutputWriteResult()
      this.workflowError = ''
    },
    setDetailOpen(open: boolean) {
      this.detailOpen = open
    },
    openImport(mode: ProjectImportMode = 'local') {
      this.formIntent = 'import'
      this.importMode = mode
      this.draft = createProjectDraft()
      this.draftNameTouched = false
      this.formOpen = true
    },
    openEdit(id: number) {
      const current = this.items.find((item) => item.id === id)
      if (!current) return

      this.formIntent = 'edit'
      this.importMode = 'local'
      this.draft = {
        id: current.id,
        name: current.name,
        path: current.path,
        projectType: current.projectType,
        gitTargetPath: '',
      }
      this.draftNameTouched = true
      this.formOpen = true
    },
    setFormOpen(value: boolean) {
      this.formOpen = value
    },
    setImportMode(mode: ProjectImportMode) {
      if (this.importMode === mode) return
      this.importMode = mode
      if (this.formIntent === 'import') {
        this.draft = createProjectDraft()
        this.draftNameTouched = false
        this.workflowError = ''
      }
    },
    setDraftField<K extends keyof ProjectDraft>(key: K, value: ProjectDraft[K]) {
      this.draft[key] = value
      if (key === 'name') this.draftNameTouched = true
    },
    suggestDraftName(name: string) {
      if (this.draftNameTouched || !name.trim()) return
      this.draft.name = name.trim()
    },
    async saveDraft() {
      const parsed = projectDraftSchema.safeParse(this.draft)
      if (!parsed.success) {
        this.workflowError = localizeProjectMessage(firstIssue(parsed.error, 'Project input is invalid.'))
        notifyWarning(this.workflowError)
        return
      }
      if (this.formIntent === 'import' && this.importMode === 'git' && !this.draft.gitTargetPath.trim()) {
        this.workflowError = translateKey('errors.gitTargetPathRequired')
        notifyWarning(this.workflowError)
        return
      }

      const isGitImport = this.formIntent === 'import' && this.importMode === 'git'
      const name = this.draft.name.trim()
        || (isGitImport
          ? repoName(this.draft.path) || pathName(this.draft.gitTargetPath)
          : pathName(this.draft.path))
        || 'project'
      const projectPath = isGitImport
        ? joinProjectPath(this.draft.gitTargetPath, repoName(this.draft.path) || name)
        : this.draft.path
      const normalizedProjectPath = normalizeProjectPath(projectPath)
      const duplicatedProject = this.items.find((item) => item.id !== this.draft.id && normalizeProjectPath(item.path) === normalizedProjectPath)
      if (duplicatedProject) {
        this.workflowError = localizeProjectMessage(`Project path already exists: ${duplicatedProject.path}`)
        notifyWarning(this.workflowError)
        return
      }

      this.importLoading = true
      try {
        const response = isGitImport
          ? await importProjectFromGit({
            repoUrl: this.draft.path,
            targetPath: projectPath,
            name,
            branch: undefined,
            projectType: this.draft.projectType,
          })
          : await saveProjectEntity({
            id: this.draft.id ?? undefined,
            name,
            path: this.draft.path,
            projectType: this.draft.projectType,
            importMode: true,
          })
        if (!response.success || !response.data) {
          this.workflowError = localizeProjectMessage(response.error?.message ?? 'Project save failed.')
          notifyError(this.workflowError)
          return
        }

        await this.hydrateFromSnapshot()
        this.activeId = response.data.id
        this.workflowError = ''
        notifySuccess(translateKey(this.formIntent === 'edit' ? 'feedback.projectSaved' : 'feedback.projectImported'))
      } catch (error) {
        this.workflowError = localizeProjectMessage(error instanceof Error ? error.message : 'Project save failed.')
        notifyError(this.workflowError)
        return
      } finally {
        this.importLoading = false
      }

      this.formOpen = false
    },
    async deleteItem(id: number) {
      this.deleteLoading = true
      try {
        const response = await deleteProjectEntity(id)
        if (!response.success) {
          this.workflowError = localizeProjectMessage(response.error?.message ?? 'Project delete failed.')
          notifyError(this.workflowError)
          return
        }
        await this.hydrateFromSnapshot()
        this.detailOpen = false
        this.workflowError = ''
        notifySuccess(translateKey('feedback.projectDeleted'))
      } catch (error) {
        this.workflowError = localizeProjectMessage(error instanceof Error ? error.message : 'Project delete failed.')
        notifyError(this.workflowError)
      } finally {
        this.deleteLoading = false
      }
    },
    setBindOpen(value: boolean) {
      this.bindOpen = value
    },
    setPreviewOpen(value: boolean) {
      this.previewOpen = value
    },
    setRuleSearch(value: string) {
      this.ruleSearch = value
    },
    toggleRuleSelection(ruleId: number) {
      if (this.bindingDraft.selectedRuleIds.includes(ruleId)) {
        this.bindingDraft.selectedRuleIds = this.bindingDraft.selectedRuleIds.filter((item) => item !== ruleId)
        return
      }
      this.bindingDraft.selectedRuleIds = [...this.bindingDraft.selectedRuleIds, ruleId]
    },
    openRuleBinding() {
      const current = this.activeItem
      const currentBinding = current?.ruleBindings.find((item) => item.toolId === null) ?? null
      this.bindingDraft.selectedRuleIds = currentBinding?.items
        .filter((item) => item.itemType === 'rule')
        .sort((a, b) => a.sortOrder - b.sortOrder)
        .map((item) => item.assetId) ?? []
      this.ruleSearch = ''
      this.bindOpen = true
    },
    async syncBindingsToProjectFile(projectId: number, toolId: number) {
      const scanResponse = await scanProjectOutput(projectId, toolId)
      if (!scanResponse.success || !scanResponse.data) {
        throw new Error(localizeProjectMessage(scanResponse.error?.message ?? 'Project output scan failed.'))
      }

      this.outputScan = scanResponse.data
      const item = this.items.find((entry) => entry.id === projectId)
      if (item) item.outputScan = scanResponse.data
      const hasRules = scanResponse.data.ruleCount > 0
      const hasManagedTarget = scanResponse.data.targetExists && scanResponse.data.managed
      if (!hasRules && !hasManagedTarget) return 'skipped'

      const writeResponse = await applyProjectOutput(projectId, toolId, true)
      if (!writeResponse.success || !writeResponse.data) {
        throw new Error(localizeProjectMessage(writeResponse.error?.message ?? 'Project output write failed.'))
      }

      this.outputResult = writeResponse.data
      return 'applied'
    },
    async saveProjectRuleIdsAndSync(
      projectId: number,
      ruleIds: number[],
      activeToolId: ToolId = toolIds.codex,
      options: {
        closeBind?: boolean
        notify?: boolean
        refreshRules?: boolean
        refreshSnapshot?: boolean
        requireSelection?: boolean
        scanActiveProject?: boolean
        successAppliedKey?: string
        successSkippedKey?: string
      } = {},
    ): Promise<ProjectRuleBindingSaveResult> {
      const current = this.items.find((item) => item.id === projectId)
      if (!current) return 'failed'
      if (options.requireSelection && !ruleIds.length) {
        this.workflowError = translateKey('errors.projectRuleSelectionRequired')
        notifyWarning(this.workflowError)
        return 'failed'
      }

      const previousRuleIds = current.ruleBindings
        .find((item) => item.toolId == null)
        ?.items.filter((item) => item.itemType === 'rule')
        .sort((a, b) => a.sortOrder - b.sortOrder)
        .map((item) => item.assetId) ?? []

      this.bindLoading = true
      try {
        const response = await saveProjectRuleBindings(projectId, null, ruleIds)
        if (!response.success) {
          this.workflowError = localizeProjectMessage(response.error?.message ?? 'Project rule bindings failed.')
          notifyError(this.workflowError)
          return 'failed'
        }

        let syncResult: Exclude<ProjectRuleBindingSaveResult, 'failed'>
        try {
          syncResult = await this.syncBindingsToProjectFile(projectId, activeToolId)
        } catch (error) {
          const syncMessage = localizeProjectMessage(error instanceof Error ? error.message : 'Project output write failed.')
          const rollbackResponse = await saveProjectRuleBindings(projectId, null, previousRuleIds)
          if (options.refreshSnapshot !== false) await this.hydrateFromSnapshot()
          if (options.refreshRules) await useRuleStore().hydrateFromSnapshot()
          if (!rollbackResponse.success) {
            const rollbackMessage = localizeProjectMessage(rollbackResponse.error?.message ?? 'Project rule binding rollback failed.')
            this.workflowError = `${syncMessage} ${rollbackMessage}`
          } else {
            this.workflowError = syncMessage
          }
          notifyError(this.workflowError)
          return 'failed'
        }

        if (options.refreshSnapshot !== false) await this.hydrateFromSnapshot()
        if (options.scanActiveProject !== false && this.activeId === projectId) {
          await this.scanOutput(activeToolId, { silent: true })
        }
        if (options.refreshRules) await useRuleStore().hydrateFromSnapshot()
        if (options.closeBind) this.bindOpen = false
        this.workflowError = ''
        if (options.notify) {
          notifySuccess(translateKey(
            syncResult === 'applied'
              ? options.successAppliedKey ?? 'feedback.projectRulesSavedApplied'
              : options.successSkippedKey ?? 'feedback.projectRulesSavedSkipped',
          ))
        }
        return syncResult
      } catch (error) {
        this.workflowError = localizeProjectMessage(error instanceof Error ? error.message : 'Project rule bindings failed.')
        notifyError(this.workflowError)
        return 'failed'
      } finally {
        this.bindLoading = false
      }
    },
    async applyRuleBinding(activeToolId: ToolId = toolIds.codex) {
      const current = this.activeItem
      if (!current) return

      await this.saveProjectRuleIdsAndSync(current.id, this.bindingDraft.selectedRuleIds, activeToolId, {
        closeBind: true,
        notify: true,
        refreshRules: true,
        requireSelection: true,
      })
    },
    async unbindRule(ruleId: number, activeToolId: ToolId = toolIds.codex) {
      const current = this.activeItem
      if (!current) return

      const binding = current.ruleBindings.find((item) => item.toolId == null)
      if (!binding) return

      const nextRuleIds = binding.items
        .filter((item) => item.itemType === 'rule' && item.assetId !== ruleId)
        .map((item) => item.assetId)

      await this.saveProjectRuleIdsAndSync(current.id, nextRuleIds, activeToolId, {
        notify: true,
        refreshRules: true,
        successAppliedKey: 'feedback.projectRuleUnboundApplied',
        successSkippedKey: 'feedback.projectRuleUnboundSkipped',
      })
    },
    async scanOutput(toolId: ToolId, options: { silent?: boolean } = {}) {
      const current = this.activeItem
      if (!current) return

      this.scanLoading = true
      try {
        const response = await scanProjectOutput(current.id, toolId)
        if (!response.success || !response.data) {
          this.workflowError = localizeProjectMessage(response.error?.message ?? 'Project output scan failed.')
          notifyError(this.workflowError)
          return
        }

        this.outputScan = response.data
        const item = this.items.find((entry) => entry.id === current.id)
        if (item) item.outputScan = response.data
        this.workflowError = ''
        if (!options.silent) {
          notifySuccess(translateKey('feedback.projectScanCompleted'))
        }
      } catch (error) {
        this.workflowError = localizeProjectMessage(error instanceof Error ? error.message : 'Project output scan failed.')
        notifyError(this.workflowError)
      } finally {
        this.scanLoading = false
      }
    },
    async loadOutputPreview(toolId: ToolId) {
      const current = this.activeItem
      if (!current) return false

      this.previewLoading = true
      try {
        const response = await previewProjectOutput(current.id, toolId)
        if (!response.success || !response.data) {
          this.workflowError = localizeProjectMessage(response.error?.message ?? 'Project output preview failed.')
          notifyError(this.workflowError)
          return false
        }

        this.outputPreview = response.data
        this.outputResult = createOutputWriteResult()
        this.outputAction = 'preview'
        this.workflowError = ''
        await this.scanOutput(toolId, { silent: true })
        return true
      } catch (error) {
        this.workflowError = localizeProjectMessage(error instanceof Error ? error.message : 'Project output preview failed.')
        notifyError(this.workflowError)
        return false
      } finally {
        this.previewLoading = false
      }
    },
    async openOutputPreview(toolId: ToolId, action: 'preview' | 'apply' | 'repair' = 'preview') {
      const loaded = await this.loadOutputPreview(toolId)
      if (!loaded) return

      this.outputAction = action
      this.previewOpen = true
    },
    async openOutputRemoval(toolId: ToolId, action: 'cleanup' | 'reset') {
      const current = this.activeItem
      if (!current) return

      this.previewLoading = true
      await this.scanOutput(toolId, { silent: true })
      if (this.workflowError) {
        this.previewLoading = false
        return
      }

      this.outputPreview = createOutputPreview()
      this.outputResult = createOutputWriteResult()
      this.outputAction = action
      this.previewOpen = true
      this.workflowError = ''
      this.previewLoading = false
    },
    async confirmOutput(toolId: ToolId) {
      const current = this.activeItem
      if (!current) return

      if (this.outputAction === 'preview') {
        this.previewOpen = false
        this.workflowError = ''
        return
      }

      this.previewLoading = true
      try {
        const response = this.outputAction === 'repair'
          ? await repairProjectOutput(current.id, toolId, true)
          : this.outputAction === 'cleanup'
            ? await cleanupProjectOutput(current.id, toolId, true)
            : this.outputAction === 'reset'
              ? await resetProjectOutput(current.id, toolId, true)
              : await applyProjectOutput(current.id, toolId, true)
        if (!response.success || !response.data) {
          this.workflowError = localizeProjectMessage(response.error?.message ?? 'Project output write failed.')
          notifyError(this.workflowError)
          return
        }

        this.outputResult = response.data
        this.previewOpen = false
        this.workflowError = ''
        await this.hydrateFromSnapshot()
        await this.scanOutput(toolId, { silent: true })
        notifySuccess(localizeProjectMessage(response.data.message))
      } catch (error) {
        this.workflowError = localizeProjectMessage(error instanceof Error ? error.message : 'Project output write failed.')
        notifyError(this.workflowError)
      } finally {
        this.previewLoading = false
      }
    },
  },
})
