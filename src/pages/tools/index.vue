<script setup lang="ts">
  import { computed } from 'vue';
  import { ToolOutlined } from '@ant-design/icons-vue';
  import EmptyState from '@/shared/components/feedback/EmptyState.vue';
  import PageHeader from '@/shared/components/shell/PageHeader.vue';
  import type { CardMoreMenuItem } from '@/shared/types/ui';
  import type { ToolId } from '@/shared/tool-registry';
  import ToolSkillBindModal from '../skills/components/ToolSkillBindModal.vue';
  import BindModal from './components/BindModal.vue';
  import DetailDrawer from './components/DetailDrawer.vue';
  import ToolCard from './components/ToolCard.vue';
  import { useToolsWorkbench } from './composables/useToolsWorkbench';

  const { t, manageRules, openTool, toolCards, toolsStore } = useToolsWorkbench();

  const toolListBusy = computed(() => toolsStore.bindLoading);

  function handleToolMore(key: string, id: ToolId) {
    if (key === 'rules') manageRules(id);
  }

  function toolMoreItems(item: { canManageRules: boolean }): CardMoreMenuItem[] {
    const busy = toolListBusy.value;
    return [{ key: 'rules', label: t('common.bind'), icon: ToolOutlined, disabled: busy || !item.canManageRules }];
  }
</script>

<template>
  <div class="workbench-page">
    <PageHeader :title="t('pages.tools.title')" :count="toolCards.length" />

    <div class="mb-3 rounded-md border border-warning/30 bg-warning/10 px-3 py-2 text-xs leading-5 text-muted">
      {{ t('pages.tools.v1SupportNotice') }}
    </div>

    <div class="workbench-body">
      <div v-if="toolCards.length" class="entity-grid">
        <ToolCard v-for="item in toolCards" :key="item.id" :item="item" :busy="toolListBusy" :more-items="toolMoreItems(item)" @open="openTool" @more="handleToolMore($event.key, $event.id)" />
      </div>
      <EmptyState
        v-else
        :title="'pages.tools.emptyState.title'"
        :description="'pages.tools.emptyState.description'"
      />
      <div v-if="toolListBusy && toolCards.length" class="list-loading-mask">
        <a-spin />
      </div>
    </div>
  </div>

  <DetailDrawer />
  <BindModal />
  <ToolSkillBindModal />
</template>
