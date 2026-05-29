<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useSettingsStore } from '@/shared/stores/settings'

const { t } = useI18n()
const settingsStore = useSettingsStore()
</script>

<template>
  <div class="grid gap-4 xl:grid-cols-2">
    <section class="rounded-vt-md border border-line bg-surface-2/40 p-4">
      <div class="text-[13px] font-semibold text-text">{{ t('ui.common.currentTool') }}</div>
      <div class="mt-2 text-[13px] leading-6 text-muted">Codex</div>
    </section>
    <section class="rounded-vt-md border border-line bg-surface-2/40 p-4">
      <div class="text-[13px] font-semibold text-text">{{ t('ui.common.credentialState') }}</div>
      <div class="mt-2 text-[13px] leading-6 text-muted">{{ settingsStore.credentialStateLabel }}</div>
    </section>
  </div>
</template>
