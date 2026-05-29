<script setup lang="ts">
  import { computed } from 'vue';
  import { EditOutlined } from '@ant-design/icons-vue';
  import { useI18n } from 'vue-i18n';
  import CardIconButton from '@/shared/components/feedback/CardIconButton.vue';
  import CardMoreMenu from '@/shared/components/feedback/CardMoreMenu.vue';
  import type { ToolId } from '@/shared/tool-registry';
  import type { CardMoreMenuItem } from '@/shared/types/ui';

  const props = defineProps<{
    item: {
      id: string;
      providerId: number;
      configId: number | null;
      name: string;
      categoryLabel: string;
      toolId: ToolId | null;
      toolTags: string[];
      toolTitle: string;
      model: string;
      baseUrl: string;
      statusLabel: string;
      active: boolean;
      activeLabel: string;
    };
    busy: boolean;
    moreItems: CardMoreMenuItem[];
  }>();

  const emit = defineEmits<{
    apply: [payload: { providerId: number; configId: number | null }];
    edit: [payload: { providerId: number; configId: number | null }];
    more: [payload: { key: string; providerId: number }];
  }>();

  const { t } = useI18n();

  const initial = computed(() => props.item.name.charAt(0).toUpperCase() || '?');
</script>

<template>
  <article
    class="provider-row group relative grid cursor-pointer gap-3 px-4 py-3 transition-colors duration-fast ease-standard md:grid-cols-[40px_minmax(0,1fr)_auto] md:items-center"
    :class="item.active ? 'bg-accent/[0.06]' : 'vt-row-hover'"
    @click="emit('edit', { providerId: item.providerId, configId: item.configId })"
  >
    <span
      v-if="item.active"
      class="vt-accent-bar"
      aria-hidden="true"
    />

    <div
      class="flex h-10 w-10 shrink-0 items-center justify-center rounded-vt-md text-[14px] font-semibold tracking-tight transition-colors"
      :class="
        item.active
          ? 'bg-accent/12 text-accent ring-1 ring-accent/25'
          : 'bg-surface-2/60 text-text/85 border border-line/40'
      "
    >
      {{ initial }}
    </div>

    <div class="min-w-0">
      <div class="flex min-w-0 flex-wrap items-center gap-2">
        <h3
          class="min-w-0 truncate text-[13px] tracking-[-0.005em] text-text transition-colors group-hover:text-accent"
          :class="item.active ? 'font-bold' : 'font-semibold'"
        >
          {{ item.name }}
        </h3>
        <span class="vt-tag">
          {{ item.categoryLabel }}
        </span>
        <span v-if="item.active" class="inline-flex items-center gap-1.5 text-[11px] font-medium text-success">
          <span class="vt-status-dot vt-status-success" />
          {{ item.activeLabel }}
        </span>
      </div>

      <p
        v-if="item.baseUrl"
        class="mt-1 truncate font-mono text-[11px] leading-4 text-muted/75"
        :title="item.baseUrl"
      >
        {{ item.baseUrl }}
      </p>

      <div class="mt-2 flex min-w-0 flex-wrap items-center gap-x-3 gap-y-1.5 text-[11px] leading-none">
        <span v-if="item.model" class="inline-flex items-baseline gap-1">
          <span class="font-medium text-muted/65">{{ t('ui.common.model') }}</span>
          <span class="font-semibold text-text/82">{{ item.model }}</span>
        </span>
        <span v-if="item.toolTags.length" class="inline-flex items-baseline gap-1" :title="item.toolTitle">
          <span class="font-medium text-muted/65">{{ t('ui.common.tools') }}</span>
          <span class="font-semibold text-text/82">{{ item.toolTags.length }}</span>
        </span>
      </div>
    </div>

    <div class="flex shrink-0 items-center justify-end gap-2" data-no-drag @click.stop>
      <div class="flex items-center gap-1">
        <a-button
          v-if="!item.active"
          type="primary"
          size="small"
          :disabled="busy"
          @click="emit('apply', { providerId: item.providerId, configId: item.configId })"
        >
          {{ t('common.apply') }}
        </a-button>
        <CardIconButton :title="t('catalog.action.edit')" :disabled="busy" @click="emit('edit', { providerId: item.providerId, configId: item.configId })">
          <EditOutlined />
        </CardIconButton>
        <CardMoreMenu :items="moreItems" @select="emit('more', { key: $event, providerId: item.providerId })" />
      </div>
    </div>
  </article>
</template>
