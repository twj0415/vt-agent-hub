<script setup lang="ts">
  import { PlusOutlined } from '@ant-design/icons-vue';
  import EmptyState from '@/shared/components/feedback/EmptyState.vue';
  import PageHeader from '@/shared/components/shell/PageHeader.vue';
  import ProviderApplyModal from './components/ProviderApplyModal.vue';
  import ProviderCard from './components/ProviderCard.vue';
  import ProviderFormModal from './components/ProviderFormModal.vue';
  import { useProviderActions } from './composables/useProviderActions';
  import { useProvidersWorkbench } from './composables/useProvidersWorkbench';

  const { providerCards, providerFilterOptions, providerListBusy, providerStore, setProviderFilter, t } = useProvidersWorkbench();
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

    <div class="mb-3 inline-flex w-fit max-w-full self-start rounded-[14px] border border-line/60 bg-panel-strong/88 p-1 shadow-surface">
      <button
        v-for="option in providerFilterOptions"
        :key="option.value"
        type="button"
        class="flex h-9 items-center gap-2 rounded-[10px] px-3 text-[12px] font-semibold transition-colors duration-fast ease-standard"
        :class="providerStore.filterToolId === option.value ? 'bg-accent/10 text-accent shadow-[inset_0_0_0_1px_rgb(var(--vt-color-accent)/0.18)]' : 'text-muted hover:bg-text/[0.045] hover:text-text'"
        @click="setProviderFilter(option.value)"
      >
        <img v-if="option.iconSrc" :src="option.iconSrc" :alt="option.label" class="h-5 w-5 object-contain" />
        <span v-else class="flex h-5 w-5 items-center justify-center rounded bg-text/10 text-[9px] font-bold uppercase tracking-[0.04em]">
          {{ option.iconText }}
        </span>
        <span>{{ option.label }}</span>
      </button>
    </div>

    <div
      v-if="providerStore.activeDrift"
      class="mb-3 rounded-[16px] border border-amber-500/30 bg-amber-500/10 p-3 text-sm text-text shadow-surface"
    >
      <div class="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
        <div class="min-w-0">
          <div class="font-semibold text-amber-700 dark:text-amber-200">
            {{ t('pages.providers.drift.title') }}
          </div>
          <p class="mt-1 text-xs leading-5 text-muted">
            {{ t('pages.providers.drift.description', { provider: providerStore.activeDrift.providerName }) }}
          </p>
        </div>
        <div class="flex shrink-0 flex-wrap items-center gap-2">
          <a-button size="small" @click="providerStore.showActiveDriftDiff()">
            {{ t('pages.providers.drift.viewDiff') }}
          </a-button>
          <a-button size="small" type="primary" :loading="providerStore.driftChecking" @click="providerStore.applyActiveDriftProvider()">
            {{ t('pages.providers.drift.reapply') }}
          </a-button>
          <a-button size="small" @click="providerStore.openImport(providerStore.activeDrift.toolId)">
            {{ t('pages.providers.drift.importNew') }}
          </a-button>
          <a-button size="small" type="text" @click="providerStore.ignoreActiveDrift()">
            {{ t('pages.providers.drift.ignore') }}
          </a-button>
        </div>
      </div>
    </div>

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
          @apply="providerStore.openApplyPreview($event.providerId, $event.configId)"
          @edit="providerStore.openEdit($event.providerId, $event.configId)"
          @more="handleMore($event.key, $event.providerId)"
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
