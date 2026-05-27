<script setup lang="ts">
  import { computed } from 'vue';
  import { useI18n } from 'vue-i18n';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { DownOutlined, MinusOutlined, BorderOutlined, CloseOutlined, CheckOutlined } from '@ant-design/icons-vue';
  import { toolRegistry, type ToolId } from '@/shared/tool-registry';
  import { useToolContextStore } from '@/shared/stores/tool-context';
  import { useToolsStore } from '@/shared/stores/tools';
  import TopbarSettingsPopover from './TopbarSettingsPopover.vue';

  const { t } = useI18n();
  const workspaceStore = useToolContextStore();
  const toolsStore = useToolsStore();
  const appWindow = getCurrentWindow();

  const appIcon = new URL('../../../assets/icon.ico', import.meta.url).href;
  const activeTool = computed(() => toolRegistry.find((item) => item.id === workspaceStore.activeToolId) ?? toolRegistry[0]);
  const selectableTools = computed(() => toolRegistry.filter((item) => item.enabled));

  function selectTool(id: ToolId) {
    const tool = toolRegistry.find((item) => item.id === id);
    if (!tool?.enabled) return;
    workspaceStore.setActiveTool(id);
    toolsStore.select(id);
  }

  function onToolMenuClick({ key }: { key: string | number }) {
    selectTool(Number(key) as ToolId);
  }

  async function minimizeWindow() {
    await appWindow.minimize();
  }

  async function toggleMaximizeWindow() {
    await appWindow.toggleMaximize();
  }

  async function closeWindow() {
    await appWindow.close();
  }
</script>

<template>
  <header class="flex h-9 shrink-0 items-center border-b border-line/35 bg-panel/72 backdrop-blur-xl backdrop-saturate-150">
    <div data-tauri-drag-region class="flex h-full min-w-0 flex-1 items-center self-stretch">
      <div data-tauri-drag-region class="ml-3 flex min-w-0 items-center gap-2">
        <img
          data-tauri-drag-region
          :src="appIcon"
          :alt="t('app.name')"
          class="h-[18px] w-[18px] shrink-0 rounded-[5px] object-contain shadow-[0_1px_2px_rgb(0_0_0/0.12)]"
        />
        <span data-tauri-drag-region class="truncate text-[12px] font-semibold tracking-[-0.01em] text-text/75">
          {{ t('app.name') }}
        </span>
      </div>
    </div>

    <div class="flex shrink-0 items-center gap-0.5 pr-1">
      <a-dropdown placement="bottomRight" trigger="click">
        <button
          type="button"
          class="inline-flex h-7 items-center gap-1.5 rounded-full border-0 bg-transparent px-2.5 text-[13px] font-medium text-text outline-none transition-[background-color,box-shadow] hover:bg-text/[0.055]"
        >
          <img
            v-if="activeTool?.iconSrc"
            :src="activeTool.iconSrc"
            :alt="t(activeTool?.nameKey ?? 'common.tool')"
            class="h-4 w-4 object-contain"
          />
          <span v-else class="text-[10px] font-semibold uppercase tracking-[0.06em]">
            {{ activeTool?.iconText }}
          </span>
          <span class="max-w-[120px] truncate">{{ t(activeTool?.nameKey ?? 'common.tool') }}</span>
          <DownOutlined class="text-[8px] text-muted" />
        </button>
        <template #overlay>
          <a-menu class="min-w-[180px]" @click="onToolMenuClick">
            <a-menu-item v-for="tool in selectableTools" :key="tool.id">
              <span class="flex items-center gap-2">
                <CheckOutlined v-if="tool.id === activeTool?.id" class="text-accent" />
                <span v-else class="w-4" />
                <span>{{ t(tool.nameKey) }}</span>
              </span>
            </a-menu-item>
          </a-menu>
        </template>
      </a-dropdown>

      <TopbarSettingsPopover />
    </div>

    <div class="flex h-full shrink-0 items-center">
      <button
        type="button"
        class="win-btn"
        :aria-label="t('topbar.window.minimize')"
        @mousedown.stop
        @click.stop="minimizeWindow"
      >
        <MinusOutlined />
      </button>
      <button
        type="button"
        class="win-btn"
        :aria-label="t('topbar.window.maximize')"
        @mousedown.stop
        @click.stop="toggleMaximizeWindow"
      >
        <BorderOutlined />
      </button>
      <button
        type="button"
        class="win-btn win-btn-close"
        :aria-label="t('topbar.window.close')"
        @mousedown.stop
        @click.stop="closeWindow"
      >
        <CloseOutlined />
      </button>
    </div>
  </header>
</template>

<style scoped>
.win-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  height: 36px;
  width: 44px;
  padding: 0;
  border: none;
  background: transparent;
  color: rgb(var(--vt-color-muted));
  font-size: 11px;
  outline: none;
  transition: background-color 0.12s ease, box-shadow 0.12s ease, color 0.12s ease;
}

.win-btn:hover {
  background: rgb(var(--vt-color-text) / 0.06);
  color: rgb(var(--vt-color-text));
}

.win-btn-close:hover {
  background: rgb(var(--vt-color-danger) / 0.15);
  color: rgb(var(--vt-color-danger));
}
</style>
