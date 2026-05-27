<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '@/shared/stores/settings'

const { t } = useI18n()
const settingsStore = useSettingsStore()
</script>

<template>
  <div class="grid gap-4 xl:grid-cols-2">
    <section class="rounded-[16px] border border-line/60 bg-panel/60 p-4">
      <div class="text-sm font-semibold text-text">{{ t('ui.common.currentTool') }}</div>
      <div class="mt-2 text-sm leading-6 text-muted">Codex</div>
    </section>
    <section class="rounded-[16px] border border-line/60 bg-panel/60 p-4">
      <div class="text-sm font-semibold text-text">{{ t('ui.common.credentialState') }}</div>
      <div class="mt-2 text-sm leading-6 text-muted">{{ settingsStore.credentialStateLabel }}</div>
    </section>
  </div>
</template>
