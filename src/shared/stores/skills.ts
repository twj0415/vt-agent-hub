import { defineStore } from 'pinia'
import {
  applyRepositoryImport,
  deleteSkillAsset,
  getLibrarySnapshot,
  importGitHubRepoSkills,
  installSkillAsset,
  markSkillAssetStale,
  previewGitHubRepoImport,
  previewRepositoryImport,
  repairSkillAsset,
  saveSkillAsset,
  uninstallSkillAsset,
} from '@/shared/api/tauri'
import {
  entityStateFromCode,
  entityStateToCode,
  skillInstallStateFromCode,
  skillInstallStateToCode,
  type EntityState,
  type SkillInstallState,
} from '@/shared/constants/status'
import type {
  GitHubRepoImportResult,
  GitHubRepoPreview,
  GitHubSkillImportSelection,
  RepositoryImportReport,
  SkillRuntime,
} from '@/shared/api/client'
import { resolveAppError, resolveUnknownError, translateKey } from '@/shared/i18n/translate'
import { isTauriRuntime } from '@/shared/utils/runtime'
import { notifyError, notifySuccess, notifyWarning } from '@/shared/utils/notify'
import { firstIssue, skillDraftSchema } from '@/shared/validation/forms'
import { defaultSkillCategoryCode } from '@/shared/taxonomy'
import { useToolContextStore } from './tool-context'
import { useToolsStore } from './tools'

type SkillSort = 'name' | 'code'
type GitHubImportStep = 'input' | 'preview' | 'result'
type GitHubSelectionState = GitHubSkillImportSelection & { selected: boolean }

type SkillToolBindDraft = {
  skillId: number | null
  selectedToolIds: number[]
}

export type SkillItem = {
  id: number
  versionId: number
  versionNo: number
  key: string
  name: string
  code: number
  state: EntityState
  summary: string
  categoryCode: number
  installState: SkillInstallState
  body: string
  runtime: SkillRuntime
  toolIds: number[]
}

export type SkillHealthState = 'normal' | 'abnormal'

type SkillDraft = {
  id: number | null
  name: string
  code: number
  state: EntityState
  summary: string
  categoryCode: number
  body: string
}

function createSkillDraft(): SkillDraft {
  return {
    id: null,
    name: '',
    code: defaultSkillCategoryCode,
    state: 'planned',
    summary: '',
    categoryCode: defaultSkillCategoryCode,
    body: '',
  }
}

function createToolBindDraft(): SkillToolBindDraft {
  return {
    skillId: null,
    selectedToolIds: [],
  }
}

function sortItems(items: SkillItem[], sort: SkillSort) {
  return [...items].sort((a, b) => {
    if (sort === 'code') return a.code - b.code
    return a.name.localeCompare(b.name)
  })
}

function buildGitHubSelections(preview: GitHubRepoPreview): Record<string, GitHubSelectionState> {
  return Object.fromEntries(preview.skills.map((skill) => [
    skill.sourcePath,
    {
      sourcePath: skill.sourcePath,
      selected: true,
      resolution: skill.conflict ? 'skip' : 'overwrite',
      renamedSkillId: skill.skillId,
    },
  ]))
}

function emptyRuntime(name: string, body: string): SkillRuntime {
  return {
    platformRoot: '',
    libraryPath: '',
    librarySkillMdPath: '',
    runtimePath: '',
    runtimeSkillMdPath: '',
    libraryExists: false,
    runtimeExists: false,
    skillMdValid: false,
    installState: 601,
    statusDetail: translateKey('feedback.skillRuntimeNotLoaded', { name }),
    libraryBody: body,
    runtimeBody: '',
    libraryTree: [],
    runtimeTree: [],
    installActionReady: false,
    uninstallActionReady: false,
    repairActionReady: false,
    markStaleActionReady: false,
  }
}

export function skillHealthState(item: Pick<SkillItem, 'runtime'>): SkillHealthState {
  const runtime = item.runtime
  if (!runtime.libraryExists || !runtime.skillMdValid) return 'abnormal'
  return 'normal'
}

export const useSkillStore = defineStore('skills', {
  state: () => ({
    items: [] as SkillItem[],
    activeId: 0,
    search: '',
    filter: 'all' as 'all' | number,
    sort: 'name' as SkillSort,
    detailOpen: false,
    formOpen: false,
    importOpen: false,
    bindOpen: false,
    importLoading: false,
    bindLoading: false,
    draft: createSkillDraft(),
    bindDraft: createToolBindDraft(),
    importDraft: {
      source: '',
      branch: '',
      conflictStrategy: 'skip' as 'skip' | 'rename' | 'overwrite',
    },
    repositoryImportReport: null as RepositoryImportReport | null,
    githubPreview: null as GitHubRepoPreview | null,
    githubImportResult: null as GitHubRepoImportResult | null,
    githubSelections: {} as Record<string, GitHubSelectionState>,
    githubSelectedPath: '',
    githubStep: 'input' as GitHubImportStep,
    actionError: '',
  }),
  getters: {
    activeItem(state) {
      return state.items.find((item) => item.id === state.activeId) ?? null
    },
    runtimeActionsConnected(state) {
      const active = state.items.find((item) => item.id === state.activeId) ?? null
      return Boolean(active?.runtime.platformRoot)
    },
    runtimeActionNotice(state) {
      const runtime = state.items.find((item) => item.id === state.activeId)?.runtime
      if (!runtime) return ''
      if (!runtime.platformRoot) return translateKey('feedback.skillPlatformRootUnavailable')
      if (!runtime.libraryExists) return translateKey('feedback.skillLibraryMissing')
      if (!runtime.skillMdValid) return translateKey('feedback.skillMdInvalid')
      return ''
    },
    filteredItems(state) {
      const keyword = state.search.trim().toLowerCase()
      const filtered = state.items.filter((item) => {
        const matchesSearch = !keyword
          || item.name.toLowerCase().includes(keyword)
          || item.summary.toLowerCase().includes(keyword)
          || String(item.code).includes(keyword)
        const matchesFilter = state.filter === 'all' || item.categoryCode === state.filter
        return matchesSearch && matchesFilter
      })

      return sortItems(filtered, state.sort)
    },
  },
  actions: {
    activeToolId() {
      return useToolContextStore().activeToolId
    },
    async hydrateFromSnapshot() {
      try {
        const response = await getLibrarySnapshot()
        if (!response.success || !response.data) {
          if (isTauriRuntime()) throw new Error(resolveAppError(response.error, 'errors.librarySnapshotFailed'))
          return
        }

        this.items = response.data.skills.map((skill) => ({
          id: skill.assetId,
          versionId: skill.versionId,
          versionNo: skill.versionNo,
          key: skill.key,
          name: skill.name,
          code: skill.code,
          state: entityStateFromCode(skill.state),
          summary: skill.summary,
          categoryCode: skill.categoryCode,
          installState: skillInstallStateFromCode(skill.runtime.installState),
          body: skill.body,
          runtime: skill.runtime,
          toolIds: skill.toolIds ?? [],
        }))
        this.activeId = this.items.find((item) => item.id === this.activeId)?.id ?? this.items[0]?.id ?? 0
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
    setSort(value: SkillSort) {
      this.sort = value
    },
    setDetailOpen(value: boolean) {
      this.detailOpen = value
    },
    openCreate() {
      this.draft = createSkillDraft()
      this.formOpen = true
    },
    openEdit(id: number) {
      const current = this.items.find((item) => item.id === id)
      if (!current) return

      this.draft = {
        id: current.id,
        name: current.name,
        code: current.code,
        state: current.state,
        summary: current.summary,
        categoryCode: current.categoryCode,
        body: current.body,
      }
      this.formOpen = true
    },
    setFormOpen(value: boolean) {
      this.formOpen = value
    },
    setBindOpen(value: boolean) {
      this.bindOpen = value
    },
    openToolBinding(skillId: number) {
      const skill = this.items.find((item) => item.id === skillId)
      if (!skill) return
      this.bindDraft = {
        skillId,
        selectedToolIds: [...skill.toolIds],
      }
      this.bindOpen = true
    },
    toggleBindTool(toolId: number) {
      if (this.bindDraft.selectedToolIds.includes(toolId)) {
        this.bindDraft.selectedToolIds = this.bindDraft.selectedToolIds.filter((id) => id !== toolId)
        return
      }
      this.bindDraft.selectedToolIds = [...this.bindDraft.selectedToolIds, toolId]
    },
    async saveToolBinding() {
      const skillId = this.bindDraft.skillId
      if (skillId == null) return

      this.bindLoading = true
      try {
        const toolsStore = useToolsStore()
        let changed = false
        for (const tool of toolsStore.items) {
          if (!toolsStore.isToolEnabled(tool.id)) continue
          const currentSkillIds = this.items
            .filter((item) => item.toolIds.includes(tool.id))
            .map((item) => item.id)
          const exists = currentSkillIds.includes(skillId)
          const checked = this.bindDraft.selectedToolIds.includes(tool.id)
          let nextSkillIds = currentSkillIds
          if (checked && !exists) nextSkillIds = [...currentSkillIds, skillId]
          if (!checked && exists) nextSkillIds = currentSkillIds.filter((id) => id !== skillId)
          if (nextSkillIds.length === currentSkillIds.length && nextSkillIds.every((id, index) => id === currentSkillIds[index])) continue

          const result = await toolsStore.saveToolSkillIdsAndSync(tool.id, nextSkillIds, {
            notify: false,
            refreshSkills: false,
            refreshSnapshot: false,
          })
          if (result === 'failed') {
            this.actionError = toolsStore.globalError || translateKey('errors.skillBindingsSaveFailed')
            notifyError(this.actionError)
            return
          }
          changed = true
        }

        if (changed) await toolsStore.hydrateFromSnapshot()
        await this.hydrateFromSnapshot()
        this.bindOpen = false
        this.actionError = ''
        notifySuccess(translateKey('feedback.skillBindingsSaved'))
      } catch (error) {
        this.actionError = resolveUnknownError(error, 'errors.skillBindingsSaveFailed')
        notifyError(this.actionError)
      } finally {
        this.bindLoading = false
      }
    },
    setDraftField<K extends keyof SkillDraft>(key: K, value: SkillDraft[K]) {
      this.draft[key] = value
    },
    async saveDraft() {
      const next = { ...this.draft }
      const parsed = skillDraftSchema.safeParse(next)
      if (!parsed.success) {
        this.actionError = firstIssue(parsed.error, translateKey('errors.skillInputInvalid'))
        notifyWarning(this.actionError)
        return
      }

      try {
        const response = await saveSkillAsset(
          next.id,
          next.code,
          next.name,
          next.categoryCode,
          entityStateToCode(next.state),
          skillInstallStateToCode('not_installed'),
          next.summary,
          next.body,
        )
        if (!response.success || !response.data) {
          this.actionError = resolveAppError(response.error, 'errors.skillSaveFailed')
          notifyError(this.actionError)
          return
        }

        await this.hydrateFromSnapshot()
        this.activeId = response.data.assetId
        this.actionError = ''
        notifySuccess(translateKey('feedback.skillSaved'))
      } catch (error) {
        this.actionError = resolveUnknownError(error, 'errors.skillSaveFailed')
        notifyError(this.actionError)
        return
      }

      this.formOpen = false
    },
    async deleteItem(id: number) {
      try {
        const response = await deleteSkillAsset(id)
        if (!response.success) {
          this.actionError = resolveAppError(response.error, 'errors.skillDeleteFailed')
          notifyError(this.actionError)
          return
        }

        await this.hydrateFromSnapshot()
        this.detailOpen = false
        this.actionError = ''
        notifySuccess(translateKey('feedback.skillDeleted'))
      } catch (error) {
        this.actionError = resolveUnknownError(error, 'errors.skillDeleteFailed')
        notifyError(this.actionError)
        return
      }
    },
    setImportOpen(value: boolean) {
      this.importOpen = value
      if (value) {
        this.resetGitHubImport()
      }
    },
    setImportField(key: 'source' | 'branch' | 'conflictStrategy', value: string) {
      if (key === 'source') this.importDraft.source = value
      if (key === 'branch') this.importDraft.branch = value
      if (key === 'conflictStrategy') {
        this.importDraft.conflictStrategy = value as 'skip' | 'rename' | 'overwrite'
      }
      this.repositoryImportReport = null
    },
    async previewRepositoryImport() {
      this.importLoading = true
      try {
        const response = await previewRepositoryImport(
          this.importDraft.source,
          this.importDraft.branch,
          this.importDraft.conflictStrategy,
        )
        if (!response.success || !response.data) {
          this.actionError = resolveAppError(response.error, 'errors.repositoryImportPreviewFailed')
          notifyError(this.actionError)
          return
        }
        this.repositoryImportReport = response.data
        this.actionError = translateKey('feedback.repositoryPreviewDetected', { count: response.data.assets.length })
        notifySuccess(this.actionError)
      } catch (error) {
        this.actionError = resolveUnknownError(error, 'errors.repositoryImportPreviewFailed')
        notifyError(this.actionError)
      } finally {
        this.importLoading = false
      }
    },
    async applyImport() {
      this.importLoading = true
      try {
        const response = await applyRepositoryImport(
          this.importDraft.source,
          this.importDraft.branch,
          this.importDraft.conflictStrategy,
        )
        if (!response.success || !response.data) {
          this.actionError = resolveAppError(response.error, 'errors.repositoryImportFailed')
          notifyError(this.actionError)
          return
        }
        await this.hydrateFromSnapshot()
        this.repositoryImportReport = response.data
        this.importOpen = false
        this.actionError = translateKey('feedback.repositoryImportCompleted', {
          rules: response.data.importedRules,
          skills: response.data.importedSkills,
          presets: response.data.detectedPresets,
        })
        notifySuccess(this.actionError)
      } catch (error) {
        this.actionError = resolveUnknownError(error, 'errors.repositoryImportFailed')
        notifyError(this.actionError)
      } finally {
        this.importLoading = false
      }
    },
    resetGitHubImport() {
      this.repositoryImportReport = null
      this.githubPreview = null
      this.githubImportResult = null
      this.githubSelections = {}
      this.githubSelectedPath = ''
      this.githubStep = 'input'
    },
    async previewGitHubImport() {
      this.importLoading = true
      this.githubImportResult = null
      try {
        const response = await previewGitHubRepoImport(this.importDraft.source)
        if (!response.success || !response.data) {
          this.actionError = resolveAppError(response.error, 'errors.repositoryImportPreviewFailed')
          notifyError(this.actionError)
          return
        }
        this.githubPreview = response.data
        this.githubSelections = buildGitHubSelections(response.data)
        this.githubSelectedPath = response.data.skills[0]?.sourcePath ?? ''
        this.githubStep = 'preview'
        this.actionError = translateKey('feedback.repositoryPreviewDetected', { count: response.data.skills.length })
        notifySuccess(this.actionError)
      } catch (error) {
        this.actionError = resolveUnknownError(error, 'errors.repositoryImportPreviewFailed')
        notifyError(this.actionError)
      } finally {
        this.importLoading = false
      }
    },
    toggleGitHubSkillSelection(sourcePath: string) {
      const current = this.githubSelections[sourcePath]
      if (!current) return
      current.selected = !current.selected
    },
    setGitHubSkillResolution(sourcePath: string, resolution: 'skip' | 'overwrite' | 'rename') {
      const current = this.githubSelections[sourcePath]
      if (!current) return
      current.resolution = resolution
    },
    setGitHubSkillRename(sourcePath: string, renamedSkillId: string) {
      const current = this.githubSelections[sourcePath]
      if (!current) return
      current.renamedSkillId = renamedSkillId
    },
    setGitHubSelectedPath(sourcePath: string) {
      this.githubSelectedPath = sourcePath
    },
    async applyGitHubImport() {
      const selections = Object.values(this.githubSelections)
        .filter((selection) => selection.selected)
        .map(({ selected: _selected, ...selection }) => selection)
      if (selections.length === 0) {
        this.actionError = translateKey('errors.repositorySourceRequired')
        notifyWarning(this.actionError)
        return
      }

      this.importLoading = true
      try {
        const response = await importGitHubRepoSkills(this.importDraft.source, selections)
        if (!response.success || !response.data) {
          this.actionError = resolveAppError(response.error, 'errors.repositoryImportFailed')
          notifyError(this.actionError)
          return
        }
        await this.hydrateFromSnapshot()
        this.githubImportResult = response.data
        this.githubStep = 'result'
        this.actionError = translateKey('feedback.repositoryImportCompleted', {
          rules: 0,
          skills: response.data.importedSkills.length,
          presets: 0,
        })
        notifySuccess(this.actionError)
      } catch (error) {
        this.actionError = resolveUnknownError(error, 'errors.repositoryImportFailed')
        notifyError(this.actionError)
      } finally {
        this.importLoading = false
      }
    },
    async install(id: number) {
      const item = this.items.find((entry) => entry.id === id)
      if (item && !item.runtime.installActionReady) {
        this.actionError = item.runtime.statusDetail
        notifyWarning(this.actionError)
        return
      }
      const response = await installSkillAsset(this.activeToolId(), id)
      if (!response.success || !response.data) {
        this.actionError = resolveAppError(response.error, 'errors.skillInstallFailed')
        notifyError(this.actionError)
        return
      }
      await this.hydrateFromSnapshot()
      this.actionError = response.data.statusDetail
      notifySuccess(translateKey('feedback.skillInstalled'))
    },
    async uninstall(id: number) {
      const item = this.items.find((entry) => entry.id === id)
      if (item && !item.runtime.uninstallActionReady) {
        this.actionError = item.runtime.statusDetail
        notifyWarning(this.actionError)
        return
      }
      const response = await uninstallSkillAsset(this.activeToolId(), id)
      if (!response.success || !response.data) {
        this.actionError = resolveAppError(response.error, 'errors.skillUninstallFailed')
        notifyError(this.actionError)
        return
      }
      await this.hydrateFromSnapshot()
      this.actionError = response.data.statusDetail
      notifySuccess(translateKey('feedback.skillUninstalled'))
    },
    async repair(id: number) {
      const item = this.items.find((entry) => entry.id === id)
      if (item && !item.runtime.repairActionReady) {
        this.actionError = item.runtime.statusDetail
        notifyWarning(this.actionError)
        return
      }
      const response = await repairSkillAsset(this.activeToolId(), id)
      if (!response.success || !response.data) {
        this.actionError = resolveAppError(response.error, 'errors.skillRepairFailed')
        notifyError(this.actionError)
        return
      }
      await this.hydrateFromSnapshot()
      this.actionError = response.data.statusDetail
      notifySuccess(translateKey('feedback.skillRepaired'))
    },
    async markStale(id: number) {
      const item = this.items.find((entry) => entry.id === id)
      if (item && !item.runtime.markStaleActionReady) {
        this.actionError = item.runtime.statusDetail
        notifyWarning(this.actionError)
        return
      }
      const response = await markSkillAssetStale(this.activeToolId(), id)
      if (!response.success || !response.data) {
        this.actionError = resolveAppError(response.error, 'errors.skillMarkStaleFailed')
        notifyError(this.actionError)
        return
      }
      await this.hydrateFromSnapshot()
      this.actionError = response.data.statusDetail
      notifySuccess(translateKey('feedback.skillMarkedStale'))
    },
  },
})
