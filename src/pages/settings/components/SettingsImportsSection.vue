<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { useFirstRunImportStore } from '@/shared/stores/first-run-import'

const { t } = useI18n()
const firstRunImportStore = useFirstRunImportStore()
</script>

<template>
  <div class="grid gap-4">
    <section class="rounded-[16px] border border-line/60 bg-panel/60 p-4">
      <div class="flex items-start justify-between gap-4">
        <div>
          <div class="text-sm font-semibold text-text">{{ t('firstRunImport.settings.title') }}</div>
          <div class="mt-2 text-sm leading-6 text-muted">{{ t('firstRunImport.settings.description') }}</div>
        </div>
        <a-button :loading="firstRunImportStore.loading" @click="firstRunImportStore.openManualScan()">
          {{ t('firstRunImport.actions.rescan') }}
        </a-button>
      </div>
    </section>
  </div>
</template>
