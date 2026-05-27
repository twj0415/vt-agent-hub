<script setup lang="ts">
  import { computed } from 'vue';
  import VTModal from '@/shared/components/feedback/VTModal.vue';
  import DiffViewer from '@/shared/components/forms/DiffViewer.vue';
  import { useProjectUi } from '../composables/useProjectUi';

  const { projectsStore, t, workspaceStore } = useProjectUi();

  // 输出弹窗：预览、应用和修复都先展示 AGENTS.md 差异。
  const loading = computed(() => projectsStore.previewLoading || projectsStore.scanLoading);
  const title = computed(() => (projectsStore.outputAction === 'repair' ? t('pages.projects.repair') : projectsStore.outputAction === 'apply' ? t('pages.projects.apply') : t('pages.projects.preview')));
  const okText = computed(() => (projectsStore.outputAction === 'repair' ? t('pages.projects.repair') : projectsStore.outputAction === 'apply' ? t('pages.projects.apply') : t('common.close')));
</script>

<template>
  <VTModal
    :open="projectsStore.previewOpen"
    :width="920"
    :title="title"
    :ok-text="okText"
    :cancel-text="t('common.close')"
    wrap-class-name="vt-modal-full"
    :body-style="{ minHeight: 'calc(100vh - 240px)', height: 'calc(100vh - 240px)', padding: '18px 20px', overflow: 'hidden' }"
    :ok-button-props="{ disabled: projectsStore.outputAction === 'apply' && !projectsStore.outputPreview?.canApply }"
    :loading="loading"
    @ok="projectsStore.confirmOutput(workspaceStore.activeToolId)"
    @close="projectsStore.setPreviewOpen(false)"
  >
    <div class="flex h-full min-h-0 flex-col">
      <DiffViewer :before="projectsStore.outputPreview?.beforeContent ?? ''" :after="projectsStore.outputPreview?.afterContent ?? ''" />
    </div>
  </VTModal>
</template>
