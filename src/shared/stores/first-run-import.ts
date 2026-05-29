import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import {
  applyFirstRunImport,
  dismissFirstRunImport,
  getFirstRunImportStatus,
  previewFirstRunImport,
  resetFirstRunImportStatus,
} from '@/shared/api/tauri'
import type { FirstRunImportCandidate, FirstRunImportPreview, FirstRunImportStatus } from '@/shared/api/client'
import { resolveAppError, resolveUnknownError, translateKey } from '@/shared/i18n/translate'
import { notifyError, notifySuccess, notifyWarning } from '@/shared/utils/notify'
import { isTauriRuntime } from '@/shared/utils/runtime'
import { useHistoryStore } from './history'
import { useProvidersStore } from './providers'
import { useRuleStore } from './rules'
import { useSettingsStore } from './settings'
import { useSkillStore } from './skills'
import { useToolsStore } from './tools'

type CandidateGroup = 'all' | 'rule' | 'skill' | 'command' | 'prompt' | 'provider_preset' | 'attention'

function isAutoPromptStatus(status: string) {
  return status === 'pending'
}

function recommendedSelection(candidates: FirstRunImportCandidate[]) {
  return candidates
    .filter((candidate) => candidate.selectable && candidate.defaultSelected && candidate.status !== 'error')
    .map((candidate) => candidate.id)
}

export const useFirstRunImportStore = defineStore('firstRunImport', () => {
  const status = ref<FirstRunImportStatus | null>(null)
  const open = ref(false)
  const loading = ref(false)
  const applying = ref(false)
  const preview = ref<FirstRunImportPreview | null>(null)
  const selectedIds = ref<string[]>([])
  const activeGroup = ref<CandidateGroup>('all')
  const activeId = ref('')
  const conflictStrategy = ref<'rename' | 'skip' | 'overwrite'>('rename')
  const error = ref('')
  const hasCheckedThisSession = ref(false)

  const selectedCount = computed(() => selectedIds.value.length)
  const candidates = computed(() => preview.value?.candidates ?? [])
  const activeCandidate = computed(() => candidates.value.find((candidate) => candidate.id === activeId.value) ?? candidates.value[0] ?? null)
  const filteredCandidates = computed(() => {
    if (activeGroup.value === 'all') return candidates.value
    if (activeGroup.value === 'attention') {
      return candidates.value.filter((candidate) => candidate.status !== 'ready' || candidate.warnings.length > 0)
    }
    return candidates.value.filter((candidate) => candidate.assetType === activeGroup.value)
  })

  async function maybeOpenAfterBootstrap() {
    if (hasCheckedThisSession.value || !isTauriRuntime()) return
    hasCheckedThisSession.value = true
    loading.value = true
    try {
      const response = await getFirstRunImportStatus()
      if (!response.success || !response.data) {
        error.value = resolveAppError(response.error, 'errors.firstRunImportStatusFailed')
        notifyWarning(error.value)
        return
      }
      status.value = response.data
      if (!isAutoPromptStatus(response.data.status) || !response.data.shouldPrompt) return
      await loadPreview(true)
    } catch (cause) {
      error.value = resolveUnknownError(cause, 'errors.firstRunImportPreviewFailed')
      notifyWarning(error.value)
    } finally {
      loading.value = false
    }
  }

  async function loadPreview(autoOpen = false) {
    loading.value = true
    error.value = ''
    try {
      const response = await previewFirstRunImport()
      if (!response.success || !response.data) {
        error.value = resolveAppError(response.error, 'errors.firstRunImportPreviewFailed')
        notifyError(error.value)
        return
      }
      preview.value = response.data
      selectedIds.value = recommendedSelection(response.data.candidates)
      activeId.value = response.data.candidates[0]?.id ?? ''
      open.value = autoOpen
        ? response.data.candidates.length > 0 && response.data.status === 'pending'
        : true
    } catch (cause) {
      error.value = resolveUnknownError(cause, 'errors.firstRunImportPreviewFailed')
      notifyError(error.value)
    } finally {
      loading.value = false
    }
  }

  function openManualScan() {
    void loadPreview(false)
  }

  function setOpen(value: boolean) {
    open.value = value
  }

  function setActiveGroup(group: CandidateGroup) {
    activeGroup.value = group
    const first = filteredCandidates.value[0]
    if (first) activeId.value = first.id
  }

  function setActiveId(id: string) {
    activeId.value = id
  }

  function toggleCandidate(id: string) {
    const candidate = candidates.value.find((item) => item.id === id)
    if (!candidate?.selectable) return
    if (selectedIds.value.includes(id)) {
      selectedIds.value = selectedIds.value.filter((item) => item !== id)
    } else {
      selectedIds.value = [...selectedIds.value, id]
    }
  }

  function selectRecommended() {
    selectedIds.value = recommendedSelection(candidates.value)
  }

  function selectAllInGroup(group: CandidateGroup) {
    const ids = candidates.value
      .filter((candidate) => candidate.selectable)
      .filter((candidate) => group === 'all' || candidate.assetType === group)
      .map((candidate) => candidate.id)
    selectedIds.value = Array.from(new Set([...selectedIds.value, ...ids]))
  }

  function clearSelection() {
    selectedIds.value = []
  }

  async function applySelected(closeOnSuccess = true) {
    if (selectedIds.value.length === 0) {
      notifyWarning(translateKey('firstRunImport.feedback.noSelection'))
      return
    }
    applying.value = true
    error.value = ''
    try {
      const response = await applyFirstRunImport({
        selectedIds: selectedIds.value,
        conflictStrategy: conflictStrategy.value,
        confirm: true,
      })
      if (!response.success || !response.data) {
        error.value = resolveAppError(response.error, 'errors.firstRunImportApplyFailed')
        notifyError(error.value)
        return
      }
      if (closeOnSuccess) open.value = false
      status.value = { status: 'completed', shouldPrompt: false }
      await Promise.all([
        useRuleStore().hydrateFromSnapshot(),
        useSkillStore().hydrateFromSnapshot(),
        useProvidersStore().hydrate(),
        useSettingsStore().hydrateFromSnapshot(),
        useToolsStore().hydrateFromSnapshot(),
        useHistoryStore().hydrateFromSnapshot(),
      ])
      notifySuccess(translateKey('feedback.firstRunImportCompleted'))
      for (const warning of response.data.warnings) notifyWarning(warning)
    } catch (cause) {
      error.value = resolveUnknownError(cause, 'errors.firstRunImportApplyFailed')
      notifyError(error.value)
    } finally {
      applying.value = false
    }
  }

  async function dismiss(reason = 'user_dismissed') {
    loading.value = true
    try {
      const response = await dismissFirstRunImport('dismissed', reason)
      if (!response.success || !response.data) {
        error.value = resolveAppError(response.error, 'errors.firstRunImportDismissFailed')
        notifyError(error.value)
        return
      }
      status.value = response.data
      open.value = false
    } catch (cause) {
      error.value = resolveUnknownError(cause, 'errors.firstRunImportDismissFailed')
      notifyError(error.value)
    } finally {
      loading.value = false
    }
  }

  async function resetPromptStatus() {
    loading.value = true
    error.value = ''
    try {
      const response = await resetFirstRunImportStatus()
      if (!response.success || !response.data) {
        error.value = resolveAppError(response.error, 'errors.firstRunImportResetFailed')
        notifyError(error.value)
        return
      }
      status.value = response.data
      hasCheckedThisSession.value = false
      notifySuccess(translateKey('feedback.firstRunImportReset'))
    } catch (cause) {
      error.value = resolveUnknownError(cause, 'errors.firstRunImportResetFailed')
      notifyError(error.value)
    } finally {
      loading.value = false
    }
  }

  return {
    status,
    open,
    loading,
    applying,
    preview,
    selectedIds,
    selectedCount,
    activeGroup,
    activeId,
    activeCandidate,
    filteredCandidates,
    conflictStrategy,
    error,
    hasCheckedThisSession,
    maybeOpenAfterBootstrap,
    loadPreview,
    openManualScan,
    setOpen,
    setActiveGroup,
    setActiveId,
    toggleCandidate,
    selectRecommended,
    selectAllInGroup,
    clearSelection,
    applySelected,
    dismiss,
    resetPromptStatus,
  }
})
