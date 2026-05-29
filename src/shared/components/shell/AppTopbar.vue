<script setup lang="ts">
  import { useI18n } from 'vue-i18n';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { MinusOutlined, BorderOutlined, CloseOutlined } from '@ant-design/icons-vue';
  import { isTauriRuntime } from '@/shared/utils/runtime';
  import TopbarSettingsPopover from './TopbarSettingsPopover.vue';

  const { t } = useI18n();
  const appWindow = isTauriRuntime() ? getCurrentWindow() : null;

  const appIcon = new URL('../../../assets/icon.ico', import.meta.url).href;

  async function minimizeWindow() {
    await appWindow?.minimize();
  }

  async function toggleMaximizeWindow() {
    await appWindow?.toggleMaximize();
  }

  async function closeWindow() {
    await appWindow?.close();
  }
</script>

<template>
  <header class="topbar-shell flex h-8 shrink-0 items-center border-b border-line/60 bg-bg/92 backdrop-blur-xl backdrop-saturate-150">
    <div data-tauri-drag-region class="flex h-full min-w-0 flex-1 items-center self-stretch">
      <div data-tauri-drag-region class="ml-3 flex min-w-0 items-center gap-2">
        <img
          data-tauri-drag-region
          :src="appIcon"
          :alt="t('app.name')"
          class="h-4 w-4 shrink-0 rounded-[5px] object-contain"
        />
        <span data-tauri-drag-region class="vt-brand-gradient truncate text-[12px] font-semibold tracking-[-0.01em]">
          {{ t('app.name') }}
        </span>
      </div>
    </div>

    <div class="flex shrink-0 items-center gap-0.5 pr-1">
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
.topbar-shell {
  position: relative;
}
.topbar-shell::after {
  content: '';
  position: absolute;
  left: 0;
  right: 0;
  bottom: -1px;
  height: 1px;
  background: linear-gradient(
    90deg,
    transparent 0%,
    rgb(var(--vt-color-accent) / 0.18) 35%,
    rgb(var(--vt-color-accent) / 0.12) 65%,
    transparent 100%
  );
  pointer-events: none;
}

.win-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  height: 32px;
  width: 38px;
  padding: 0;
  border: none;
  background: transparent;
  color: rgb(var(--vt-color-muted));
  font-size: 10px;
  outline: none;
  transition: background-color 0.12s ease, color 0.12s ease;
}

.win-btn:hover {
  background: rgb(var(--vt-color-text) / 0.06);
  color: rgb(var(--vt-color-text));
}

.win-btn-close:hover {
  background: rgb(var(--vt-color-danger) / 0.18);
  color: rgb(var(--vt-color-danger));
}
</style>
