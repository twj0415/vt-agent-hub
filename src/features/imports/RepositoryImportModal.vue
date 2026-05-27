<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import type { FormInstance, Rule } from 'ant-design-vue/es/form'
import { useI18n } from 'vue-i18n'
import type { RepositoryImportAsset, RepositoryImportReport } from '@/shared/api/client'
import { pickFolderPath } from '@/shared/api/tauri'
import StatusBadge from '@/shared/components/feedback/StatusBadge.vue'
import VTModal from '@/shared/components/feedback/VTModal.vue'
import { repositoryConflictStrategyOptions } from '@/shared/taxonomy'

const props = defineProps<{
  open: boolean
  draft: {
    source: string
    branch: string
    conflictStrategy: 'skip' | 'rename' | 'overwrite'
  }
  loading: boolean
  report: RepositoryImportReport | null
}>()

const emit = defineEmits<{
  close: []
  preview: []
  apply: []
  field: [{ key: 'source' | 'branch' | 'conflictStrategy'; value: string }]
}>()

const { t } = useI18n()
const totalDetected = computed(() => props.report?.assets.length ?? 0)
const rowKey = (asset: RepositoryImportAsset) => `${asset.assetType}-${asset.sourcePath}`
const formRef = ref<FormInstance>()
const formRules = computed<Record<string, Rule[]>>(() => ({
  source: [{ required: true, whitespace: true, message: t('errors.repositorySourceRequired'), trigger: ['blur', 'change'] }],
  conflictStrategy: [{ required: true, message: t('errors.repositoryConflictRequired'), trigger: 'change' }],
}))

watch(
  () => props.open,
  async (open) => {
    if (!open) return
    await nextTick()
    formRef.value?.clearValidate()
  },
)

async function chooseRepositoryFolder() {
  const response = await pickFolderPath()
  if (response.success && response.data) {
    emit('field', { key: 'source', value: response.data })
  }
}

async function submitPreview() {
  try {
    await formRef.value?.validate()
  } catch {
    return
  }
  emit('preview')
}

async function submitApply() {
  try {
    await formRef.value?.validate()
  } catch {
    return
  }
  emit('apply')
}
</script>

<template>
  <VTModal
    :open="open"
    :title="t('catalog.repositoryImportTitle')"
    :ok-text="t('catalog.repositoryApply')"
    :cancel-text="t('common.close')"
    :loading="loading"
    :width="940"
    @ok="submitApply"
    @close="emit('close')"
  >
    <a-form ref="formRef" layout="vertical" :model="draft" :rules="formRules">
      <p class="m-0 text-sm leading-6 text-muted">{{ t('catalog.repositoryImportDesc') }}</p>
      <a-row class="mt-4" :gutter="[16, 10]">
        <a-col :span="24">
          <a-form-item name="source" :label="t('catalog.repositorySource')" class="!mb-2">
            <a-input-group compact>
              <a-input
                :value="draft.source"
                :placeholder="t('catalog.repositorySourcePlaceholder')"
                style="width: calc(100% - 104px)"
                @update:value="emit('field', { key: 'source', value: String($event) })"
              />
              <a-button style="width: 104px" @click="chooseRepositoryFolder">
                {{ t('common.selectFolder') }}
              </a-button>
            </a-input-group>
          </a-form-item>
        </a-col>

        <a-col :xs="24" :md="12">
          <a-form-item name="branch" :label="t('catalog.repositoryBranch')" class="!mb-2">
            <a-input
              :value="draft.branch"
              :placeholder="t('catalog.repositoryBranchPlaceholder')"
              @update:value="emit('field', { key: 'branch', value: String($event) })"
            />
          </a-form-item>
        </a-col>

        <a-col :xs="24" :md="12">
          <a-form-item name="conflictStrategy" :label="t('catalog.repositoryConflict')" class="!mb-2">
            <a-select
              :value="draft.conflictStrategy"
              :options="repositoryConflictStrategyOptions.map((option) => ({ value: option.value, label: t(option.labelKey) }))"
              @update:value="emit('field', { key: 'conflictStrategy', value: String($event) })"
            />
          </a-form-item>
        </a-col>
      </a-row>

      <div class="flex flex-wrap items-center gap-2 pt-2">
        <a-button :loading="loading" @click="submitPreview">{{ t('catalog.repositoryPreview') }}</a-button>
        <StatusBadge tone="planned" :label="`${t('catalog.repositoryDetected')}: ${totalDetected}`" />
      </div>
      <a-table
        v-if="report"
        class="mt-4"
        size="small"
        :pagination="false"
        :data-source="report.assets"
        :row-key="rowKey"
        :columns="[
          { title: t('catalog.repositoryAssetType'), dataIndex: 'assetType' },
          { title: t('catalog.repositoryAssetName'), dataIndex: 'name' },
          { title: t('catalog.repositoryAssetStatus'), dataIndex: 'status' },
        ]"
      />
    </a-form>
  </VTModal>
</template>
