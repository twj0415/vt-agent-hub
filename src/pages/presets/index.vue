<script setup lang="ts">
  import { PlusOutlined } from '@ant-design/icons-vue';
  import EmptyState from '@/shared/components/feedback/EmptyState.vue';
  import PageHeader from '@/shared/components/shell/PageHeader.vue';
  import ProviderApplyModal from './components/ProviderApplyModal.vue';
  import ProviderCard from './components/ProviderCard.vue';
  import ProviderFormModal from './components/ProviderFormModal.vue';
  import { useProviderActions } from './composables/useProviderActions';
  import { useProvidersWorkbench } from './composables/useProvidersWorkbench';

  const { providerCards, providerListBusy, providerStore, t } = useProvidersWorkbench();
  const { handleMore, moreItems } = useProviderActions();
</script>

<template>
  <div class="workbench-page">
    <PageHeader :title="t('pages.providers.title')" :count="providerCards.length">
      <a-button type="primary" @click="providerStore.openCreate()">
        <template #icon><PlusOutlined /></template>
        {{ t('pages.providers.create') }}
      </a-button>
    </PageHeader>

    <div
      v-if="providerCards.length"
      class="overflow-hidden rounded-[16px] border border-line/60 bg-panel-strong/92 shadow-surface"
    >
      <div class="divide-y divide-line/40">
        <ProviderCard
          v-for="item in providerCards"
          :key="item.id"
          :item="item"
          :busy="providerListBusy"
          :more-items="moreItems"
          @apply="providerStore.openApplyPreview"
          @edit="providerStore.openEdit"
          @more="handleMore($event.key, $event.id)"
        />
      </div>
    </div>

    <EmptyState
      v-else
      :title="'pages.providers.emptyState.title'"
      :description="'pages.providers.emptyState.description'"
      :action-label="'pages.providers.emptyState.cta'"
      @action="providerStore.openCreate()"
    />
  </div>

  <ProviderFormModal />
  <ProviderApplyModal />
</template>
