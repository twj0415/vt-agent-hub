<script setup lang="ts">
  import { computed, ref } from 'vue';
  import { EditOutlined, EyeOutlined } from '@ant-design/icons-vue';
  import { useI18n } from 'vue-i18n';
  import CardIconButton from '@/shared/components/feedback/CardIconButton.vue';
  import CardMoreMenu from '@/shared/components/feedback/CardMoreMenu.vue';
  import { skillCategoryCodes } from '@/shared/taxonomy';
  import { useDragBinding } from '@/shared/composables/useDragBinding';
  import { useSkillActions } from '../composables/useSkillActions';

  const props = defineProps<{
    item: {
      id: number;
      name: string;
      categoryCode: number;
      categoryLabel: string;
      versionLabel: string;
      summaryText: string;
      toolTags: string[];
      toolCount: number;
      toolTitle: string;
    };
    busy: boolean;
  }>();

  const { t } = useI18n();
  const { handleMore, moreItems, openSkillDetail, skillStore } = useSkillActions();
  const { begin, end } = useDragBinding();

  const isDragging = ref(false);

  function onDragStart(event: DragEvent) {
    isDragging.value = true;
    begin({ type: 'skill', id: props.item.id, name: props.item.name }, event);
  }

  function onDragEnd() {
    isDragging.value = false;
    end();
  }

  const categoryColorMap: Record<number, string> = {
    [skillCategoryCodes.coding]: '#6f9f9b',
    [skillCategoryCodes.uiDesign]: '#c07a8d',
  };

  const categoryColor = computed(() => categoryColorMap[props.item.categoryCode] ?? '#8e8e93');
</script>

<template>
  <article
    class="vt-card vt-card-hover skill-card group overflow-hidden"
    :class="isDragging ? 'opacity-50 scale-[0.985]' : ''"
    draggable="true"
    @click="openSkillDetail(item.id)"
    @dragstart="onDragStart"
    @dragend="onDragEnd"
  >
    <span
      class="vt-accent-bar opacity-75"
      :style="{ background: categoryColor }"
      aria-hidden="true"
    />
    <div class="grid gap-3 px-4 py-3 pl-5 md:grid-cols-[minmax(0,1fr)_auto] md:items-center">
      <div class="min-w-0">
        <div class="flex min-w-0 items-center gap-2">
          <h3
            class="min-w-0 truncate text-[13px] font-semibold leading-snug tracking-[-0.005em] text-text transition-colors group-hover:text-accent"
          >
            {{ item.name }}
          </h3>
          <span class="vt-tag">
            {{ item.categoryLabel }}
          </span>
        </div>

        <p
          v-if="item.summaryText"
          class="mt-1 line-clamp-2 text-[12px] leading-5 text-muted/85"
          :title="item.summaryText"
        >
          {{ item.summaryText }}
        </p>

        <div class="mt-2.5 flex min-w-0 flex-wrap items-center gap-x-3 gap-y-1.5 text-[11px] leading-none">
          <span class="inline-flex items-baseline gap-1" :title="item.toolTitle">
            <span class="font-medium text-muted/65">{{ t('ui.common.tools') }}</span>
            <span class="font-semibold text-text/82">{{ item.toolCount }}</span>
          </span>
          <span class="inline-flex items-baseline gap-1">
            <span class="font-medium text-muted/65">{{ t('ui.common.version') }}</span>
            <span class="font-semibold text-text/72">{{ item.versionLabel }}</span>
          </span>
        </div>
      </div>

      <div class="flex shrink-0 items-center gap-1 justify-self-end" data-no-drag draggable="false" @click.stop>
        <CardIconButton :title="t('common.detail')" :disabled="busy" @click="openSkillDetail(item.id)">
          <EyeOutlined />
        </CardIconButton>
        <CardIconButton
          :title="t('catalog.action.edit')"
          :disabled="busy"
          @click="skillStore.openEdit(item.id)"
        >
          <EditOutlined />
        </CardIconButton>
        <CardMoreMenu :items="moreItems" @select="handleMore($event, item)" />
      </div>
    </div>
  </article>
</template>
