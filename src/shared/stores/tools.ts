import { defineStore } from 'pinia'
import {
  applyGlobalOutput,
  clearToolCredentialState,
  cleanupGlobalOutput,
  exportLibraryDiagnostics,
  getToolDiagnostics,
  getToolsSnapshot,
  installSkillAsset,
  previewGlobalOutput,
  repairGlobalOutput,
  repairTool,
  scanLibraryDiagnostics,
  saveToolCredentialState,
  saveToolGlobalRuleBindings,
  saveToolSkillBindings,
  setToolEnabled,
  uninstallSkillAsset,
  verifyToolCredential,
} from '@/shared/api/tauri'
import type {
  DiagnosticExportResult,
  GlobalOutputPreview,
  GlobalOutputWriteResult,
  LibraryDiagnostics,
  ToolActionResult,
  ToolDiagnostics,
  ToolRulePackBinding,
  ToolSkillInstall,
} from '@/shared/api/client'
import { capabilityOrder, getToolById, toolIds, toolRegistry, type ToolCapabilityKey, type ToolId } from '@/shared/tool-registry'
import { isTauriRuntime } from '@/shared/utils/runtime'
import { notifyError, notifySuccess, notifyWarning } from '@/shared/utils/notify'
import { credentialSchema, firstIssue } from '@/shared/validation/forms'
import { translateKey } from '@/shared/i18n/translate'
import { localizeMessage } from '@/shared/utils/message'
import { useRuleStore } from './rules'
import { useSkillStore } from './skills'

type ToolDraft = {
  profile: string
  root: string
  notes: string
  token: string
}

export type ToolRuleBindingSaveResult = 'saved' | 'needsOverwriteConfirm' | 'failed'

function createEmptyDiagnostics(): ToolDiagnostics {
  return {
    installationDetected: false,
    version: '-',
    liveConfigPath: '',
    credentialState: 'unknown',
    credentialStateCode: 504,
    skillState: 'unknown',
    skillStateCode: 504,
    projectOutputState: 'unknown',
    projectOutputStateCode: 504,
    repairState: 'unknown',
    repairStateCode: 504,
    repairHint: '',
  }
}

function createActionResult(): ToolActionResult {
  return {
    ok: false,
    state: 'idle',
    detail: '',
    manualSteps: [],
  }
}

type ToolStepKey =
  | 'toolCredentialTokenCheck'
  | 'toolCredentialRetryCheck'
  | 'toolCredentialSaveStorage'
  | 'toolCredentialVerifyBeforeUse'
  | 'toolRepairRetry'
  | 'toolCredentialClearStorage'

function toolStepKey(key: ToolStepKey) {
  return `ui.common.steps.${key}`
}

export const useToolsStore = defineStore('tools', {
  state: () => ({
    activeId: toolIds.codex as ToolId,
    draft: {
      profile: '',
      root: '',
      notes: '',
      token: '',
    } as ToolDraft,
    diagnostics: createEmptyDiagnostics() as ToolDiagnostics,
    verifyResult: createActionResult() as ToolActionResult,
    repairResult: createActionResult() as ToolActionResult,
    globalRuleBinding: null as ToolRulePackBinding | null,
    skillPackBinding: null as ToolRulePackBinding | null,
    skillInstalls: [] as ToolSkillInstall[],
    globalPreview: null as GlobalOutputPreview | null,
    globalPreviewLoading: false,
    globalWriteResult: null as GlobalOutputWriteResult | null,
    globalError: '',
    detailOpen: false,
    bindOpen: false,
    bindLoading: false,
    bindingDraft: {
      selectedNewRuleIds: [] as number[],
    },
    skillBindOpen: false,
    skillBindLoading: false,
    skillBindingDraft: {
      selectedNewSkillIds: [] as number[],
    },
    libraryDiagnostics: null as LibraryDiagnostics | null,
    lastDiagnosticsExport: null as DiagnosticExportResult | null,
    diagnosticsError: '',
  }),
  getters: {
    items() {
      return toolRegistry
    },
    activeItem(state) {
      return getToolById(state.activeId)
    },
    activeCapabilityCount(): number {
      if (!this.activeItem?.enabled) return 0
      return capabilityOrder.filter((key) => this.activeItem?.capabilities[key]).length
    },
    activeCapabilities(): ToolCapabilityKey[] {
      if (!this.activeItem?.enabled) return []
      return capabilityOrder.filter((key) => this.activeItem?.capabilities[key])
    },
    activeToolEnabled(): boolean {
      return Boolean(this.activeItem?.enabled)
    },
    selectedRuleCount(state): number {
      return state.bindingDraft.selectedNewRuleIds.length
    },
    selectedSkillCount(state): number {
      return state.skillBindingDraft.selectedNewSkillIds.length
    },
  },
  actions: {
    async hydrateFromSnapshot() {
      try {
        const response = await getToolsSnapshot(this.activeId)
        if (!response.success || !response.data) {
          if (isTauriRuntime()) throw new Error(response.error?.message ?? 'Tools snapshot failed.')
          return
        }

        const active = response.data.tools.find((item) => item.id === this.activeId && item.enabled)
        if (!active) {
          const fallback = response.data.tools.find((item) => item.enabled)
          if (fallback && getToolById(fallback.id)?.enabled && !getToolById(this.activeId)?.enabled) {
            this.activeId = fallback.id as ToolId
          }
        }
        this.globalRuleBinding = response.data.globalRuleBinding ?? null
        this.skillPackBinding = response.data.skillPackBinding ?? null
        this.skillInstalls = response.data.skillInstalls ?? []
      } catch (error) {
        if (isTauriRuntime()) throw error
      }

      await this.hydrateDiagnostics()
      await this.hydrateLibraryDiagnostics()
    },
    async hydrateDiagnostics() {
      try {
        const response = await getToolDiagnostics(this.activeId)
        if (!response.success || !response.data) {
          if (isTauriRuntime()) throw new Error(response.error?.message ?? 'Tool diagnostics failed.')
          return
        }

        this.diagnostics = response.data
        this.draft.root = response.data.liveConfigPath
      } catch (error) {
        if (isTauriRuntime()) throw error
      }
    },
    async hydrateLibraryDiagnostics() {
      try {
        const response = await scanLibraryDiagnostics()
        if (!response.success || !response.data) {
          if (isTauriRuntime()) throw new Error(response.error?.message ?? 'Library diagnostics failed.')
          return
        }

        this.libraryDiagnostics = response.data
        this.diagnosticsError = ''
      } catch (error) {
        this.diagnosticsError = error instanceof Error ? error.message : 'Library diagnostics failed.'
        if (isTauriRuntime()) throw error
      }
    },
    isToolEnabled(id?: ToolId | number) {
      return Boolean(getToolById(id ?? this.activeId)?.enabled)
    },
    async verifyCredential() {
      if (!this.activeToolEnabled) return
      const parsed = credentialSchema.safeParse({ token: this.draft.token })
      if (!parsed.success) {
        this.verifyResult = {
          ok: false,
          state: 'local_invalid',
          detail: firstIssue(parsed.error, 'Credential input is invalid.'),
          manualSteps: [translateKey(toolStepKey('toolCredentialTokenCheck'))],
        }
        notifyWarning(this.verifyResult.detail)
        return
      }

      try {
        const response = await verifyToolCredential(this.activeId, this.draft.token)
        if (!response.success || !response.data) {
          this.verifyResult = {
            ok: false,
            state: response.error?.code ?? 'verify_failed',
            detail: response.error?.message ?? 'Credential verification failed.',
            manualSteps: [translateKey(toolStepKey('toolCredentialRetryCheck'))],
          }
          notifyError(this.verifyResult.detail)
          return
        }

        this.verifyResult = response.data
        notifySuccess(response.data.detail || response.data.state)
      } catch (error) {
        this.verifyResult = {
          ok: false,
          state: 'verify_failed',
          detail: error instanceof Error ? error.message : 'Credential verification failed.',
          manualSteps: [translateKey(toolStepKey('toolCredentialRetryCheck'))],
        }
        notifyError(this.verifyResult.detail)
      }
    },
    async saveCredential() {
      if (!this.activeToolEnabled) return
      const parsed = credentialSchema.safeParse({ token: this.draft.token })
      if (!parsed.success) {
        this.verifyResult = {
          ok: false,
          state: 'local_invalid',
          detail: firstIssue(parsed.error, 'Credential input is invalid.'),
          manualSteps: [translateKey(toolStepKey('toolCredentialTokenCheck'))],
        }
        notifyWarning(this.verifyResult.detail)
        return
      }

      try {
        const persist = await saveToolCredentialState(this.activeId, this.draft.token)
        if (!persist.success) {
          this.verifyResult = {
            ok: false,
            state: persist.error?.code ?? 'credential_persist_failed',
            detail: persist.error?.message ?? 'Credential persistence failed.',
            manualSteps: [translateKey(toolStepKey('toolCredentialSaveStorage'))],
          }
          notifyError(this.verifyResult.detail)
          return
        }

        this.verifyResult = {
          ok: true,
          state: 'credential_saved',
          detail: translateKey('feedback.credentialSaved'),
          manualSteps: [translateKey(toolStepKey('toolCredentialVerifyBeforeUse'))],
        }
        notifySuccess(this.verifyResult.detail)
        await this.hydrateDiagnostics()
      } catch (error) {
        this.verifyResult = {
          ok: false,
          state: 'credential_persist_failed',
          detail: error instanceof Error ? error.message : 'Credential persistence failed.',
          manualSteps: [translateKey(toolStepKey('toolCredentialSaveStorage'))],
        }
        notifyError(this.verifyResult.detail)
      }
    },
    async repair() {
      if (!this.activeToolEnabled) return
      try {
        const response = await repairTool(this.activeId)
        if (!response.success || !response.data) {
          this.repairResult = {
            ok: false,
            state: response.error?.code ?? 'repair_failed',
            detail: response.error?.message ?? 'Repair failed.',
            manualSteps: [translateKey(toolStepKey('toolRepairRetry'))],
          }
          notifyError(this.repairResult.detail)
          return
        }

        this.repairResult = response.data
        notifySuccess(response.data.detail || response.data.state)
        try {
          await this.hydrateDiagnostics()
          await this.hydrateLibraryDiagnostics()
        } catch {
          // Preserve the repair result when the post-repair refresh fails.
        }
      } catch (error) {
        this.repairResult = {
          ok: false,
          state: 'repair_failed',
          detail: error instanceof Error ? error.message : 'Repair failed.',
          manualSteps: [translateKey(toolStepKey('toolRepairRetry'))],
        }
        notifyError(this.repairResult.detail)
      }
    },
    select(id: ToolId) {
      if (getToolById(id)?.enabled) {
        this.activeId = id
        this.clearToolRuntimeState()
        void this.hydrateFromSnapshot()
      }
    },
    clearToolRuntimeState() {
      this.globalRuleBinding = null
      this.skillPackBinding = null
      this.skillInstalls = []
      this.globalPreview = null
      this.globalWriteResult = null
      this.globalError = ''
      this.detailOpen = false
      this.bindOpen = false
      this.skillBindOpen = false
      this.bindingDraft.selectedNewRuleIds = []
      this.skillBindingDraft.selectedNewSkillIds = []
    },
    async setToolEnabled(toolId: ToolId, enabled: boolean) {
      try {
        const response = await setToolEnabled(toolId, enabled)
        if (!response.success) {
          this.globalError = response.error?.message ? localizeMessage(response.error.message) : translateKey('errors.toolSetEnabledFailed')
          notifyError(this.globalError)
          return false
        }
        if (!enabled && toolId === this.activeId) {
          const fallback = toolRegistry.find((item) => item.enabled && item.id !== toolId)
          this.activeId = (fallback?.id ?? toolIds.codex) as ToolId
          this.clearToolRuntimeState()
        }
        await this.hydrateFromSnapshot()
        return true
      } catch (error) {
        this.globalError = error instanceof Error ? localizeMessage(error.message) : translateKey('errors.toolSetEnabledFailed')
        notifyError(this.globalError)
        return false
      }
    },
    setDraftField(key: keyof ToolDraft, value: string) {
      this.draft[key] = value
    },
    setGlobalRuleBinding(binding: ToolRulePackBinding | null) {
      this.globalRuleBinding = binding
    },
    setDetailOpen(value: boolean) {
      this.detailOpen = value && this.activeToolEnabled
    },
    setBindOpen(value: boolean) {
      this.bindOpen = value && this.activeToolEnabled
    },
    setSkillBindOpen(value: boolean) {
      this.skillBindOpen = value && this.activeToolEnabled
    },
    toggleRuleSelection(ruleId: number) {
      if (!this.activeToolEnabled) return
      if (this.bindingDraft.selectedNewRuleIds.includes(ruleId)) {
        this.bindingDraft.selectedNewRuleIds = this.bindingDraft.selectedNewRuleIds.filter((id) => id !== ruleId)
        return
      }
      this.bindingDraft.selectedNewRuleIds = [...this.bindingDraft.selectedNewRuleIds, ruleId]
    },
    toggleSkillSelection(skillId: number) {
      if (!this.activeToolEnabled) return
      if (this.skillBindingDraft.selectedNewSkillIds.includes(skillId)) {
        this.skillBindingDraft.selectedNewSkillIds = this.skillBindingDraft.selectedNewSkillIds.filter((id) => id !== skillId)
        return
      }
      this.skillBindingDraft.selectedNewSkillIds = [...this.skillBindingDraft.selectedNewSkillIds, skillId]
    },
    openRuleBinding() {
      if (!this.activeToolEnabled) return
      this.bindingDraft.selectedNewRuleIds = []
      this.bindOpen = true
    },
    openSkillBinding() {
      if (!this.activeToolEnabled) return
      this.skillBindingDraft.selectedNewSkillIds = useSkillStore().items
        .filter((item) => item.toolIds.includes(this.activeId))
        .map((item) => item.id)
      this.skillBindOpen = true
    },
    async saveToolRuleIdsAndSync(
      toolId: ToolId,
      ruleIds: number[],
      options: {
        closeBind?: boolean
        forceRepair?: boolean
        notify?: boolean
        refreshRules?: boolean
        refreshSnapshot?: boolean
        updatePreview?: boolean
      } = {},
    ): Promise<ToolRuleBindingSaveResult> {
      if (!this.isToolEnabled(toolId)) return 'failed'
      this.bindLoading = true
      try {
        const response = await saveToolGlobalRuleBindings(toolId, ruleIds)
        if (!response.success) {
          this.globalError = response.error?.message ?? translateKey('errors.toolGlobalRuleBindingsFailed')
          notifyError(this.globalError)
          return 'failed'
        }

        const syncResponse = ruleIds.length
          ? options.forceRepair
            ? await repairGlobalOutput(toolId, true)
            : await applyGlobalOutput(toolId, true)
          : await cleanupGlobalOutput(toolId, true)
        if (!syncResponse.success || (ruleIds.length && !syncResponse.data)) {
          if (ruleIds.length && !options.forceRepair && syncResponse.error?.code === 'global_output_apply_failed') {
            this.globalError = translateKey('errors.toolGlobalOutputOverwriteRequired')
            return 'needsOverwriteConfirm'
          }
          this.globalError = translateKey('errors.toolRulesSavedApplyFailed', {
            error: syncResponse.error?.message ?? translateKey('errors.toolGlobalOutputSyncFailed'),
          })
          notifyError(this.globalError)
          return 'failed'
        }

        this.globalWriteResult = syncResponse.data ?? null
        if (options.refreshSnapshot !== false) await this.hydrateFromSnapshot()
        if (options.refreshRules) await useRuleStore().hydrateFromSnapshot()
        if (options.updatePreview !== false && toolId === this.activeId && ruleIds.length) {
          await this.loadGlobalPreview()
        } else if (toolId === this.activeId && !ruleIds.length) {
          this.globalPreview = null
        }
        if (options.closeBind) this.bindOpen = false
        this.globalError = ''
        if (options.notify) notifySuccess(translateKey('feedback.toolRuleBindingsApplied'))
        return 'saved'
      } catch (error) {
        this.globalError = translateKey('errors.toolRulesSavedApplyFailed', {
          error: error instanceof Error ? error.message : translateKey('errors.toolGlobalOutputSyncFailed'),
        })
        notifyError(this.globalError)
        return 'failed'
      } finally {
        this.bindLoading = false
      }
    },
    async saveRuleBindingAndApply(options: { forceRepair?: boolean } = {}) {
      if (!this.activeToolEnabled) return
      const currentRuleIds = this.globalRuleBinding?.items
        .filter((item) => item.itemType === 'rule')
        .sort((a, b) => a.sortOrder - b.sortOrder)
        .map((item) => item.assetId) ?? []
      const nextRuleIds = Array.from(new Set([...currentRuleIds, ...this.bindingDraft.selectedNewRuleIds]))

      await this.saveToolRuleIdsAndSync(this.activeId, nextRuleIds, {
        closeBind: true,
        forceRepair: options.forceRepair,
        notify: true,
        refreshRules: true,
      })
    },
    async saveToolSkillIdsAndSync(
      toolId: ToolId,
      skillIds: number[],
      options: {
        closeBind?: boolean
        notify?: boolean
        refreshSkills?: boolean
        refreshSnapshot?: boolean
      } = {},
    ): Promise<ToolRuleBindingSaveResult> {
      if (!this.isToolEnabled(toolId)) return 'failed'
      this.skillBindLoading = true
      try {
        const skillStore = useSkillStore()
        const nextSkillIds = Array.from(new Set(skillIds))
        const currentSkillIds = skillStore.items
          .filter((item) => item.toolIds.includes(toolId))
          .map((item) => item.id)
        const addedSkillIds = nextSkillIds.filter((id) => !currentSkillIds.includes(id))
        const removedSkillIds = currentSkillIds.filter((id) => !nextSkillIds.includes(id))

        const installedSkillIds: number[] = []
        for (const skillId of addedSkillIds) {
          const installResponse = await installSkillAsset(toolId, skillId)
          if (!installResponse.success) {
            await Promise.allSettled(installedSkillIds.map((id) => uninstallSkillAsset(toolId, id)))
            this.globalError = installResponse.error?.message ?? translateKey('errors.skillInstallFailed')
            notifyError(this.globalError)
            return 'failed'
          }
          installedSkillIds.push(skillId)
        }

        const uninstalledSkillIds: number[] = []
        for (const skillId of removedSkillIds) {
          const uninstallResponse = await uninstallSkillAsset(toolId, skillId)
          if (!uninstallResponse.success) {
            await Promise.allSettled(installedSkillIds.map((id) => uninstallSkillAsset(toolId, id)))
            await Promise.allSettled(uninstalledSkillIds.map((id) => installSkillAsset(toolId, id)))
            this.globalError = uninstallResponse.error?.message ?? translateKey('errors.skillUninstallFailed')
            notifyError(this.globalError)
            return 'failed'
          }
          uninstalledSkillIds.push(skillId)
        }

        const response = await saveToolSkillBindings(toolId, nextSkillIds)
        if (!response.success) {
          await Promise.allSettled(installedSkillIds.map((id) => uninstallSkillAsset(toolId, id)))
          await Promise.allSettled(uninstalledSkillIds.map((id) => installSkillAsset(toolId, id)))
          this.globalError = response.error?.message ?? translateKey('errors.skillSaveFailed')
          notifyError(this.globalError)
          return 'failed'
        }

        if (options.refreshSnapshot !== false) await this.hydrateFromSnapshot()
        if (options.refreshSkills !== false) await skillStore.hydrateFromSnapshot()
        if (options.closeBind) this.skillBindOpen = false
        this.globalError = ''
        if (options.notify) notifySuccess(translateKey('feedback.skillBindingsSaved'))
        return 'saved'
      } catch (error) {
        this.globalError = error instanceof Error ? error.message : translateKey('errors.skillSaveFailed')
        notifyError(this.globalError)
        return 'failed'
      } finally {
        this.skillBindLoading = false
      }
    },
    async saveSkillBinding() {
      if (!this.activeToolEnabled) return
      const nextSkillIds = Array.from(new Set(this.skillBindingDraft.selectedNewSkillIds))

      await this.saveToolSkillIdsAndSync(this.activeId, nextSkillIds, {
        closeBind: true,
        notify: true,
        refreshSkills: true,
      })
    },
    async loadGlobalPreview() {
      if (!this.activeToolEnabled) return false
      this.globalPreviewLoading = true
      try {
        const response = await previewGlobalOutput(this.activeId)
        if (!response.success || !response.data) {
          this.globalError = response.error?.message ? localizeMessage(response.error.message) : translateKey('errors.globalAgentsPreviewFailed')
          notifyError(this.globalError)
          return
        }
        this.globalPreview = response.data
        this.globalError = ''
        return true
      } catch (error) {
        this.globalError = error instanceof Error ? localizeMessage(error.message) : translateKey('errors.globalAgentsPreviewFailed')
        notifyError(this.globalError)
        return false
      } finally {
        this.globalPreviewLoading = false
      }
    },
    async applyGlobalAgents(confirmRisk = true) {
      if (!this.activeToolEnabled) return
      try {
        const response = await applyGlobalOutput(this.activeId, confirmRisk)
        if (!response.success || !response.data) {
          this.globalError = response.error?.message ? localizeMessage(response.error.message) : translateKey('errors.globalAgentsApplyFailed')
          notifyError(this.globalError)
          return
        }
        this.globalWriteResult = response.data
        this.globalError = response.data.message
        notifySuccess(this.globalError)
      } catch (error) {
        this.globalError = error instanceof Error ? localizeMessage(error.message) : translateKey('errors.globalAgentsApplyFailed')
        notifyError(this.globalError)
      }
    },
    async repairGlobalAgents(confirmRisk = true) {
      if (!this.activeToolEnabled) return
      try {
        const response = await repairGlobalOutput(this.activeId, confirmRisk)
        if (!response.success || !response.data) {
          this.globalError = response.error?.message ? localizeMessage(response.error.message) : translateKey('errors.globalAgentsRepairFailed')
          notifyError(this.globalError)
          return
        }
        this.globalWriteResult = response.data
        this.globalError = response.data.message
        notifySuccess(this.globalError)
      } catch (error) {
        this.globalError = error instanceof Error ? localizeMessage(error.message) : translateKey('errors.globalAgentsRepairFailed')
        notifyError(this.globalError)
      }
    },
    async clearCredential() {
      if (!this.activeToolEnabled) return
      try {
        const response = await clearToolCredentialState(this.activeId)
        if (!response.success) {
          this.verifyResult = {
            ok: false,
            state: response.error?.code ?? 'credential_clear_failed',
            detail: response.error?.message ?? 'Credential clear failed.',
            manualSteps: [translateKey(toolStepKey('toolCredentialClearStorage'))],
          }
          notifyError(this.verifyResult.detail)
          return
        }
      } catch (error) {
        this.verifyResult = {
          ok: false,
          state: 'credential_clear_failed',
          detail: error instanceof Error ? error.message : 'Credential clear failed.',
          manualSteps: [translateKey(toolStepKey('toolCredentialClearStorage'))],
        }
        notifyError(this.verifyResult.detail)
        return
      }
      this.draft.token = ''
      this.verifyResult = createActionResult()
      notifySuccess(translateKey('feedback.credentialCleared'))
      try {
        await this.hydrateDiagnostics()
      } catch {
        // Clearing the secret succeeded; diagnostics refresh is best effort.
      }
    },
    async scanDiagnostics() {
      await this.hydrateLibraryDiagnostics()
      notifySuccess(translateKey('feedback.diagnosticsScanCompleted'))
    },
    async exportDiagnostics() {
      try {
        const response = await exportLibraryDiagnostics()
        if (!response.success || !response.data) {
          this.diagnosticsError = response.error?.message ?? 'Diagnostics export failed.'
          notifyError(this.diagnosticsError)
          return
        }

        this.lastDiagnosticsExport = response.data
        this.diagnosticsError = response.data.message
        notifySuccess(translateKey('feedback.diagnosticsExported', { path: response.data.path }))
        await this.hydrateLibraryDiagnostics()
      } catch (error) {
        this.diagnosticsError = error instanceof Error ? error.message : 'Diagnostics export failed.'
        notifyError(this.diagnosticsError)
      }
    },
  },
})
