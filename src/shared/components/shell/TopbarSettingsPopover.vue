<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { ReloadOutlined, RightOutlined, SettingOutlined } from '@ant-design/icons-vue'
import { appRoutes } from '@/shared/config/routes'
import { useAppStore } from '@/shared/stores/app'
import VTPreferenceSwitch from './VTPreferenceSwitch.vue'

const router = useRouter()
const { t } = useI18n()
const appStore = useAppStore()
const settingsOpen = ref(false)

function closeMenu() {
  settingsOpen.value = false
}

function openSettings() {
  closeMenu()
  void router.push(appRoutes.settings)
}

function refreshApp() {
  if (appStore.loading) return
  closeMenu()
  void appStore.bootstrapAll()
}

</script>

<template>
  <a-popover v-model:open="settingsOpen" trigger="click" placement="bottomRight" overlay-class-name="topbar-settings-popover">
    <a-button
      type="text"
      shape="circle"
      class="topbar-settings-button topbar-icon-button !flex !h-8 !w-8 !items-center !justify-center !rounded-full !border-0"
      :class="[
        '!bg-transparent !text-muted hover:!bg-bg hover:!text-text',
        settingsOpen ? 'topbar-settings-button-open' : '',
      ]"
    >
      <SettingOutlined class="topbar-settings-icon" />
    </a-button>

    <template #content>
      <div class="topbar-settings-panel text-text">
        <button type="button" class="topbar-settings-entry" @click="openSettings">
          <span>{{ t('topbar.settingsCenter') }}</span>
          <RightOutlined class="topbar-row-icon" />
        </button>

        <div class="topbar-panel-divider" />

        <div class="topbar-panel-bottom">
          <VTPreferenceSwitch compact />

          <button
            type="button"
            class="topbar-refresh-button"
            :disabled="appStore.loading"
            @click="refreshApp"
          >
            <ReloadOutlined :class="appStore.loading ? 'animate-spin' : ''" />
          </button>
        </div>
      </div>
    </template>
  </a-popover>
</template>

<style scoped>
:global(.topbar-settings-popover .ant-popover-inner) {
  overflow: hidden;
  border: 1px solid rgb(var(--vt-color-line-strong, var(--vt-color-line)) / 0.5);
  border-radius: 18px;
  background: rgb(var(--vt-color-panel-strong) / 0.94);
  box-shadow: var(--vt-shadow-surface-lg);
  backdrop-filter: blur(22px) saturate(180%);
}

:global(.topbar-settings-popover .ant-popover-inner-content) {
  padding: 0;
}

:global(.topbar-settings-popover .ant-popover-arrow) {
  display: none;
}

.topbar-settings-icon {
  transition: transform 180ms ease;
}

.topbar-settings-button:hover .topbar-settings-icon,
.topbar-settings-button-open .topbar-settings-icon {
  transform: rotate(90deg);
}

.topbar-settings-panel {
  width: 206px;
  padding: 7px;
}

.topbar-settings-entry {
  display: flex;
  width: 100%;
  height: 30px;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
  border: 0;
  border-radius: 10px;
  background: rgb(var(--vt-color-text) / 0.045);
  color: rgb(var(--vt-color-text));
  cursor: pointer;
  font-size: 12px;
  font-weight: 700;
  padding: 0 9px;
  text-align: left;
}

.topbar-settings-entry:hover {
  background: rgb(var(--vt-color-text) / 0.07);
}

.topbar-panel-divider {
  height: 1px;
  margin: 6px 2px;
  background: rgb(var(--vt-color-line-strong, var(--vt-color-line)) / 0.5);
}

.topbar-panel-bottom {
  display: flex;
  align-items: center;
  gap: 6px;
}

.topbar-refresh-button {
  display: grid;
  width: 30px;
  height: 30px;
  flex: 0 0 auto;
  place-items: center;
  border: 1px solid rgb(var(--vt-color-line) / 0.5);
  border-radius: 999px;
  background: rgb(var(--vt-color-bg) / 0.5);
  color: rgb(var(--vt-color-muted));
  cursor: pointer;
}

.topbar-refresh-button:hover:not(:disabled) {
  background: rgb(var(--vt-color-text) / 0.06);
  color: rgb(var(--vt-color-text));
}

.topbar-refresh-button:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.topbar-row-icon {
  flex-shrink: 0;
  color: rgb(var(--vt-color-muted) / 0.82);
  font-size: 10px;
}
</style>
