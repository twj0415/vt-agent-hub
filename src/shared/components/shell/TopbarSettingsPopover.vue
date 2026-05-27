<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { CheckOutlined, ReloadOutlined, RightOutlined, SettingOutlined } from '@ant-design/icons-vue'
import { appRoutes } from '@/shared/config/routes'
import { useAppStore } from '@/shared/stores/app'
import { useSettingsStore } from '@/shared/stores/settings'
import type { LocaleCode } from '@/shared/stores/i18n'
import { themePresets, type ThemePreset } from '@/shared/stores/theme'

const router = useRouter()
const { t } = useI18n()
const appStore = useAppStore()
const settingsStore = useSettingsStore()
const settingsOpen = ref(false)

const localeOptions: Array<{ value: LocaleCode; labelKey: string }> = [
  { value: 'zh-CN', labelKey: 'language.zhCN' },
  { value: 'en-US', labelKey: 'language.enUS' },
]

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

function setLocale(value: LocaleCode) {
  settingsStore.setLocaleCode(value)
  closeMenu()
}

function setTheme(value: ThemePreset) {
  settingsStore.setThemePreset(value)
  closeMenu()
}
</script>

<template>
  <a-popover v-model:open="settingsOpen" trigger="hover" placement="bottomRight" overlay-class-name="topbar-settings-popover">
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
        <div class="topbar-panel-actions">
          <button type="button" class="topbar-row is-primary" @click="openSettings">
            <span>{{ t('topbar.settingsCenter') }}</span>
            <RightOutlined class="topbar-row-icon" />
          </button>
          <button
            type="button"
            class="topbar-row"
            :disabled="appStore.loading"
            @click="refreshApp"
          >
            <span>{{ t('common.refresh') }}</span>
            <ReloadOutlined class="topbar-row-icon" :class="appStore.loading ? 'animate-spin' : ''" />
          </button>
        </div>

        <section class="topbar-panel-section">
          <div class="topbar-section-title">{{ t('settings.language.label') }}</div>
          <div class="topbar-option-list compact">
            <button
              v-for="item in localeOptions"
              :key="item.value"
              type="button"
              class="topbar-option-row"
              :class="settingsStore.localeCode === item.value ? 'is-active' : ''"
              @click="setLocale(item.value)"
            >
              <span class="topbar-option-main">{{ t(item.labelKey) }}</span>
              <CheckOutlined class="topbar-option-check" />
            </button>
          </div>
        </section>

        <section class="topbar-panel-section">
          <div class="topbar-section-title">{{ t('settings.theme.label') }}</div>
          <div class="topbar-option-list theme-list">
            <button
              v-for="item in themePresets"
              :key="item.value"
              type="button"
              class="topbar-option-row"
              :class="settingsStore.themePreset === item.value ? 'is-active' : ''"
              @click="setTheme(item.value)"
            >
              <span class="theme-swatch-mini" :data-theme-preview="item.value" aria-hidden="true">
                <span />
                <span />
                <span />
              </span>
              <span class="topbar-option-main">{{ t(item.labelKey) }}</span>
              <CheckOutlined class="topbar-option-check" />
            </button>
          </div>
        </section>
      </div>
    </template>
  </a-popover>
</template>

<style scoped>
:global(.topbar-settings-popover .ant-popover-inner) {
  overflow: hidden;
  border: 1px solid rgb(var(--vt-color-line) / 0.52);
  border-radius: 16px;
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
  width: 244px;
  padding: 8px;
}

.topbar-panel-actions,
.topbar-panel-section {
  border-radius: 12px;
  background: rgb(var(--vt-color-text) / 0.025);
}

.topbar-panel-actions {
  padding: 4px;
}

.topbar-panel-section {
  margin-top: 8px;
  padding: 7px;
}

.topbar-section-title {
  padding: 0 5px 6px;
  color: rgb(var(--vt-color-muted) / 0.78);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.08em;
  line-height: 1;
  text-transform: uppercase;
}

.topbar-row,
.topbar-option-row {
  display: flex;
  width: 100%;
  align-items: center;
  border: 0;
  outline: none;
  background: transparent;
  color: rgb(var(--vt-color-text));
  text-align: left;
  transition: background-color var(--vt-duration-fast) var(--vt-ease-standard),
    color var(--vt-duration-fast) var(--vt-ease-standard);
}

.topbar-row {
  height: 30px;
  justify-content: space-between;
  gap: 10px;
  border-radius: 9px;
  padding: 0 9px;
  font-size: 13px;
  font-weight: 500;
}

.topbar-row.is-primary {
  font-weight: 650;
}

.topbar-row:hover:not(:disabled),
.topbar-option-row:hover,
.topbar-option-row.is-active {
  background: rgb(var(--vt-color-text) / 0.055);
}

.topbar-row:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.topbar-row-icon {
  flex-shrink: 0;
  color: rgb(var(--vt-color-muted) / 0.82);
  font-size: 10px;
}

.topbar-option-list {
  display: grid;
  gap: 2px;
}

.topbar-option-list.theme-list {
  max-height: 184px;
  overflow-y: auto;
  padding-right: 1px;
}

.topbar-option-row {
  height: 30px;
  gap: 8px;
  border-radius: 9px;
  padding: 0 8px;
  font-size: 13px;
  font-weight: 500;
}

.topbar-option-main {
  min-width: 0;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.topbar-option-check {
  flex-shrink: 0;
  color: rgb(var(--vt-color-accent));
  font-size: 11px;
  opacity: 0;
}

.topbar-option-row.is-active .topbar-option-check {
  opacity: 1;
}

.theme-swatch-mini {
  display: grid;
  width: 24px;
  height: 16px;
  flex-shrink: 0;
  grid-template-columns: 1fr 1fr;
  overflow: hidden;
  border: 1px solid rgb(var(--vt-color-line) / 0.55);
  border-radius: 6px;
  background: rgb(var(--vt-color-panel-strong));
}

.theme-swatch-mini span:first-child {
  grid-row: span 2;
}

.theme-swatch-mini[data-theme-preview='apple'] span:first-child { background: #f5f5f7; }
.theme-swatch-mini[data-theme-preview='apple'] span:nth-child(2) { background: #ffffff; }
.theme-swatch-mini[data-theme-preview='apple'] span:nth-child(3) { background: #0066cc; }

.theme-swatch-mini[data-theme-preview='warm'] span:first-child { background: #f4efe7; }
.theme-swatch-mini[data-theme-preview='warm'] span:nth-child(2) { background: #fffdf8; }
.theme-swatch-mini[data-theme-preview='warm'] span:nth-child(3) { background: #c8612d; }

.theme-swatch-mini[data-theme-preview='clean'] span:first-child { background: #f5f5f7; }
.theme-swatch-mini[data-theme-preview='clean'] span:nth-child(2) { background: #fbfbfd; }
.theme-swatch-mini[data-theme-preview='clean'] span:nth-child(3) { background: #0071e3; }

.theme-swatch-mini[data-theme-preview='dark'] span:first-child { background: #0c0d12; }
.theme-swatch-mini[data-theme-preview='dark'] span:nth-child(2) { background: #181922; }
.theme-swatch-mini[data-theme-preview='dark'] span:nth-child(3) { background: #768eff; }
</style>
