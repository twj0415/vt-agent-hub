<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '@/shared/stores/settings'
import type { SettingsPath } from '@/shared/api/client'

const { t } = useI18n()
const settingsStore = useSettingsStore()

const storagePathKeys = ['storage_root', 'app_db', 'library_root', 'backups', 'logs', 'snapshots', 'runtime'] as const

const storagePathCards = computed<SettingsPath[]>(() =>
  storagePathKeys
    .map((key) => settingsStore.paths.find((item) => item.key === key))
    .filter((item): item is SettingsPath => Boolean(item)),
)

// 把 key 翻成显示名；没匹配到的回退原 key，避免硬编码 fallback。
function pathLabel(key: string) {
  const labels: Record<string, string> = {
    storage_root: t('settings.paths.storageRoot'),
    app_db: t('settings.paths.appDb'),
    library_root: t('settings.paths.libraryRoot'),
    project_output: t('settings.paths.projectOutput'),
    backups: t('settings.paths.backups'),
    logs: t('settings.paths.logs'),
    snapshots: t('settings.paths.snapshots'),
    runtime: t('settings.paths.runtime'),
  }
  return labels[key] ?? key
}
</script>

<template>
  <div class="overflow-hidden rounded-[16px] border border-line/60 bg-panel/60">
    <article
      v-for="item in storagePathCards"
      :key="item.key"
      class="grid gap-2 border-b border-line/70 px-4 py-4 last:border-b-0 xl:grid-cols-[180px_minmax(0,1fr)]"
    >
      <div>
        <div class="text-sm font-semibold text-text">{{ pathLabel(item.key) }}</div>
        <div class="mt-1 text-xs text-muted">{{ item.key }}</div>
      </div>
      <div class="min-w-0">
        <div class="break-all rounded-[12px] border border-line/55 bg-panel-strong/70 px-3 py-2 text-sm text-text">{{ item.path }}</div>
        <div class="mt-2 text-sm leading-6 text-muted">{{ item.note }}</div>
      </div>
    </article>
  </div>
</template>
