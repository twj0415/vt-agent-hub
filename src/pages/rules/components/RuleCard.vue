<script setup lang="ts">
  import { EditOutlined, EyeOutlined, HolderOutlined } from '@ant-design/icons-vue';
  import { computed } from 'vue';
  import { useI18n } from 'vue-i18n';
  import CardIconButton from '@/shared/components/feedback/CardIconButton.vue';
  import CardMoreMenu from '@/shared/components/feedback/CardMoreMenu.vue';
  import { useRuleActions } from '../composables/useRuleActions';

  const props = defineProps<{
    item: {
      id: number;
      name: string;
      bodyText: string;
      summaryText: string;
      categoryCode: number;
      categoryLabel: string;
      versionLabel: string;
      projectTags: string[];
      projectTitle: string;
      toolTags: string[];
      toolTitle: string;
    };
    busy: boolean;
  }>();

  const { t } = useI18n();
  const { handleMore, moreItems, openRuleDetail, ruleStore } = useRuleActions();

  const categoryColors = ['#7aa2d8', '#82a889', '#c89a6f', '#aa8fca', '#c07f8f', '#8d9aa8'];
  const categoryColor = computed(() => categoryColors[Math.abs(props.item.categoryCode) % categoryColors.length]);
</script>

<template>
  <article
    class="rule-row group relative cursor-pointer px-4 py-4 transition-colors duration-fast ease-standard hover:bg-text/[0.04]"
    data-rule-card
    @click="openRuleDetail(item.id)"
  >
    <span
      class="pointer-events-none absolute bottom-3 left-0 top-3 w-[3px] rounded-r-full opacity-75"
      :style="{ background: categoryColor }"
      aria-hidden="true"
    />
    <div class="grid gap-3 md:grid-cols-[auto_minmax(0,1fr)_auto] md:items-center">
      <button
        data-drag-handle
        data-no-detail
        type="button"
        class="flex h-8 w-8 cursor-grab items-center justify-center rounded-[10px] border border-transparent bg-text/[0.035] text-muted/65 transition-colors duration-fast ease-standard hover:border-line/55 hover:bg-text/[0.055] hover:text-text active:cursor-grabbing"
        title="拖动绑定到项目"
        @click.stop
      >
        <HolderOutlined />
      </button>

      <div class="min-w-0">
        <div class="flex min-w-0 items-center gap-2">
          <h3
            class="min-w-0 truncate text-[14px] font-semibold leading-snug tracking-[-0.005em] text-text transition-colors group-hover:text-accent"
          >
            {{ item.name }}
          </h3>
          <span class="shrink-0 text-[11px] font-medium leading-4 text-muted/72">
            {{ item.categoryLabel }}
          </span>
        </div>

        <p
          v-if="item.summaryText || item.bodyText"
          class="mt-1.5 line-clamp-2 max-w-4xl text-[12px] leading-5 text-muted/85"
        >
          {{ item.summaryText || item.bodyText }}
        </p>

        <div data-rule-meta class="mt-3 flex min-w-0 flex-wrap items-center gap-x-3 gap-y-1.5 text-[11px] leading-none">
          <span class="inline-flex items-baseline gap-1" :title="item.projectTitle">
            <span class="font-medium text-muted/65">{{ t('ui.common.projects') }}</span>
            <span class="font-semibold text-text/82">{{ item.projectTags.length }}</span>
          </span>
          <span class="inline-flex items-baseline gap-1" :title="item.toolTitle">
            <span class="font-medium text-muted/65">{{ t('ui.common.tools') }}</span>
            <span class="font-semibold text-text/82">{{ item.toolTags.length }}</span>
          </span>
          <span class="inline-flex items-baseline gap-1">
            <span class="font-medium text-muted/65">{{ t('ui.common.version') }}</span>
            <span class="font-semibold text-text/72">{{ item.versionLabel }}</span>
          </span>
        </div>
      </div>

      <div data-no-drag class="flex shrink-0 items-center gap-1 justify-self-end" draggable="false" @click.stop>
        <CardIconButton :title="t('common.detail')" :disabled="busy" @click="openRuleDetail(item.id)">
          <EyeOutlined />
        </CardIconButton>
        <CardIconButton :title="t('catalog.action.edit')" :disabled="busy" @click="ruleStore.openEdit(item.id)">
          <EditOutlined />
        </CardIconButton>
        <CardMoreMenu :items="moreItems" @select="handleMore($event, item)" />
      </div>
    </div>
  </article>
</template>
