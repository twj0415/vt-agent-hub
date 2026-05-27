<script setup lang="ts">
  import { computed, nextTick, ref, watch } from 'vue';
  import type { FormInstance, Rule } from 'ant-design-vue/es/form';
  import { useI18n } from 'vue-i18n';
  import CodePreview from '@/shared/components/feedback/CodePreview.vue';
  import VTModal from '@/shared/components/feedback/VTModal.vue';
  import PathPickerInput from '@/shared/components/forms/PathPickerInput.vue';
  import { useRuleImport } from '../composables/useRuleImport';

  const { t } = useI18n();
  const { categoryOptions, conflictOptions, handleFilePickerError, ruleStore, setImportField } = useRuleImport();
  const formRef = ref<FormInstance>();
  const formRules = computed<Record<string, Rule[]>>(() => ({
    sourcePath: [{ required: true, whitespace: true, message: t('errors.ruleImportFileRequired'), trigger: ['blur', 'change'] }],
    name: [{ required: true, whitespace: true, message: t('errors.ruleNameRequired'), trigger: ['blur', 'change'] }],
    categoryCode: [{ required: true, type: 'number', message: t('errors.ruleCategoryRequired'), trigger: 'change' }],
    summary: [{ required: true, whitespace: true, message: t('errors.ruleDescriptionRequired'), trigger: ['blur', 'change'] }],
    conflictStrategy: [{ required: true, message: t('errors.ruleConflictRequired'), trigger: 'change' }],
  }));

  watch(
    () => ruleStore.importOpen,
    async (open) => {
      if (!open) return;
      await nextTick();
      formRef.value?.clearValidate();
    }
  );

  async function submitImport() {
    try {
      await formRef.value?.validate();
    } catch {
      return;
    }
    await ruleStore.applyImport();
  }
</script>

<template>
  <!-- 规则导入弹窗：选择 markdown 文件并预览 frontmatter 解析结果。 -->
  <VTModal
    :open="ruleStore.importOpen"
    :title="t('pages.rules.importTitle')"
    :ok-text="t('pages.rules.importApply')"
    :cancel-text="t('common.close')"
    :loading="ruleStore.importLoading"
    :width="960"
    :body-style="{ maxHeight: '76vh', overflowY: 'auto' }"
    @ok="submitImport"
    @close="ruleStore.setImportOpen(false)"
  >
    <!-- template：左侧填写导入信息，右侧预览规则正文。 -->
    <div>
      <a-alert type="info" show-icon :message="t('pages.rules.importFrontmatterHint')" />

      <div class="mt-4 grid gap-4 lg:grid-cols-[minmax(0,0.92fr)_minmax(0,1.08fr)]">
        <a-form ref="formRef" layout="vertical" :model="ruleStore.importDraft" :rules="formRules" class="rounded-[16px] border border-line/60 bg-panel-strong/92 p-4 shadow-surface">
          <a-row :gutter="[16, 10]">
            <a-col :span="24">
              <a-form-item name="sourcePath" :label="t('pages.rules.importFile')" class="!mb-2">
                <PathPickerInput
                  :value="ruleStore.importDraft.sourcePath"
                  mode="file"
                  file-kind="markdown"
                  :placeholder="t('pages.rules.importFilePlaceholder')"
                  :button-text="t('common.selectFile')"
                  :disabled="ruleStore.importLoading"
                  @update:value="setImportField('sourcePath', $event)"
                  @error="handleFilePickerError"
                />
              </a-form-item>
            </a-col>

            <a-col :span="24">
              <a-form-item name="name" :label="t('pages.rules.importName')" class="!mb-2">
                <a-input :value="ruleStore.importDraft.name" :placeholder="t('pages.rules.importNamePlaceholder')" @update:value="setImportField('name', String($event))" />
              </a-form-item>
            </a-col>

            <a-col :xs="24" :md="12">
              <a-form-item name="categoryCode" :label="t('pages.rules.importCategory')" class="!mb-2">
                <a-select :value="ruleStore.importDraft.categoryCode" :options="categoryOptions" :placeholder="t('pages.rules.importCategoryPlaceholder')" @update:value="setImportField('categoryCode', Number($event))" />
              </a-form-item>
            </a-col>

            <a-col :xs="24" :md="12">
              <a-form-item name="conflictStrategy" :label="t('pages.rules.importConflict')" class="!mb-2">
                <a-select :value="ruleStore.importDraft.conflictStrategy" :options="conflictOptions.map((item) => ({ value: item.value, label: t(item.labelKey) }))" @update:value="setImportField('conflictStrategy', String($event))" />
              </a-form-item>
            </a-col>

            <a-col :span="24">
              <a-form-item class="!mb-2" name="summary" :label="t('pages.rules.importDescription')">
                <a-textarea :value="ruleStore.importDraft.summary" :placeholder="t('catalog.summaryPlaceholder')" :rows="4" @update:value="setImportField('summary', String($event))" />
              </a-form-item>
            </a-col>
          </a-row>
        </a-form>

        <section class="flex min-h-[420px] flex-col overflow-hidden rounded-[16px] border border-line/60 bg-panel/85 shadow-surface">
          <div class="flex items-center justify-between border-b border-line/80 px-4 py-3">
            <div class="text-sm font-semibold text-text">{{ t('pages.rules.importBodyPreview') }}</div>
            <span class="rounded-full border border-line/70 bg-panel/80 px-2.5 py-1 text-[11px] font-medium text-muted">
              {{ ruleStore.importDraft.body ? t('pages.rules.importBodyLength', { count: ruleStore.importDraft.body.length }) : t('common.empty') }}
            </span>
          </div>
          <CodePreview  :content="ruleStore.importDraft.body" :empty-text="t('pages.rules.importBodyPreviewEmpty')" size="xs" />
        </section>
      </div>
    </div>
  </VTModal>
</template>
