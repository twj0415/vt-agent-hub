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
      <div class="text-[13px] font-semibold text-text">{{ t('ui.common.theme') }}</div>
      <div class="mt-3 max-h-[360px] overflow-y-auto rounded-vt-md border border-line bg-surface-2/40 p-1.5">
        <button
          v-for="item in themeOptions"
          :key="item.value"
          type="button"
          class="theme-option group flex w-full items-center gap-3 rounded-vt-md px-3 py-2.5 text-left transition-all duration-fast ease-standard hover:bg-text/[0.05]"
          :class="settingsStore.themePreset === item.value ? 'theme-option-active' : ''"
          @click="settingsStore.setThemePreset(item.value)"
        >
          <span class="theme-swatch" :data-theme-preview="item.value" aria-hidden="true">
            <span />
            <span />
            <span />
          </span>
          <span class="min-w-0 flex-1">
            <span class="block truncate text-[13px] font-semibold text-text">{{ item.title }}</span>
            <span class="mt-0.5 block truncate text-[11px] leading-5 text-muted/80">{{ item.description }}</span>
          </span>
          <CheckOutlined
            class="shrink-0 text-[13px] text-accent transition-opacity"
            :class="settingsStore.themePreset === item.value ? 'opacity-100' : 'opacity-0 group-hover:opacity-30'"
          />
        </button>
      </div>
    </section>

    <section>
      <div class="text-[13px] font-semibold text-text">{{ t('ui.common.language') }}</div>
      <div class="mt-3 grid gap-1.5 rounded-vt-md border border-line bg-surface-2/40 p-1.5">
        <button
          v-for="item in localeOptions"
          :key="item.value"
          type="button"
          class="theme-option flex items-center justify-between rounded-vt-md px-3 py-2.5 text-left transition-all duration-fast ease-standard hover:bg-text/[0.05]"
          :class="settingsStore.localeCode === item.value ? 'theme-option-active' : ''"
          @click="settingsStore.setLocaleCode(item.value)"
        >
          <span class="block text-[13px] font-semibold text-text">{{ item.title }}</span>
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
.theme-option-active {
  background: rgb(var(--vt-color-accent) / 0.10);
  box-shadow: inset 0 0 0 1px rgb(var(--vt-color-accent) / 0.18);
}

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

.theme-swatch[data-theme-preview='apple'] span:first-child { background: #FAFAFA; }
.theme-swatch[data-theme-preview='apple'] span:nth-child(2) { background: #FFFFFF; }
.theme-swatch[data-theme-preview='apple'] span:nth-child(3) { background: #2563EB; }

.theme-swatch[data-theme-preview='warm'] span:first-child { background: #F8F5EF; }
.theme-swatch[data-theme-preview='warm'] span:nth-child(2) { background: #FFFDF8; }
.theme-swatch[data-theme-preview='warm'] span:nth-child(3) { background: #B45309; }

.theme-swatch[data-theme-preview='graphite'] span:first-child { background: #18181B; }
.theme-swatch[data-theme-preview='graphite'] span:nth-child(2) { background: #1F1F23; }
.theme-swatch[data-theme-preview='graphite'] span:nth-child(3) { background: #10B981; }

.theme-swatch[data-theme-preview='dark'] span:first-child { background: #0A0A0F; }
.theme-swatch[data-theme-preview='dark'] span:nth-child(2) { background: #12131A; }
.theme-swatch[data-theme-preview='dark'] span:nth-child(3) { background: #3B82F6; }
</style>
