<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import type { FormInstance, Rule } from 'ant-design-vue/es/form'
import { useI18n } from 'vue-i18n'
import VTModal from '@/shared/components/feedback/VTModal.vue'

const props = defineProps<{
  open: boolean
  draft: {
    name: string
    code: number
    summary: string
    categoryCode: number
    body: string
  }
  categoryOptions: Array<{ value: number; label: string }>
}>()

const emit = defineEmits<{
  close: []
  save: []
  field: [{ key: 'name' | 'code' | 'summary' | 'categoryCode' | 'body'; value: string | number }]
}>()

const { t } = useI18n()
const formRef = ref<FormInstance>()
const formRules = computed<Record<string, Rule[]>>(() => ({
  name: [{ required: true, whitespace: true, message: t('errors.skillNameRequired'), trigger: ['blur', 'change'] }],
  code: [{ required: true, type: 'number', message: t('errors.skillCodeUnsupported'), trigger: 'change' }],
  categoryCode: [{ required: true, type: 'number', message: t('errors.skillCategoryRequired'), trigger: 'change' }],
  body: [{ required: true, whitespace: true, message: t('errors.skillBodyRequired'), trigger: ['blur', 'change'] }],
}))

watch(
  () => props.open,
  async (open) => {
    if (!open) return
    await nextTick()
    formRef.value?.clearValidate()
  },
)

async function submitForm() {
  try {
    await formRef.value?.validate()
  } catch {
    return
  }
  emit('save')
}
</script>

<template>
  <VTModal
    :open="open"
    :title="t('catalog.skillModalTitle')"
    :ok-text="t('catalog.action.save')"
    :cancel-text="t('common.close')"
    :width="940"
    @ok="submitForm"
    @close="emit('close')"
  >
    <a-form ref="formRef" class="mx-auto max-w-[760px]" layout="vertical" :model="draft" :rules="formRules">
      <a-row :gutter="[16, 10]">
        <a-col :xs="24" :lg="12">
          <a-form-item name="name" :label="t('pages.skills.form.name')" class="!mb-2">
            <a-input
              :value="draft.name"
              :placeholder="t('catalog.skillNamePlaceholder')"
              @update:value="emit('field', { key: 'name', value: String($event) })"
            />
          </a-form-item>
        </a-col>

        <a-col :xs="24" :sm="14" :lg="8">
          <a-form-item name="categoryCode" :label="t('pages.skills.form.category')" class="!mb-2">
            <a-select
              :value="draft.categoryCode"
              :options="categoryOptions"
              @update:value="emit('field', { key: 'categoryCode', value: Number($event) })"
            />
          </a-form-item>
        </a-col>

        <a-col :xs="24" :sm="10" :lg="4">
          <a-form-item name="code" :label="t('common.code')" class="!mb-2">
            <a-input-number
              :value="draft.code"
              class="!w-full"
              @update:value="emit('field', { key: 'code', value: Number($event ?? 0) })"
            />
          </a-form-item>
        </a-col>

        <a-col :span="24">
          <a-form-item name="summary" :label="t('pages.skills.form.summary')" class="!mb-2">
            <a-textarea
              :value="draft.summary"
              :placeholder="t('catalog.summaryPlaceholder')"
              :rows="3"
              @update:value="emit('field', { key: 'summary', value: String($event) })"
            />
          </a-form-item>
        </a-col>

        <a-col :span="24">
          <a-form-item name="body" :label="t('pages.skills.form.body')" class="!mb-2">
            <a-textarea
              :value="draft.body"
              :placeholder="t('catalog.skillBodyPlaceholder')"
              :rows="10"
              @update:value="emit('field', { key: 'body', value: String($event) })"
            />
          </a-form-item>
        </a-col>
      </a-row>
    </a-form>
  </VTModal>
</template>
