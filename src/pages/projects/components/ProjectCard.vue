<script setup lang="ts">
  import { computed } from 'vue';
  import { EditOutlined, EyeOutlined } from '@ant-design/icons-vue';
  import { useI18n } from 'vue-i18n';
  import CardIconButton from '@/shared/components/feedback/CardIconButton.vue';
  import CardMoreMenu from '@/shared/components/feedback/CardMoreMenu.vue';
  import { projectTypeCodes } from '@/shared/taxonomy';
  import type { CardMoreMenuItem } from '@/shared/types/ui';
  import type { ProjectStatusTone } from '../utils/status';

  const props = defineProps<{
    item: {
      id: number;
      name: string;
      path: string;
      isActive: boolean;
      projectType: number;
      projectTypeLabel: string;
      ruleTags: string[];
      rulePreviewTitle: string;
      statusLabel: string;
      statusTone: ProjectStatusTone;
    };
    busy: boolean;
    moreItems: CardMoreMenuItem[];
  }>();

  const emit = defineEmits<{
    open: [id: number];
    edit: [id: number];
    more: [payload: { key: string; id: number }];
  }>();

  const { t } = useI18n();

  const typeColorMap: Record<number, string> = {
    [projectTypeCodes.web]: '#6fb08c',
    [projectTypeCodes.mini]: '#c79243',
    [projectTypeCodes.desktop]: '#a58ac2',
  };

  const typeColor = computed(() => typeColorMap[props.item.projectType] ?? '#8e8e93');
  const ruleCount = computed(() => props.item.ruleTags.length);
</script>

<template>
  <article
    class="project-card group relative cursor-pointer overflow-hidden rounded-[16px] border border-line/60 bg-panel-strong/92 shadow-surface transition-colors duration-normal ease-standard hover:border-line-strong/60 hover:bg-panel-strong"
    @click="emit('open', item.id)"
  >
    <span
      class="pointer-events-none absolute bottom-3 left-0 top-3 w-[3px] rounded-r-full"
      :style="{ background: typeColor }"
      aria-hidden="true"
    />

    <div class="flex flex-col gap-4 p-5 pl-6">
      <div class="flex items-start justify-between gap-3">
        <div class="min-w-0 flex-1">
          <div class="flex min-w-0 items-center gap-2">
            <h3
              class="truncate text-[15px] font-semibold leading-snug tracking-[-0.005em] text-text transition-colors group-hover:text-accent"
            >
              {{ item.name }}
            </h3>
            <span class="shrink-0 text-[11px] font-medium leading-4 text-muted/72">
              {{ item.projectTypeLabel }}
            </span>
          </div>
          <p class="mt-1.5 truncate font-mono text-[11px] leading-4 text-muted/78" :title="item.path">
            {{ item.path }}
          </p>
        </div>
        <span class="shrink-0 text-[11px] font-medium leading-4 text-muted/72">
          {{ item.statusLabel }}
        </span>
      </div>

      <div class="flex items-center justify-between gap-3">
        <div class="flex min-w-0 flex-wrap items-center gap-x-3 gap-y-1.5 text-[11px] leading-none text-muted/75">
          <span v-if="ruleCount" class="inline-flex items-baseline gap-1" :title="item.rulePreviewTitle">
            <span class="font-medium text-muted/65">{{ t('ui.common.rules') }}</span>
            <span class="font-semibold text-text/82">{{ ruleCount }}</span>
          </span>
        </div>

        <div class="flex shrink-0 items-center gap-1" @click.stop>
          <CardIconButton :title="t('common.detail')" :disabled="busy" @click="emit('open', item.id)">
            <EyeOutlined />
          </CardIconButton>
          <CardIconButton :title="t('catalog.action.edit')" :disabled="busy" @click="emit('edit', item.id)">
            <EditOutlined />
          </CardIconButton>
          <CardMoreMenu :items="moreItems" @select="emit('more', { key: $event, id: item.id })" />
        </div>
      </div>
    </div>
  </article>
</template>
