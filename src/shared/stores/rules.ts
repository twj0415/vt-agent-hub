import { defineStore } from 'pinia'
import {
  deleteRuleAsset,
  getLibrarySnapshot,
  importRuleAsset,
  moveRuleAsset,
  previewRuleImport,
  previewRuleImpact,
  saveRuleAsset,
} from '@/shared/api/tauri'
import { entityStateFromCode, entityStateToCode, type EntityState } from '@/shared/constants/status'
import type { RuleImpact } from '@/shared/api/client'
import { resolveAppError, resolveUnknownError, translateKey } from '@/shared/i18n/translate'
import { isTauriRuntime } from '@/shared/utils/runtime'
import { notifyError, notifySuccess, notifyWarning } from '@/shared/utils/notify'
import { firstIssue, ruleDraftSchema } from '@/shared/validation/forms'
import { defaultRuleCategoryCode, ruleCategoryOptions } from '@/shared/taxonomy'
import { toolIds, type ToolId } from '@/shared/tool-registry'
import { useProjectsStore } from './projects'
import { useToolContextStore } from './tool-context'
import { useToolsStore } from './tools'

type RuleSort = 'order' | 'name' | 'code'

export type RuleItem = {
  id: number
  versionId: number
  versionNo: number
  key: string
  name: string
  code: number
  state: EntityState
  sortOrder: number
  summary: string
  categoryCode: number
  body: string
  impact: RuleImpact | null
}

type RuleDraft = {
  id: number | null
  name: string
  code: number
  state: EntityState
  sortOrder: number
  summary: string
  categoryCode: number
  body: string
}

type RuleBindMode = 'project' | 'tool'

type RuleBindDraft = {
  ruleId: number | null
  mode: RuleBindMode
  selectedProjectIds: number[]
  selectedToolIds: number[]
}

type RuleBindSaveResult = 'saved' | 'needsToolOverwriteConfirm' | 'failed'

const ruleCategoryCodes = ruleCategoryOptions.map((item) => item.value)

function isValidRuleCategoryCode(value: unknown): value is number {
  return typeof value === 'number' && Number.isInteger(value) && ruleCategoryCodes.includes(value as never)
}

function ruleImportValidationError(draft: {
  sourcePath: string
  name: string
  summary: string
  categoryCode: number | null
  body: string
}) {
  if (!draft.sourcePath.trim()) return translateKey('errors.ruleImportFileRequired')
  if (!draft.name.trim()) return translateKey('errors.ruleNameRequired')
  if (!isValidRuleCategoryCode(draft.categoryCode)) return translateKey('errors.ruleCategoryRequired')
  if (!draft.summary.trim()) return translateKey('errors.ruleDescriptionRequired')
  if (!draft.body.trim()) return translateKey('errors.ruleBodyRequired')
  return ''
}

function sortItems(items: RuleItem[], sort: RuleSort) {
  return [...items].sort((a, b) => {
    if (sort === 'order') return a.categoryCode - b.categoryCode || a.sortOrder - b.sortOrder || a.id - b.id
    if (sort === 'code') return a.code - b.code
    return a.name.localeCompare(b.name)
  })
}

function createEmptyDraft(): RuleDraft {
  return {
    id: null,
    name: '',
    code: defaultRuleCategoryCode,
    state: 'planned',
    sortOrder: 0,
    summary: '',
    categoryCode: defaultRuleCategoryCode,
    body: '',
  }
}

function createEmptyBindDraft(): RuleBindDraft {
  return {
    ruleId: null,
    mode: 'project',
    selectedProjectIds: [],
    selectedToolIds: [],
  }
}

export const useRuleStore = defineStore('rules', {
  state: () => ({
    items: [] as RuleItem[],
    activeId: 0,
    search: '',
    filter: 'all' as 'all' | number,
    sort: 'order' as RuleSort,
    detailOpen: false,
    formOpen: false,
    importOpen: false,
    bindOpen: false,
    importLoading: false,
    bindLoading: false,
    draft: createEmptyDraft(),
    bindDraft: createEmptyBindDraft() as RuleBindDraft,
    importDraft: {
      sourcePath: '',
      name: '',
      summary: '',
      body: '',
      categoryCode: null as number | null,
      conflictStrategy: 'skip' as 'skip' | 'rename' | 'overwrite',
    },
    importTouched: {
      name: false,
      summary: false,
      categoryCode: false,
    },
    actionError: '',
    lastImpact: null as RuleImpact | null,
    lastImportReport: '',
  }),
  getters: {
    activeItem(state) {
      return state.items.find((item) => item.id === state.activeId) ?? null
    },
    filteredItems(state) {
      const keyword = state.search.trim().toLowerCase()
      const filtered = state.items.filter((item) => {
        const matchesSearch = !keyword
          || item.name.toLowerCase().includes(keyword)
          || item.key.toLowerCase().includes(keyword)
          || item.summary.toLowerCase().includes(keyword)
          || String(item.code).includes(keyword)
        const matchesFilter = state.filter === 'all' || item.categoryCode === state.filter
        return matchesSearch && matchesFilter
      })

      return sortItems(filtered, state.sort)
    },
  },
  actions: {
    async hydrateFromSnapshot() {
      try {
        const response = await getLibrarySnapshot()
        if (!response.success || !response.data) {
          if (isTauriRuntime()) throw new Error(resolveAppError(response.error, 'errors.librarySnapshotFailed'))
          return
        }

        this.items = response.data.rules.map((rule) => ({
          id: rule.assetId,
          versionId: rule.versionId,
          versionNo: rule.versionNo,
          key: rule.key,
          name: rule.name,
          code: rule.code,
          state: entityStateFromCode(rule.state),
          sortOrder: rule.sortOrder,
          summary: rule.summary,
          categoryCode: rule.categoryCode,
          body: rule.body,
          impact: null,
        }))
        this.activeId = this.items.find((item) => item.id === this.activeId)?.id ?? this.items[0]?.id ?? 0
        await Promise.all(this.items.map(async (item) => {
          const impact = await previewRuleImpact(item.id)
          if (impact.success && impact.data) {
            const current = this.items.find((entry) => entry.id === item.id)
            if (current) current.impact = impact.data
          }
        }))
      } catch (error) {
        if (isTauriRuntime()) throw error
      }
    },
    select(id: number) {
      this.activeId = id
    },
    setSearch(value: string) {
      this.search = value
    },
    setFilter(value: 'all' | number) {
      this.filter = value
    },
    setSort(value: RuleSort) {
      this.sort = value
    },
    setDetailOpen(value: boolean) {
      this.detailOpen = value
    },
    openCreate() {
      this.draft = createEmptyDraft()
      this.formOpen = true
    },
    openEdit(id: number) {
      const current = this.items.find((item) => item.id === id)
      if (!current) return

      this.draft = { ...current }
      this.formOpen = true
    },
    setFormOpen(value: boolean) {
      this.formOpen = value
    },
    setBindOpen(value: boolean) {
      this.bindOpen = value
      if (!value) {
        this.bindDraft = createEmptyBindDraft()
      }
    },
    setDraftField<K extends keyof RuleDraft>(key: K, value: RuleDraft[K]) {
      this.draft[key] = value
      if (key === 'categoryCode') {
        this.draft.code = value as number
      }
    },
    openProjectBinding(ruleId: number) {
      const projectsStore = useProjectsStore()
      this.bindDraft = {
        ruleId,
        mode: 'project',
        selectedProjectIds: projectsStore.items
          .filter((project) =>
            project.ruleBindings.some((binding) =>
              binding.toolId == null && binding.items.some((item) => item.itemType === 'rule' && item.assetId === ruleId),
            ))
          .map((project) => project.id),
        selectedToolIds: [],
      }
      this.bindOpen = true
    },
    openToolBinding(ruleId: number) {
      const toolsStore = useToolsStore()
      const boundRuleIds = toolsStore.globalRuleBinding?.items
        .filter((item) => item.itemType === 'rule')
        .map((item) => item.assetId) ?? []
      this.bindDraft = {
        ruleId,
        mode: 'tool',
        selectedProjectIds: [],
        selectedToolIds: boundRuleIds.includes(ruleId) ? [toolsStore.activeId] : [],
      }
      this.bindOpen = true
    },
    toggleBindProject(projectId: number) {
      if (this.bindDraft.selectedProjectIds.includes(projectId)) {
        this.bindDraft.selectedProjectIds = this.bindDraft.selectedProjectIds.filter((id) => id !== projectId)
        return
      }
      this.bindDraft.selectedProjectIds = [...this.bindDraft.selectedProjectIds, projectId]
    },
    toggleBindTool(toolId: number) {
      if (this.bindDraft.selectedToolIds.includes(toolId)) {
        this.bindDraft.selectedToolIds = this.bindDraft.selectedToolIds.filter((id) => id !== toolId)
        return
      }
      this.bindDraft.selectedToolIds = [...this.bindDraft.selectedToolIds, toolId]
    },
    async saveBindingDraft(options: { forceToolOverwrite?: boolean } = {}): Promise<RuleBindSaveResult> {
      const ruleId = this.bindDraft.ruleId
      if (ruleId == null) return 'failed'

      this.bindLoading = true
      try {
        if (this.bindDraft.mode === 'project') {
          const projectsStore = useProjectsStore()
          const activeToolId = useToolContextStore().activeToolId as ToolId
          let changed = false
          for (const project of projectsStore.items) {
            const currentRuleIds = project.ruleBindings
              .find((binding) => binding.toolId == null)
              ?.items.filter((item) => item.itemType === 'rule')
              .map((item) => item.assetId) ?? []
            const exists = currentRuleIds.includes(ruleId)
            const checked = this.bindDraft.selectedProjectIds.includes(project.id)
            let nextRuleIds = currentRuleIds
            if (checked && !exists) nextRuleIds = [...currentRuleIds, ruleId]
            if (!checked && exists) nextRuleIds = currentRuleIds.filter((id) => id !== ruleId)
            if (nextRuleIds.length === currentRuleIds.length && nextRuleIds.every((id, index) => id === currentRuleIds[index])) continue

            const result = await projectsStore.saveProjectRuleIdsAndSync(project.id, nextRuleIds, activeToolId, {
              notify: false,
              refreshRules: false,
              refreshSnapshot: false,
              scanActiveProject: false,
            })
            if (result === 'failed') {
              this.actionError = projectsStore.workflowError || translateKey('errors.projectRuleBindingsFailed')
              return 'failed'
            }
            changed = true
          }
          if (changed) await projectsStore.hydrateFromSnapshot()
        } else {
          const toolsStore = useToolsStore()
          if (!toolsStore.activeToolEnabled) return 'failed'
          const currentRuleIds = toolsStore.globalRuleBinding?.items
            .filter((item) => item.itemType === 'rule')
            .map((item) => item.assetId) ?? []
          const exists = currentRuleIds.includes(ruleId)
          const checked = this.bindDraft.selectedToolIds.includes(toolsStore.activeId)
          let nextRuleIds = currentRuleIds
          if (checked && !exists) nextRuleIds = [...currentRuleIds, ruleId]
          if (!checked && exists) nextRuleIds = currentRuleIds.filter((id) => id !== ruleId)

          const result = await toolsStore.saveToolRuleIdsAndSync(toolsStore.activeId, nextRuleIds, {
            forceRepair: options.forceToolOverwrite,
            notify: false,
            refreshRules: false,
          })
          if (result === 'needsOverwriteConfirm') {
            this.actionError = translateKey('errors.toolGlobalOutputOverwriteRequired')
            return 'needsToolOverwriteConfirm'
          }
          if (result === 'failed') {
            this.actionError = toolsStore.globalError || translateKey('errors.toolGlobalRuleBindingsFailed')
            return 'failed'
          }
        }

        await this.hydrateFromSnapshot()
        this.actionError = ''
        notifySuccess(translateKey(this.bindDraft.mode === 'tool' ? 'feedback.toolRuleBindingsApplied' : 'feedback.projectRuleBindingsSaved'))
        this.setBindOpen(false)
        return 'saved'
      } catch (error) {
        this.actionError = resolveUnknownError(
          error,
          this.bindDraft.mode === 'tool' ? 'errors.toolGlobalRuleBindingsFailed' : 'errors.projectRuleBindingsFailed',
        )
        notifyError(this.actionError)
        return 'failed'
      } finally {
        this.bindLoading = false
      }
    },
    async saveDraft() {
      const next = { ...this.draft }
      const parsed = ruleDraftSchema.safeParse(next)
      if (!parsed.success) {
        this.actionError = firstIssue(parsed.error, translateKey('errors.ruleInputInvalid'))
        notifyWarning(this.actionError)
        return
      }

      try {
        const impact = next.id == null ? null : await previewRuleImpact(next.id)
        const response = await saveRuleAsset(
          next.id,
          next.code,
          next.name,
          next.categoryCode,
          entityStateToCode(next.state),
          next.summary,
          next.body,
        )
        if (!response.success || !response.data) {
          this.actionError = resolveAppError(response.error, 'errors.ruleSaveFailed')
          notifyError(this.actionError)
          return
        }

        await this.hydrateFromSnapshot()
        this.activeId = response.data.assetId
        if (impact?.success && impact.data?.requiresProjectRegeneration) {
          this.lastImpact = impact.data
          this.actionError = translateKey('feedback.ruleUpdatedImpact', { count: impact.data.boundProjectCount })
          notifyWarning(this.actionError)
        } else {
          this.actionError = ''
          notifySuccess(translateKey('feedback.ruleSaved'))
        }
      } catch (error) {
        this.actionError = resolveUnknownError(error, 'errors.ruleSaveFailed')
        notifyError(this.actionError)
        return
      }

      this.formOpen = false
    },
    async deleteItem(id: number) {
      try {
        const impact = await previewRuleImpact(id)
        if (impact.success && impact.data && (impact.data.boundProjectCount > 0 || impact.data.boundToolCount > 0)) {
          this.actionError = translateKey('errors.ruleDeleteBound', {
            projects: impact.data.boundProjectCount,
            tools: impact.data.boundToolCount,
          })
          notifyWarning(this.actionError)
          return
        }
        const response = await deleteRuleAsset(id)
        if (!response.success) {
          this.actionError = resolveAppError(response.error, 'errors.ruleDeleteFailed')
          notifyError(this.actionError)
          return
        }

        await this.hydrateFromSnapshot()
        this.detailOpen = false
        if (impact.success && impact.data) {
          this.lastImpact = impact.data
          this.actionError = translateKey('feedback.ruleDeletedImpact', { count: impact.data.boundProjectCount })
          notifyWarning(this.actionError)
        } else {
          notifySuccess(translateKey('feedback.ruleDeleted'))
        }
      } catch (error) {
        this.actionError = resolveUnknownError(error, 'errors.ruleDeleteFailed')
        notifyError(this.actionError)
      }
    },
    async ensureRuleCanDelete(id: number) {
      try {
        const impact = await previewRuleImpact(id)
        if (!impact.success || !impact.data) return true
        if (impact.data.boundProjectCount === 0 && impact.data.boundToolCount === 0) return true

        this.actionError = translateKey('errors.ruleDeleteBound', {
          projects: impact.data.boundProjectCount,
          tools: impact.data.boundToolCount,
        })
        notifyWarning(this.actionError)
        return false
      } catch (error) {
        this.actionError = resolveUnknownError(error, 'errors.ruleDeleteFailed')
        notifyError(this.actionError)
        return false
      }
    },
    setImportOpen(value: boolean) {
      this.importOpen = value
      if (value) {
        this.importDraft = {
          sourcePath: '',
          name: '',
          summary: '',
          body: '',
          categoryCode: null,
          conflictStrategy: 'skip',
        }
        this.importTouched = {
          name: false,
          summary: false,
          categoryCode: false,
        }
        this.lastImportReport = ''
      }
    },
    async previewImportSource(sourcePath: string) {
      if (!sourcePath.trim()) return
      const response = await previewRuleImport(sourcePath)
      if (this.importDraft.sourcePath !== sourcePath) return
      if (!response.success || !response.data) return
      if (!this.importTouched.name) this.importDraft.name = response.data.name.trim()
      if (!this.importTouched.summary) this.importDraft.summary = response.data.summary
      this.importDraft.body = response.data.body
    },
    setImportField(key: 'sourcePath' | 'name' | 'summary' | 'categoryCode' | 'conflictStrategy', value: string | number | null) {
      if (key === 'sourcePath') {
        const nextPath = String(value)
        if (this.importDraft.sourcePath === nextPath) return
        this.importDraft.sourcePath = nextPath
        this.importDraft.name = ''
        this.importDraft.summary = ''
        this.importDraft.body = ''
        this.importDraft.categoryCode = null
        this.importTouched = {
          name: false,
          summary: false,
          categoryCode: false,
        }
        void this.previewImportSource(nextPath)
      }
      if (key === 'name') {
        this.importTouched.name = true
        this.importDraft.name = String(value)
      }
      if (key === 'summary') {
        this.importTouched.summary = true
        this.importDraft.summary = String(value)
      }
      if (key === 'categoryCode') {
        this.importTouched.categoryCode = true
        const nextCategoryCode = value == null ? null : Number(value)
        this.importDraft.categoryCode = isValidRuleCategoryCode(nextCategoryCode) ? nextCategoryCode : null
      }
      if (key === 'conflictStrategy') {
        this.importDraft.conflictStrategy = String(value) as 'skip' | 'rename' | 'overwrite'
      }
    },
    async applyImport() {
      const validationError = ruleImportValidationError(this.importDraft)
      if (validationError) {
        this.actionError = validationError
        notifyWarning(this.actionError)
        return
      }
      const categoryCode = this.importDraft.categoryCode
      if (!isValidRuleCategoryCode(categoryCode)) return
      this.importLoading = true
      try {
        const response = await importRuleAsset(
          this.importDraft.sourcePath,
          this.importDraft.name,
          categoryCode,
          this.importDraft.summary,
          this.importDraft.conflictStrategy,
        )
        if (!response.success || !response.data) {
          this.actionError = response.error?.message?.trim() || resolveAppError(response.error, 'errors.ruleImportFailed')
          notifyError(this.actionError)
          return
        }

        await this.hydrateFromSnapshot()
        this.activeId = response.data.rule.assetId
        this.importOpen = false
        this.lastImportReport = translateKey(`feedback.ruleImport.${response.data.operation}`, {
          name: response.data.rule.name,
        })
        this.actionError = this.lastImportReport
        notifySuccess(this.actionError)
      } catch (error) {
        this.actionError = resolveUnknownError(error, 'errors.ruleImportFailed')
        notifyError(this.actionError)
      } finally {
        this.importLoading = false
      }
    },
    async moveItem(id: number, categoryCode: number, sortOrder: number) {
      try {
        const response = await moveRuleAsset(id, categoryCode, sortOrder)
        if (!response.success || !response.data) {
          this.actionError = resolveAppError(response.error, 'errors.ruleMoveFailed')
          notifyError(this.actionError)
          return
        }

        await this.hydrateFromSnapshot()
        this.activeId = response.data.assetId
        notifySuccess(translateKey('feedback.ruleMoved'))
      } catch (error) {
        this.actionError = resolveUnknownError(error, 'errors.ruleMoveFailed')
        notifyError(this.actionError)
      }
    },
  },
})
