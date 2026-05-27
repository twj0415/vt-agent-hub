<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import type { FormInstance, Rule } from 'ant-design-vue/es/form'
import { useI18n } from 'vue-i18n'
import VTModal from '@/shared/components/feedback/VTModal.vue'
import { useRuleForm } from '../composables/useRuleForm'

const { t } = useI18n()
const { categoryOptions, ruleStore, setDraftField } = useRuleForm()
const title = computed(() => (ruleStore.draft.id == null ? t('pages.rules.create') : t('catalog.action.edit')))
const formRef = ref<FormInstance>()
const formRules = computed<Record<string, Rule[]>>(() => ({
  name: [{ required: true, whitespace: true, message: t('errors.ruleNameRequired'), trigger: ['blur', 'change'] }],
  categoryCode: [{ required: true, type: 'number', message: t('errors.ruleCategoryRequired'), trigger: 'change' }],
  body: [{ required: true, whitespace: true, message: t('errors.ruleBodyRequired'), trigger: ['blur', 'change'] }],
}))

watch(
  () => ruleStore.formOpen,
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
  await ruleStore.saveDraft()
}
</script>

<template>
  <VTModal
    :open="ruleStore.formOpen"
    :title="title"
    :ok-text="t('catalog.action.save')"
    :cancel-text="t('common.close')"
    :width="940"
    @ok="submitForm"
    @close="ruleStore.setFormOpen(false)"
  >
    <a-form ref="formRef" class="mx-auto max-w-[720px]" layout="vertical" :model="ruleStore.draft" :rules="formRules">
      <a-row :gutter="[16, 10]">
        <a-col :xs="24" :md="14">
          <a-form-item name="name" :label="t('pages.rules.form.name')" class="!mb-2">
            <a-input v-model:value="ruleStore.draft.name" :placeholder="t('catalog.ruleNamePlaceholder')" />
          </a-form-item>
        </a-col>

        <a-col :xs="24" :md="10">
          <a-form-item name="categoryCode" :label="t('pages.rules.form.category')" class="!mb-2">
            <a-select :value="ruleStore.draft.categoryCode" :options="categoryOptions" @update:value="setDraftField('categoryCode', Number($event))" />
          </a-form-item>
        </a-col>

        <a-col :span="24">
          <a-form-item name="summary" :label="t('pages.rules.form.description')" class="!mb-2">
            <a-textarea v-model:value="ruleStore.draft.summary" :placeholder="t('catalog.summaryPlaceholder')" :rows="3" />
          </a-form-item>
        </a-col>

        <a-col :span="24">
          <a-form-item name="body" :label="t('pages.rules.form.body')" class="!mb-2">
            <a-textarea v-model:value="ruleStore.draft.body" :placeholder="t('catalog.bodyPlaceholder')" :rows="10" />
          </a-form-item>
        </a-col>
      </a-row>
    </a-form>
  </VTModal>
</template>
