<script setup lang="ts">
import { computed } from 'vue'
import { CheckOutlined } from '@ant-design/icons-vue'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '@/shared/stores/settings'
import { themePresets } from '@/shared/stores/theme'
import type { LocaleCode } from '@/shared/stores/i18n'

const { t } = useI18n()
const settingsStore = useSettingsStore()

const localeOptions: Array<{ value: LocaleCode; title: string }> = [
  { value: 'zh-CN', title: t('language.zhCN') },
  { value: 'en-US', title: t('language.enUS') },
]

const themeOptions = computed(() =>
  themePresets.map((item) => ({
    ...item,
    title: t(item.labelKey),
    description: t(`theme.desc.${item.value}`),
  })),
)
</script>

<template>
  <div class="grid gap-5 xl:grid-cols-[minmax(0,1.15fr)_minmax(280px,0.85fr)]">
    <section>
      <div class="text-sm font-semibold text-text">{{ t('ui.common.theme') }}</div>
      <div class="mt-3 max-h-[360px] overflow-y-auto rounded-[16px] border border-line/60 bg-panel/60 p-2">
        <button
          v-for="item in themeOptions"
          :key="item.value"
          type="button"
          class="group flex w-full items-center gap-3 rounded-[12px] px-3 py-2.5 text-left transition-colors duration-normal ease-standard hover:bg-text/[0.055]"
          :class="settingsStore.themePreset === item.value ? 'bg-accent/[0.09]' : ''"
          @click="settingsStore.setThemePreset(item.value)"
        >
          <span class="theme-swatch" :data-theme-preview="item.value" aria-hidden="true">
            <span />
            <span />
            <span />
          </span>
          <span class="min-w-0 flex-1">
            <span class="block truncate text-sm font-semibold text-text">{{ item.title }}</span>
            <span class="mt-0.5 block truncate text-xs leading-5 text-muted/80">{{ item.description }}</span>
          </span>
          <CheckOutlined
            class="shrink-0 text-[13px] text-accent transition-opacity"
            :class="settingsStore.themePreset === item.value ? 'opacity-100' : 'opacity-0 group-hover:opacity-30'"
          />
        </button>
      </div>
    </section>

    <section>
      <div class="text-sm font-semibold text-text">{{ t('ui.common.language') }}</div>
      <div class="mt-3 grid gap-2 rounded-[16px] border border-line/60 bg-panel/60 p-2">
        <button
          v-for="item in localeOptions"
          :key="item.value"
          type="button"
          class="flex items-center justify-between rounded-[12px] px-3 py-2.5 text-left transition-colors duration-normal ease-standard hover:bg-text/[0.055]"
          :class="settingsStore.localeCode === item.value ? 'bg-accent/[0.09]' : ''"
          @click="settingsStore.setLocaleCode(item.value)"
        >
          <span class="block text-sm font-semibold text-text">{{ item.title }}</span>
          <CheckOutlined
            class="shrink-0 text-[13px] text-accent transition-opacity"
            :class="settingsStore.localeCode === item.value ? 'opacity-100' : 'opacity-0'"
          />
        </button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.theme-swatch {
  display: grid;
  width: 42px;
  height: 30px;
  flex-shrink: 0;
  grid-template-columns: 1fr 1fr;
  overflow: hidden;
  border: 1px solid rgb(var(--vt-color-line) / 0.55);
  border-radius: 10px;
  background: rgb(var(--vt-color-panel-strong));
  box-shadow: 0 1px 2px rgb(0 0 0 / 0.04);
}

.theme-swatch span:first-child {
  grid-row: span 2;
}

.theme-swatch[data-theme-preview='apple'] span:first-child { background: #f5f5f7; }
.theme-swatch[data-theme-preview='apple'] span:nth-child(2) { background: #ffffff; }
.theme-swatch[data-theme-preview='apple'] span:nth-child(3) { background: #0066cc; }

.theme-swatch[data-theme-preview='warm'] span:first-child { background: #f4efe7; }
.theme-swatch[data-theme-preview='warm'] span:nth-child(2) { background: #fffdf8; }
.theme-swatch[data-theme-preview='warm'] span:nth-child(3) { background: #c8612d; }

.theme-swatch[data-theme-preview='clean'] span:first-child { background: #f5f5f7; }
.theme-swatch[data-theme-preview='clean'] span:nth-child(2) { background: #fbfbfd; }
.theme-swatch[data-theme-preview='clean'] span:nth-child(3) { background: #0071e3; }

.theme-swatch[data-theme-preview='dark'] span:first-child { background: #0c0d12; }
.theme-swatch[data-theme-preview='dark'] span:nth-child(2) { background: #181922; }
.theme-swatch[data-theme-preview='dark'] span:nth-child(3) { background: #768eff; }
</style>
