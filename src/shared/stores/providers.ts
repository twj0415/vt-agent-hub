import { computed, ref } from 'vue'
import { defineStore } from 'pinia'
import {
  applyProviderToLiveConfig,
  deleteProvider,
  detectProviderLiveDrift,
  duplicateProvider,
  importProviderConfig,
  listProviders,
  previewProviderApply,
  saveProvider,
} from '@/shared/api/tauri'
import { entityStateFromCode, normalizeProviderCheckStatus, type ProviderCheckStatus } from '@/shared/constants/status'
import { resolveAppError, resolveUnknownError, translateKey } from '@/shared/i18n/translate'
import {
  defaultProviderCategory,
  getProviderImportParts,
  getProviderToolSchema,
  isSupportedProviderCategory,
  isSupportedProviderModel,
  isSupportedProviderReasoning,
  providerCategoryOptions,
  providerToolOptions,
  type ProviderCategory,
} from '@/shared/providers'
import { toolIds, toolRegistry, type ToolId } from '@/shared/tool-registry'
import { notifyError, notifySuccess, notifyWarning } from '@/shared/utils/notify'
import { useToolContextStore } from './tool-context'
import type { ProviderApplyPreview, ProviderApplyResult, ProviderLiveDrift, ProviderSummary, ProviderToolConfig } from '@/shared/api/client'

type ProviderDraft = {
  id: number | null
  name: string
  category: ProviderCategory
  website: string
  note: string
  toolId: ToolId
  configId: number | null
  displayName: string
  model: string
  reasoning: string
  baseUrl: string
  credentialRef: string
  credentialToken: string
  hasCredential: boolean
  configJson: Record<string, unknown>
}

export type ProviderItem = ProviderSummary

export type ProviderCardItem = {
  id: string
  providerId: number
  configId: number | null
  name: string
  category: string
  toolId: ToolId | null
  toolTags: string[]
  toolTitle: string
  model: string
  baseUrl: string
  status: ProviderCheckStatus
  state: 'missing' | 'ready' | 'error' | 'planned'
  active: boolean
}

const defaultSchema = getProviderToolSchema(toolIds.codex, 1)

function schemaDefaults(toolId: number) {
  const schema = getProviderToolSchema(toolId, 1) ?? defaultSchema
  return Object.fromEntries((schema?.fields ?? []).map((field) => [field.key, field.defaultValue]))
}

function createDraft(): ProviderDraft {
  const values = schemaDefaults(toolIds.codex)
  return {
    id: null,
    name: '',
    category: defaultProviderCategory,
    website: '',
    note: '',
    toolId: toolIds.codex,
    configId: null,
    displayName: String(values.displayName ?? ''),
    model: String(values.model ?? 'gpt-5.5'),
    reasoning: String(values.reasoning ?? 'medium'),
    baseUrl: String(values.baseUrl ?? 'https://api.openai.com/v1'),
    credentialRef: '',
    credentialToken: '',
    hasCredential: false,
    configJson: {},
  }
}

function firstConfigForTool(provider: ProviderSummary, toolId: number) {
  return provider.configs.find((config) => config.toolId === toolId) ?? null
}

function requireConfig(config: ProviderToolConfig | null) {
  if (!config) {
    notifyWarning(translateKey('errors.providerToolConfigRequired'))
    return false
  }
  return true
}

export const useProvidersStore = defineStore('providers', () => {
  const toolContextStore = useToolContextStore()
  const defaultToolId = computed(() => toolContextStore.activeToolId)
  const items = ref<ProviderItem[]>([])
  const activeId = ref(0)
  const filterToolId = ref<ToolId>(toolIds.codex)
  const formOpen = ref(false)
  const importOpen = ref(false)
  const applyOpen = ref(false)
  const loading = ref(false)
  const saving = ref(false)
  const importing = ref(false)
  const draft = ref<ProviderDraft>(createDraft())
  const importToolId = ref<ToolId>(toolIds.codex)
  const importParts = ref<Record<string, string>>({})
  const applyPreviewResult = ref<ProviderApplyPreview | null>(null)
  const applyResult = ref<ProviderApplyResult | null>(null)
  const driftByConfigId = ref<Record<number, ProviderLiveDrift>>({})
  const ignoredDriftConfigIds = ref<number[]>([])
  const driftChecking = ref(false)
  const actionError = ref('')

  const activeToolId = computed(() => toolContextStore.activeToolId)
  const categoryOptions = computed(() => providerCategoryOptions)
  const importPartSchemas = computed(() => getProviderImportParts(importToolId.value))
  const activeToolSchema = computed(() => getProviderToolSchema(draft.value.toolId, 1) ?? getProviderToolSchema(defaultToolId.value, 1) ?? defaultSchema)
  const reasoningOptions = computed(() => activeToolSchema.value?.fields.find((field) => field.key === 'reasoning')?.options ?? [])
  const modelOptions = computed(() => activeToolSchema.value?.fields.find((field) => field.key === 'model')?.options ?? [])
  const activeItem = computed(() => items.value.find((item) => item.id === activeId.value) ?? null)
  const currentCards = computed<ProviderCardItem[]>(() =>
    items.value.flatMap((provider) => provider.configs
      .filter((config) => config.toolId === filterToolId.value)
      .map((config) => {
        const tool = toolRegistry.find((item) => item.id === config.toolId)
        const toolLabel = tool ? translateKey(tool.nameKey) : String(config.toolId)

        return {
          id: `${provider.id}:${config.id}`,
          providerId: provider.id,
          configId: config.id,
          name: provider.name,
          category: provider.category,
          toolId: config.toolId as ToolId,
          toolTags: [tool?.key ?? String(config.toolId)],
          toolTitle: toolLabel,
          model: config.model || translateKey('common.empty'),
          baseUrl: config.baseUrl || translateKey('common.empty'),
          status: normalizeProviderCheckStatus(config.lastCheckStatus),
          state: entityStateFromCode(config.state),
          active: Boolean(config.isActive),
        }
      })),
  )
  const applyPreview = computed(() => applyPreviewResult.value)
  const activeDrift = computed(() => {
    const config = currentCards.value.find((item) => item.active)?.configId
    if (!config || ignoredDriftConfigIds.value.includes(config)) return null
    const drift = driftByConfigId.value[config]
    return drift?.hasDrift ? drift : null
  })

  async function hydrate() {
    loading.value = true
    try {
      const response = await listProviders(null)
      if (!response.success || !response.data) {
        actionError.value = resolveAppError(response.error, 'errors.providerListFailed')
        notifyError(actionError.value)
        return
      }
      items.value = response.data
      activeId.value = items.value.find((item) => firstConfigForTool(item, activeToolId.value)?.isActive)?.id
        ?? items.value.find((item) => item.id === activeId.value)?.id
        ?? items.value[0]?.id
        ?? 0
      void checkActiveLiveDrift()
    } catch (error) {
      actionError.value = resolveUnknownError(error, 'errors.providerListFailed')
      notifyError(actionError.value)
    } finally {
      loading.value = false
    }
  }

  function openCreate() {
    draft.value = createDraft()
    setDraftTool(activeToolId.value)
    formOpen.value = true
    actionError.value = ''
  }

  function openImport(toolId?: number) {
    const nextToolId = toolId && providerToolOptions.some((item) => item.value === toolId)
      ? toolId
      : draft.value.toolId || activeToolId.value
    importToolId.value = nextToolId as ToolId
    resetImportParts()
    importOpen.value = true
    actionError.value = ''
  }

  function openEdit(id: number, configId?: number | null) {
    const provider = items.value.find((item) => item.id === id)
    if (!provider) return
    const config = provider.configs.find((item) => item.id === configId) ?? firstConfigForTool(provider, activeToolId.value) ?? provider.configs[0] ?? null
    const values = schemaDefaults(config?.toolId ?? activeToolId.value)
    draft.value = {
      id: provider.id,
      name: provider.name,
      category: provider.category as ProviderCategory,
      website: provider.website,
      note: provider.note,
      toolId: (config?.toolId ?? activeToolId.value) as ToolId,
      configId: config?.id ?? null,
      displayName: config?.displayName ?? provider.name,
      model: config?.model ?? String(values.model ?? 'gpt-5.5'),
      reasoning: config?.reasoning ?? String(values.reasoning ?? 'medium'),
      baseUrl: config?.baseUrl ?? String(values.baseUrl ?? 'https://api.openai.com/v1'),
      credentialRef: config?.credentialRef ?? '',
      credentialToken: '',
      hasCredential: Boolean(config?.hasCredential),
      configJson: config?.configJson ?? {},
    }
    formOpen.value = true
    actionError.value = ''
  }

  function setFormOpen(value: boolean) {
    formOpen.value = value
  }

  function setImportOpen(value: boolean) {
    importOpen.value = value
  }

  function setFilterToolId(value: ToolId) {
    filterToolId.value = value
    void checkActiveLiveDrift()
  }

  function setApplyOpen(value: boolean) {
    applyOpen.value = value
  }

  function setDraftField<K extends keyof ProviderDraft>(key: K, value: ProviderDraft[K]) {
    draft.value[key] = value
  }

  function setDraftTool(toolId: number) {
    if (!providerToolOptions.some((item) => item.value === toolId)) return
    draft.value.toolId = toolId as ToolId
    const schema = getProviderToolSchema(toolId, 1)
    if (!schema) return
    const values = schemaDefaults(toolId)
    if (!isSupportedProviderModel(toolId, draft.value.model)) {
      draft.value.model = String(values.model ?? '')
    }
    if (!isSupportedProviderReasoning(draft.value.reasoning)) {
      draft.value.reasoning = String(values.reasoning ?? '')
    }
    if (!draft.value.baseUrl.trim()) {
      draft.value.baseUrl = String(values.baseUrl ?? '')
    }
  }

  function setImportTool(toolId: number) {
    if (!providerToolOptions.some((item) => item.value === toolId)) return
    importToolId.value = toolId as ToolId
    resetImportParts()
  }

  function setImportPart(role: string, content: string) {
    importParts.value = {
      ...importParts.value,
      [role]: content,
    }
  }

  function resetImportParts() {
    importParts.value = Object.fromEntries(getProviderImportParts(importToolId.value).map((part) => [part.role, '']))
  }

  function validateImportParts() {
    const missing = importPartSchemas.value.find((part) => part.required && !importParts.value[part.role]?.trim())
    if (missing) {
      actionError.value = translateKey('errors.providerImportRequiredPart')
      notifyWarning(actionError.value)
      return false
    }
    return true
  }

  async function importDraftFromPaste() {
    if (!validateImportParts()) return
    importing.value = true
    try {
      const response = await importProviderConfig({
        toolId: importToolId.value,
        parts: importPartSchemas.value.map((part) => ({
          role: part.role,
          content: importParts.value[part.role] ?? '',
        })),
      })
      if (!response.success || !response.data) {
        actionError.value = resolveAppError(response.error, 'errors.providerImportFailed')
        notifyError(actionError.value)
        return
      }

      const imported = response.data
      draft.value = {
        id: null,
        name: imported.name,
        category: isSupportedProviderCategory(imported.category) ? imported.category : defaultProviderCategory,
        website: imported.website,
        note: imported.note,
        toolId: imported.toolId as ToolId,
        configId: null,
        displayName: imported.displayName,
        model: imported.model,
        reasoning: imported.reasoning,
        baseUrl: imported.baseUrl,
        credentialRef: imported.credentialRef,
        credentialToken: imported.credentialToken ?? '',
        hasCredential: imported.hasCredential,
        configJson: imported.configJson ?? {},
      }
      importOpen.value = false
      formOpen.value = true
      actionError.value = ''
      notifySuccess(translateKey('feedback.providerImported'))
    } catch (error) {
      actionError.value = resolveUnknownError(error, 'errors.providerImportFailed')
      notifyError(actionError.value)
    } finally {
      importing.value = false
    }
  }

  async function saveDraft() {
    if (!draft.value.name.trim()) {
      actionError.value = translateKey('errors.providerNameRequired')
      notifyWarning(actionError.value)
      return
    }
    if (!isSupportedProviderCategory(draft.value.category)) {
      actionError.value = translateKey('errors.providerCategoryRequired')
      notifyWarning(actionError.value)
      return
    }
    if (!isSupportedProviderModel(draft.value.toolId, draft.value.model)) {
      actionError.value = translateKey('errors.providerModelUnsupported')
      notifyWarning(actionError.value)
      return
    }
    if (!isSupportedProviderReasoning(draft.value.reasoning)) {
      actionError.value = translateKey('errors.providerReasoningUnsupported')
      notifyWarning(actionError.value)
      return
    }
    const allowsDisplayUrl = draft.value.toolId === toolIds.claude && /^(bedrock|vertex):\/\//.test(draft.value.baseUrl)
    if (!allowsDisplayUrl && !draft.value.baseUrl.startsWith('http://') && !draft.value.baseUrl.startsWith('https://')) {
      actionError.value = translateKey('errors.providerBaseUrlInvalid')
      notifyWarning(actionError.value)
      return
    }

    saving.value = true
    try {
      const response = await saveProvider({
        id: draft.value.id,
        name: draft.value.name,
        category: draft.value.category,
        website: draft.value.website,
        note: draft.value.note,
        toolConfigs: [{
          id: draft.value.configId,
          toolId: draft.value.toolId,
          schemaVersion: 1,
          displayName: draft.value.displayName || draft.value.name,
          model: draft.value.model,
          reasoning: draft.value.reasoning,
          baseUrl: draft.value.baseUrl,
          credentialRef: draft.value.credentialRef || null,
          credentialToken: draft.value.credentialToken || null,
          configJson: draft.value.configJson,
        }],
      })
      if (!response.success || !response.data) {
        actionError.value = resolveAppError(response.error, 'errors.providerSaveFailed')
        notifyError(actionError.value)
        return
      }
      await hydrate()
      activeId.value = response.data.id
      formOpen.value = false
      actionError.value = ''
      notifySuccess(translateKey('feedback.providerSaved'))
    } catch (error) {
      actionError.value = resolveUnknownError(error, 'errors.providerSaveFailed')
      notifyError(actionError.value)
    } finally {
      saving.value = false
    }
  }

  async function deleteItem(id: number) {
    try {
      const response = await deleteProvider(id)
      if (!response.success) {
        actionError.value = resolveAppError(response.error, 'errors.providerDeleteFailed')
        notifyError(actionError.value)
        return
      }
      await hydrate()
      notifySuccess(translateKey('feedback.providerDeleted'))
    } catch (error) {
      actionError.value = resolveUnknownError(error, 'errors.providerDeleteFailed')
      notifyError(actionError.value)
    }
  }

  async function duplicateItem(id: number) {
    try {
      const response = await duplicateProvider(id)
      if (!response.success || !response.data) {
        actionError.value = resolveAppError(response.error, 'errors.providerDuplicateFailed')
        notifyError(actionError.value)
        return
      }
      await hydrate()
      activeId.value = response.data.id
      notifySuccess(translateKey('feedback.providerDuplicated'))
    } catch (error) {
      actionError.value = resolveUnknownError(error, 'errors.providerDuplicateFailed')
      notifyError(actionError.value)
    }
  }

  async function openApplyPreview(providerId: number, configId?: number | null) {
    const provider = items.value.find((item) => item.id === providerId)
    const config = provider
      ? provider.configs.find((item) => item.id === configId) ?? firstConfigForTool(provider, activeToolId.value)
      : null
    if (!config) {
      requireConfig(config)
      return
    }
    try {
      const response = await previewProviderApply(config.id)
      if (!response.success || !response.data) {
        actionError.value = resolveAppError(response.error, 'errors.providerPreviewApplyFailed')
        notifyError(actionError.value)
        return
      }
      activeId.value = providerId
      applyPreviewResult.value = response.data
      applyOpen.value = true
      actionError.value = ''
    } catch (error) {
      actionError.value = resolveUnknownError(error, 'errors.providerPreviewApplyFailed')
      notifyError(actionError.value)
    }
  }

  async function checkLiveDrift(configId: number) {
    driftChecking.value = true
    try {
      const response = await detectProviderLiveDrift(configId)
      if (!response.success || !response.data) {
        actionError.value = resolveAppError(response.error, 'errors.providerDriftCheckFailed')
        return null
      }
      driftByConfigId.value = {
        ...driftByConfigId.value,
        [configId]: response.data,
      }
      return response.data
    } catch (error) {
      actionError.value = resolveUnknownError(error, 'errors.providerDriftCheckFailed')
      return null
    } finally {
      driftChecking.value = false
    }
  }

  async function checkActiveLiveDrift() {
    const configId = currentCards.value.find((item) => item.active)?.configId
    if (!configId) return null
    return checkLiveDrift(configId)
  }

  function ignoreActiveDrift() {
    const configId = activeDrift.value?.configId
    if (!configId) return
    ignoredDriftConfigIds.value = Array.from(new Set([...ignoredDriftConfigIds.value, configId]))
    notifySuccess(translateKey('feedback.providerDriftIgnored'))
  }

  function showActiveDriftDiff() {
    const drift = activeDrift.value
    if (!drift) return
    applyPreviewResult.value = {
      toolId: drift.toolId,
      providerId: drift.providerId,
      configId: drift.configId,
      providerName: drift.providerName,
      targetPath: drift.targetPath,
      targetExists: drift.targetExists,
      backupRequired: drift.files.some((file) => file.backupRequired),
      beforeContent: drift.files[0]?.beforeContent ?? '',
      afterContent: drift.files[0]?.afterContent ?? '',
      diff: drift.files[0]?.diff ?? '',
      files: drift.files,
      warning: drift.warning,
    }
    applyOpen.value = true
  }

  async function applyActiveDriftProvider() {
    const drift = activeDrift.value
    if (!drift) return
    applyPreviewResult.value = {
      toolId: drift.toolId,
      providerId: drift.providerId,
      configId: drift.configId,
      providerName: drift.providerName,
      targetPath: drift.targetPath,
      targetExists: drift.targetExists,
      backupRequired: drift.files.some((file) => file.backupRequired),
      beforeContent: drift.files[0]?.beforeContent ?? '',
      afterContent: drift.files[0]?.afterContent ?? '',
      diff: drift.files[0]?.diff ?? '',
      files: drift.files,
      warning: drift.warning,
    }
    await applyToLiveConfig(true)
  }

  async function applyToLiveConfig(confirmRisk = true) {
    const configId = applyPreviewResult.value?.configId
    if (!configId) return
    try {
      const response = await applyProviderToLiveConfig(configId, confirmRisk)
      if (!response.success || !response.data) {
        actionError.value = resolveAppError(response.error, 'errors.providerApplyFailed')
        notifyError(actionError.value)
        return
      }
      applyResult.value = response.data
      driftByConfigId.value = {
        ...driftByConfigId.value,
        [configId]: {
          toolId: response.data.toolId,
          providerId: response.data.providerId,
          configId: response.data.configId,
          providerName: applyPreviewResult.value?.providerName ?? '',
          hasDrift: false,
          targetPath: response.data.targetPath,
          targetExists: true,
          files: [],
          warning: undefined,
        },
      }
      ignoredDriftConfigIds.value = ignoredDriftConfigIds.value.filter((id) => id !== configId)
      await hydrate()
      applyOpen.value = false
      notifySuccess(translateKey('feedback.providerApplied'))
    } catch (error) {
      actionError.value = resolveUnknownError(error, 'errors.providerApplyFailed')
      notifyError(actionError.value)
    }
  }

  return {
    actionError,
    activeId,
    activeItem,
    activeToolId,
    activeToolSchema,
    applyOpen,
    activeDrift,
    applyActiveDriftProvider,
    applyPreview,
    applyPreviewResult,
    applyResult,
    applyToLiveConfig,
    categoryOptions,
    checkActiveLiveDrift,
    checkLiveDrift,

    currentCards,
    deleteItem,
    draft,
    driftByConfigId,
    driftChecking,
    duplicateItem,
    filterToolId,
    formOpen,
    hydrate,
    ignoreActiveDrift,
    importDraftFromPaste,
    importOpen,
    importing,
    importPartSchemas,
    importParts,
    importToolId,
    items,
    loading,
    modelOptions,
    openApplyPreview,
    openCreate,
    openEdit,
    openImport,
    reasoningOptions,
    saveDraft,
    saving,
    setApplyOpen,
    setDraftField,
    setDraftTool,
    setFilterToolId,
    setFormOpen,
    setImportOpen,
    setImportPart,
    setImportTool,
    showActiveDriftDiff,
  }
})
