<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import type { FormInstance, Rule } from 'ant-design-vue/es/form'
import { useI18n } from 'vue-i18n'
import VTModal from '@/shared/components/feedback/VTModal.vue'
import SchemaForm from '@/shared/components/forms/SchemaForm.vue'
import { providerToolOptions } from '@/shared/providers'
import { useProvidersStore } from '@/shared/stores/providers'
import type { FormField } from '@/shared/types/ui'

const { t } = useI18n()
const providerStore = useProvidersStore()
const title = computed(() => (providerStore.draft.id == null ? t('pages.providers.create') : t('catalog.action.edit')))
const importActiveRole = ref('')
const formRef = ref<FormInstance>()
const importFormRef = ref<FormInstance>()

const categoryOptions = computed(() => providerStore.categoryOptions.map((item) => ({
  label: t(item.labelKey),
  value: item.value,
})))
const toolOptions = computed(() => providerToolOptions.map((item) => ({
  label: t(item.labelKey),
  value: item.value,
})))
const toolConfigFields = computed<FormField[]>(() =>
  (providerStore.activeToolSchema?.fields ?? []).map((field) => ({
    key: field.key,
    type: field.type,
    labelKey: field.labelKey,
    placeholderKey: field.placeholderKey,
    helpKey: field.helpKey,
    groupKey: field.groupKey,
    rows: field.rows,
    options: field.options,
    value: String(providerStore.draft[field.key as keyof typeof providerStore.draft] ?? ''),
  })),
)
const formRules = computed<Record<string, Rule[]>>(() => ({
  name: [{ required: true, whitespace: true, message: t('errors.providerNameRequired'), trigger: ['blur', 'change'] }],
  category: [{ required: true, message: t('errors.providerCategoryRequired'), trigger: 'change' }],
  toolId: [{ required: true, type: 'number', message: t('errors.providerToolConfigRequired'), trigger: 'change' }],
  model: [{ required: true, whitespace: true, message: t('errors.providerModelUnsupported'), trigger: ['blur', 'change'] }],
  reasoning: [{ required: true, whitespace: true, message: t('errors.providerReasoningUnsupported'), trigger: 'change' }],
  baseUrl: [
    { required: true, whitespace: true, message: t('errors.providerBaseUrlInvalid'), trigger: ['blur', 'change'] },
    {
      validator: async (_rule, value) => {
        const input = String(value ?? '')
        if (input.startsWith('http://') || input.startsWith('https://')) return
        throw new Error(t('errors.providerBaseUrlInvalid'))
      },
      trigger: ['blur', 'change'],
    },
  ],
}))
const importRules = computed<Record<string, Rule[]>>(() =>
  Object.fromEntries(providerStore.importPartSchemas.map((part) => [
    part.role,
    part.required
      ? [{ required: true, whitespace: true, message: t('errors.providerImportRequiredPart'), trigger: ['blur', 'change'] }]
      : [],
  ])),
)

function updateToolConfigField(payload: { key: string; value: string }) {
  if (payload.key === 'displayName' || payload.key === 'model' || payload.key === 'reasoning' || payload.key === 'baseUrl') {
    providerStore.setDraftField(payload.key, payload.value)
  }
}

watch(
  () => providerStore.importPartSchemas,
  (parts) => {
    importActiveRole.value = parts[0]?.role ?? ''
  },
  { immediate: true },
)

watch(
  () => providerStore.formOpen,
  async (open) => {
    if (!open) return
    await nextTick()
    formRef.value?.clearValidate()
  },
)

watch(
  () => providerStore.importOpen,
  async (open) => {
    if (!open) return
    await nextTick()
    importFormRef.value?.clearValidate()
  },
)

async function submitForm() {
  try {
    await formRef.value?.validate()
  } catch {
    return
  }
  await providerStore.saveDraft()
}

async function submitImport() {
  try {
    await importFormRef.value?.validate()
  } catch {
    return
  }
  await providerStore.importDraftFromPaste()
}
</script>

<template>
  <VTModal
    :open="providerStore.formOpen"
    :title="title"
    :ok-text="t('catalog.action.save')"
    :cancel-text="t('common.close')"
    :loading="providerStore.saving"
    :ok-button-props="{ disabled: providerStore.saving }"
    :width="980"
    @ok="submitForm"
    @close="providerStore.setFormOpen(false)"
  >
    <a-form ref="formRef" class="mx-auto max-w-[760px]" layout="vertical" :model="providerStore.draft" :rules="formRules">
      <div class="mb-4 flex flex-col gap-2 rounded-md border border-border/60 bg-muted/20 px-4 py-3 sm:flex-row sm:items-center sm:justify-between">
        <div class="text-xs leading-5 text-muted">
          {{ t('pages.providers.copyImportHint') }}
        </div>
        <a-button type="dashed" @click="providerStore.openImport()">
          {{ t('pages.providers.copyImport') }}
        </a-button>
      </div>

      <a-row :gutter="[16, 10]">
        <a-col :xs="24" :md="12">
          <a-form-item name="name" :label="t('pages.providers.providerName')" class="!mb-2">
            <a-input v-model:value="providerStore.draft.name" :placeholder="t('pages.providers.providerNamePlaceholder')" />
          </a-form-item>
        </a-col>

        <a-col :xs="24" :md="12">
          <a-form-item name="category" :label="t('pages.providers.providerCategoryLabel')" class="!mb-2">
            <a-select
              :value="providerStore.draft.category"
              :options="categoryOptions"
              :disabled="providerStore.draft.id !== null"
              @update:value="providerStore.setDraftField('category', String($event) as never)"
            />
          </a-form-item>
        </a-col>

        <a-col :span="24">
          <a-form-item name="toolId" :label="t('pages.providers.supportedTools')" class="!mb-2">
            <a-select
              :value="providerStore.draft.toolId"
              :options="toolOptions"
              @update:value="providerStore.setDraftTool(Number($event))"
            />
            <template #extra>
              <span class="text-xs leading-5 text-muted">
                {{ t('pages.providers.v1SupportNotice') }}
              </span>
            </template>
          </a-form-item>
        </a-col>
      </a-row>

      <SchemaForm class="mt-2" :fields="toolConfigFields" @update-field="updateToolConfigField" />

      <a-row class="mt-3 border-t border-line/70 pt-4" :gutter="[16, 10]">
        <a-col :xs="24" :md="12">
          <a-form-item name="credentialToken" :label="t('pages.providers.credentialLabel')" class="!mb-2">
            <a-input-password
              v-model:value="providerStore.draft.credentialToken"
              :placeholder="providerStore.draft.hasCredential ? t('pages.providers.credentialPlaceholderSaved') : t('pages.providers.credentialPlaceholder')"
            />
            <template #extra>
              <span class="text-xs leading-5 text-muted">
                {{ providerStore.draft.hasCredential ? t('pages.providers.credentialSavedHint') : t('pages.providers.credentialHint') }}
              </span>
            </template>
          </a-form-item>
        </a-col>

        <a-col :xs="24" :md="12">
          <a-form-item name="website" :label="t('pages.providers.website')" class="!mb-2">
            <a-input v-model:value="providerStore.draft.website" placeholder="https://example.com" />
          </a-form-item>
        </a-col>

        <a-col :span="24">
          <a-form-item name="note" :label="t('pages.providers.form.noteLabel')" class="!mb-2">
            <a-textarea v-model:value="providerStore.draft.note" :rows="4" />
          </a-form-item>
        </a-col>
      </a-row>
    </a-form>

    <VTModal
      :open="providerStore.importOpen"
      :title="t('pages.providers.importTitle')"
      :ok-text="t('pages.providers.applyImport')"
      :cancel-text="t('common.close')"
      :loading="providerStore.importing"
      :width="860"
      @ok="submitImport"
      @close="providerStore.setImportOpen(false)"
    >
      <a-form ref="importFormRef" class="mx-auto max-w-[720px]" layout="vertical" :model="providerStore.importParts" :rules="importRules">
        <a-form-item :label="t('pages.providers.supportedTools')" class="!mb-3">
          <a-select
            :value="providerStore.importToolId"
            :options="toolOptions"
            @update:value="providerStore.setImportTool(Number($event))"
          />
          <template #extra>
            <span class="text-xs leading-5 text-muted">
              {{ t('pages.providers.v1SupportNotice') }}
            </span>
          </template>
        </a-form-item>

        <a-tabs v-model:active-key="importActiveRole">
          <a-tab-pane
            v-for="part in providerStore.importPartSchemas"
            :key="part.role"
            :tab="t(part.labelKey)"
          >
            <a-form-item :name="part.role" :validate-status="part.required && !providerStore.importParts[part.role]?.trim() ? 'error' : ''" class="!mb-2">
              <a-textarea
                :value="providerStore.importParts[part.role] ?? ''"
                :rows="part.rows"
                :placeholder="t(part.placeholderKey)"
                @update:value="providerStore.setImportPart(part.role, String($event))"
              />
              <template v-if="part.helpKey" #extra>
                <span class="text-xs leading-5 text-muted">
                  {{ t(part.helpKey) }}
                </span>
              </template>
            </a-form-item>
          </a-tab-pane>
        </a-tabs>
      </a-form>
    </VTModal>
  </VTModal>
</template>
