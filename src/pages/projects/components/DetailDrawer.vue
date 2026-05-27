<script setup lang="ts">
  import { computed, ref, watch } from 'vue';
  import { useI18n } from 'vue-i18n';
  import CodePreview from '@/shared/components/feedback/CodePreview.vue';
  import VTEntityDetailDrawer from '@/shared/components/feedback/VTEntityDetailDrawer.vue';
  import type { DetailField, DetailHeader, DetailTab } from '@/shared/components/feedback/VTEntityDetailDrawer.vue';
  import RuleBindList from '@/shared/components/rules/RuleBindList.vue';
  import { useConfirm } from '@/shared/composables/useConfirm';
  import { useProjectUi } from '../composables/useProjectUi';

  const { t, commonRules, projectsStore, selectedProject, selectedProjectCard, workspaceStore } = useProjectUi();
  const { confirmAction } = useConfirm();
  const bottomTab = ref<'rules' | 'preview'>('rules');
  const activeToolId = computed(() => workspaceStore.activeToolId as 101 | 102 | 103);

  const loading = computed(() => projectsStore.listLoading || projectsStore.scanLoading || projectsStore.bindLoading || projectsStore.deleteLoading);
  const previewLoading = computed(() => projectsStore.previewLoading || projectsStore.scanLoading);
  const open = computed({
    get: () => projectsStore.detailOpen,
    set: (value) => projectsStore.setDetailOpen(value),
  });
  const preview = computed(() => projectsStore.outputPreview);
  const previewContent = computed(() => preview.value?.afterContent ?? '');
  const previewCanLoad = computed(() => Boolean(selectedProject.value && activeToolId.value));

  // 切到预览 tab 时按需拉取，避免每次打开抽屉都重新请求。
  watch(
    () => [open.value, bottomTab.value, selectedProject.value?.id] as const,
    async ([drawerOpen, tab]) => {
      if (!drawerOpen || tab !== 'preview' || !previewCanLoad.value || previewLoading.value || preview.value) return;
      await projectsStore.loadOutputPreview(activeToolId.value);
    },
    { immediate: true }
  );

  watch(
    () => selectedProject.value?.id,
    () => {
      bottomTab.value = 'rules';
    }
  );

  const headerFields = computed<DetailField[]>(() => {
    if (!selectedProject.value) return [];
    return [
      { label: t('common.path'), value: selectedProject.value.path, mono: true },
      { label: t('pages.projects.projectType'), value: selectedProjectCard.value?.projectTypeLabel ?? '-' },
      { label: t('pages.projects.noteLabel'), value: selectedProjectCard.value?.statusNote ?? '-' },
    ];
  });

  const header = computed<DetailHeader>(() => ({
    name: selectedProject.value?.name ?? '',
    status: selectedProjectCard.value
      ? { tone: selectedProjectCard.value.statusTone, label: selectedProjectCard.value.statusLabel ?? '-' }
      : undefined,
    fields: headerFields.value,
  }));

  const tabs = computed<DetailTab[]>(() => [
    { key: 'rules', label: t('nav.rules') },
    { key: 'preview', label: t('common.preview') },
  ]);

  function confirmUnbind(rule: { id: number; name: string }) {
    confirmAction({
      danger: true,
      okText: t('pages.projects.binding.unbindAndApply'),
      content: t('pages.projects.binding.unbindConfirmContent', { name: rule.name }),
      onOk: () => projectsStore.unbindRule(rule.id, activeToolId.value),
    });
  }
</script>

<template>
  <VTEntityDetailDrawer
    v-if="selectedProject"
    v-model:open="open"
    v-model:active-tab="bottomTab"
    :title="t('pages.projects.drawerTitle')"
    :loading="loading"
    :header="header"
    :tabs="tabs"
  >
    <template #tab-rules>
      <div class="min-h-0 flex-1 overflow-auto">
        <RuleBindList
          :rules="commonRules"
          :empty-text="t('pages.projects.binding.empty')"
          :unbind-text="t('common.unbind')"
          :loading="loading"
          @unbind="confirmUnbind($event)"
        />
      </div>
    </template>

    <template #tab-preview>
      <div class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-[12px]">
        <CodePreview v-if="preview || !previewLoading" :content="previewContent" :empty-text="t('pages.projects.drawer.previewIdle')" />
      </div>
    </template>
  </VTEntityDetailDrawer>
</template>
