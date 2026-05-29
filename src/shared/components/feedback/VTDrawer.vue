<script setup lang="ts">
  import { computed } from 'vue';
  import { useI18n } from 'vue-i18n';

  const props = defineProps<{
    title?: string;
    titleKey?: string;
    width?: number;
  }>();

  // macOS 风抽屉：圆角左边、header 精致分隔、body 慷慨留白、footer backdrop。
  const open = defineModel<boolean>('open', { required: true });
  const { t } = useI18n();

  const drawerTitle = computed(() => {
    if (props.titleKey) return t(props.titleKey);
    return props.title ?? '';
  });
</script>

<template>
  <a-drawer
    class="app-drawer"
    v-model:open="open"
    :title="null"
    :closable="false"
    :width="width ?? 520"
    :body-style="{
      background: 'rgb(var(--vt-color-panel))',
      display: 'flex',
      flexDirection: 'column',
      minHeight: '0',
      padding: '0',
    }"
  >
    <header
      class="flex h-11 shrink-0 items-center justify-between border-b border-line/55 px-5"
    >
      <h2 class="truncate text-[14px] font-semibold tracking-[-0.005em] text-text">
        {{ drawerTitle }}
      </h2>
      <button
        type="button"
        class="flex h-6 w-6 items-center justify-center rounded-vt-sm text-[11px] text-muted transition-colors hover:bg-text/[0.06] hover:text-text"
        :aria-label="t('common.close')"
        @click="open = false"
      >
        <svg viewBox="0 0 10 10" class="h-3 w-3" fill="none">
          <path d="M2 2l6 6M8 2L2 8" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
        </svg>
      </button>
    </header>

    <div class="flex min-h-0 flex-1 flex-col overflow-y-auto px-5 py-4">
      <slot />
    </div>

    <div
      v-if="$slots.footer"
      class="shrink-0 border-t border-line/55 bg-panel-strong/85 px-5 py-3 backdrop-blur-xl"
    >
      <div class="flex flex-wrap justify-end gap-2">
        <slot name="footer" />
      </div>
    </div>
  </a-drawer>
</template>
