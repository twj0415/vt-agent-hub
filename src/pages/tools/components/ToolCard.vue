<script setup lang="ts">
  import { computed } from 'vue';
  import { EyeOutlined } from '@ant-design/icons-vue';
  import { useI18n } from 'vue-i18n';
  import CardIconButton from '@/shared/components/feedback/CardIconButton.vue';
  import CardMoreMenu from '@/shared/components/feedback/CardMoreMenu.vue';
  import { toolRegistry, type ToolId } from '@/shared/tool-registry';
  import type { CardMoreMenuItem } from '@/shared/types/ui';

  const props = defineProps<{
    item: {
      id: ToolId;
      name: string;
      desc: string;
      isActive: boolean;
      enabled: boolean;
      path: string;
      version: string;
      ruleTags: string[];
      rulePreviewTitle: string;
      statusLabel: string;
    };
    busy: boolean;
    moreItems: CardMoreMenuItem[];
  }>();

  const emit = defineEmits<{
    open: [id: ToolId];
    more: [payload: { key: string; id: ToolId }];
  }>();

  const { t } = useI18n();

  const tool = computed(() => toolRegistry.find((item) => item.id === props.item.id));
  const ruleCount = computed(() => props.item.ruleTags.length);
</script>

<template>
  <article
    class="tool-card group relative overflow-hidden rounded-[16px] border border-line/60 bg-panel-strong/92 p-4 shadow-surface transition-colors duration-normal ease-standard"
    :class="item.enabled ? 'cursor-pointer hover:border-line-strong/60 hover:bg-panel-strong' : 'cursor-not-allowed opacity-60 grayscale-[0.35]'"
    @click="item.enabled && emit('open', item.id)"
  >
    <span
      class="pointer-events-none absolute bottom-3 left-0 top-3 w-[3px] rounded-r-full opacity-75"
      :class="item.enabled ? 'bg-accent' : 'bg-muted/70'"
      aria-hidden="true"
    />
    <div class="flex items-start gap-4">
      <div class="flex h-14 w-14 shrink-0 items-center justify-center rounded-[14px] border border-line/45 bg-text/[0.04]">
        <img
          v-if="tool?.iconSrc"
          :src="tool.iconSrc"
          :alt="item.name"
          class="h-10 w-10 object-contain"
        />
        <span v-else class="text-[14px] font-bold tracking-[0.04em] text-text/85">
          {{ tool?.iconText ?? '?' }}
        </span>
      </div>

      <div class="min-w-0 flex-1">
        <div class="flex min-w-0 items-start justify-between gap-3">
          <div class="min-w-0">
            <h3
              class="truncate text-[15px] font-semibold tracking-[-0.005em] text-text transition-colors group-hover:text-accent"
            >
              {{ item.name }}
            </h3>
            <p v-if="item.desc" class="mt-1 line-clamp-2 text-[12px] leading-5 text-muted/85">
              {{ item.desc }}
            </p>
          </div>
        </div>

        <p
          v-if="item.path"
          class="mt-2 truncate font-mono text-[11px] leading-4 text-muted/62"
          :title="item.path"
        >
          {{ item.path }}
        </p>

        <div class="mt-3 flex min-w-0 flex-wrap items-center gap-x-3 gap-y-1.5 text-[11px] leading-none">
          <span v-if="item.version" class="inline-flex items-baseline gap-1">
            <span class="font-medium text-muted/65">{{ t('ui.common.version') }}</span>
            <span class="font-semibold text-text/72">{{ item.version }}</span>
          </span>
          <span v-if="ruleCount" class="inline-flex items-baseline gap-1" :title="item.rulePreviewTitle">
            <span class="font-medium text-muted/65">{{ t('ui.common.rules') }}</span>
            <span class="font-semibold text-text/82">{{ ruleCount }}</span>
          </span>
        </div>
      </div>

      <div class="flex shrink-0 items-center gap-1" @click.stop>
        <CardIconButton :title="t('common.detail')" :disabled="busy || !item.enabled" @click="emit('open', item.id)">
          <EyeOutlined />
        </CardIconButton>
        <CardMoreMenu :items="moreItems" :disabled="busy || !item.enabled" @select="emit('more', { key: $event, id: item.id })" />
      </div>
    </div>
  </article>
</template>
