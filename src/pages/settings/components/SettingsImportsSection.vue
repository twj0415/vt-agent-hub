<script setup lang="ts">
import { useI18n } from 'vue-i18n'

const { t } = useI18n()
</script>

<template>
  <div class="grid gap-4">
    <section class="rounded-[16px] border border-line/60 bg-panel/60 p-4">
      <div class="text-sm font-semibold text-text">{{ t('ui.common.importAndBackup') }}</div>
      <div class="mt-2 text-sm leading-6 text-muted">{{ t('ui.common.importAndBackup') }}</div>
    </section>
  </div>
</template>
