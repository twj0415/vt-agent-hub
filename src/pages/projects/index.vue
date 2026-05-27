<script setup lang="ts">
  import { computed } from 'vue';
  import { DeleteOutlined, ImportOutlined, ToolOutlined } from '@ant-design/icons-vue';
  import { useConfirm } from '@/shared/composables/useConfirm';
  import type { CardMoreMenuItem } from '@/shared/types/ui';
  import EmptyState from '@/shared/components/feedback/EmptyState.vue';
  import PageHeader from '@/shared/components/shell/PageHeader.vue';
  import BindModal from './components/BindModal.vue';
  import DetailDrawer from './components/DetailDrawer.vue';
  import FormModal from './components/FormModal.vue';
  import OutputModal from './components/OutputModal.vue';
  import ProjectCard from './components/ProjectCard.vue';
  import { useProjectsWorkbench } from './composables/useProjectsWorkbench';

  const { t, manageRules, openProject, projectCards, projectsStore, repairProject } = useProjectsWorkbench();
  const { confirmAction } = useConfirm();

  const projectListBusy = computed(() => projectsStore.scanLoading);

  function confirmProjectDelete(id: number) {
    const project = projectCards.value.find((item) => item.id === id);
    confirmAction({
      danger: true,
      okText: t('common.delete'),
      content: t('pages.projects.confirmDeleteContent', { name: project?.name ?? `#${id}` }),
      onOk: () => projectsStore.deleteItem(id),
    });
  }

  function handleProjectMore(key: string, id: number) {
    if (key === 'repair') repairProject(id);
    if (key === 'rules') manageRules(id);
    if (key === 'edit') projectsStore.openEdit(id);
    if (key === 'delete') confirmProjectDelete(id);
  }

  function projectMoreItems(item: { needsRepair: boolean; canManageRules: boolean }): CardMoreMenuItem[] {
    const busy = projectListBusy.value || projectsStore.previewLoading || projectsStore.bindLoading || projectsStore.deleteLoading;
    return [
      { key: 'repair', label: t('pages.projects.repair'), icon: ToolOutlined, disabled: busy || !item.needsRepair },
      { key: 'rules', label: t('common.bind'), disabled: busy || !item.canManageRules },
      { key: 'delete', label: t('common.delete'), icon: DeleteOutlined, danger: true, disabled: busy },
    ];
  }
</script>

<template>
  <div class="workbench-page">
    <PageHeader :title="t('pages.projects.title')" :count="projectCards.length">
      <a-button type="primary" :disabled="projectsStore.listLoading" @click="projectsStore.openImport('local')">
        <template #icon><ImportOutlined /></template>
        {{ t('pages.projects.importTitle') }}
      </a-button>
    </PageHeader>

    <div class="workbench-body">
      <div v-if="projectCards.length" class="entity-grid">
        <ProjectCard
          v-for="item in projectCards"
          :key="item.id"
          :item="item"
          :busy="projectListBusy"
          :more-items="projectMoreItems(item)"
          @open="openProject"
          @edit="projectsStore.openEdit"
          @more="handleProjectMore($event.key, $event.id)"
        />
      </div>
      <EmptyState
        v-else
        :title="'pages.projects.emptyState.title'"
        :description="'pages.projects.emptyState.description'"
        :action-label="'pages.projects.emptyState.cta'"
        @action="projectsStore.openImport('local')"
      />
      <div v-if="projectListBusy && projectCards.length" class="list-loading-mask">
        <a-spin />
      </div>
    </div>
  </div>

  <DetailDrawer />
  <FormModal />
  <OutputModal />
  <BindModal />
</template>
