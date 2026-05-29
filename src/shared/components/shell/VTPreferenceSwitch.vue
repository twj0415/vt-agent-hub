<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { BgColorsOutlined, GlobalOutlined } from '@ant-design/icons-vue'
import { useSettingsStore } from '@/shared/stores/settings'
import { themePresets, type ThemePreset } from '@/shared/stores/theme'
import type { LocaleCode } from '@/shared/stores/i18n'

const props = withDefaults(defineProps<{
  compact?: boolean
}>(), {
  compact: false,
})

const settingsStore = useSettingsStore()
const { t } = useI18n()

const localeOptions: LocaleCode[] = ['zh-CN', 'en-US']
const themeOptions = themePresets.map((item) => item.value)
const currentLocaleLabel = computed(() => settingsStore.localeCode === 'en-US' ? 'language.enUS' : 'language.zhCN')
const currentThemeLabel = computed(() => themePresets.find((item) => item.value === settingsStore.themePreset)?.labelKey ?? 'theme.preset.apple')

function toggleLocale() {
  const index = localeOptions.indexOf(settingsStore.localeCode)
  settingsStore.setLocaleCode(localeOptions[(index + 1) % localeOptions.length])
}

function toggleTheme() {
  const index = themeOptions.indexOf(settingsStore.themePreset)
  settingsStore.setThemePreset(themeOptions[(index + 1) % themeOptions.length] as ThemePreset)
}
</script>

<template>
  <div class="preference-switch" :class="{ compact: props.compact }">
    <button type="button" class="preference-button" :aria-label="t('settings.language.label')" @click="toggleLocale">
      <GlobalOutlined />
      <span>{{ t(currentLocaleLabel) }}</span>
    </button>

    <button type="button" class="preference-button" :aria-label="t('settings.theme.label')" @click="toggleTheme">
      <BgColorsOutlined />
      <span>{{ t(currentThemeLabel) }}</span>
    </button>
  </div>
</template>

<style scoped>
.preference-switch {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  border: 1px solid rgb(var(--vt-color-line) / 0.54);
  border-radius: 999px;
  background: rgb(var(--vt-color-bg) / 0.5);
  padding: 4px;
}

.preference-button {
  display: inline-flex;
  height: 30px;
  min-width: 0;
  align-items: center;
  gap: 7px;
  border: 0;
  border-radius: 999px;
  background: transparent;
  color: rgb(var(--vt-color-muted));
  cursor: pointer;
  font-size: 12px;
  font-weight: 700;
  padding: 0 10px;
}

.preference-button span {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.preference-button:hover {
  background: rgb(var(--vt-color-text) / 0.06);
  color: rgb(var(--vt-color-text));
}

.preference-switch.compact {
  width: 154px;
  gap: 3px;
  padding: 3px;
}

.preference-switch.compact .preference-button {
  height: 26px;
  flex: 1;
  justify-content: center;
  gap: 5px;
  font-size: 11px;
  padding: 0 6px;
}
</style>
