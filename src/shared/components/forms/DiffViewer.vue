<script setup lang="ts">
import { useI18n } from 'vue-i18n'

defineProps<{
  before: string
  after: string
}>()

const { t } = useI18n()
</script>

<template>
  <div class="grid h-full min-h-0 gap-4 lg:grid-cols-2">
    <section class="flex min-h-0 flex-col rounded-[16px] border border-line/60 bg-panel/80 p-4 shadow-surface">
      <div class="mb-3 text-[11px] font-semibold uppercase tracking-[0.18em] text-muted/85">{{ t('common.before') }}</div>
      <pre class="min-h-0 flex-1 overflow-auto whitespace-pre-wrap rounded-[16px] bg-panel p-3 text-sm leading-6 text-text">{{ before }}</pre>
    </section>
    <section class="flex min-h-0 flex-col rounded-[16px] border border-line/60 bg-panel/80 p-4 shadow-surface">
      <div class="mb-3 text-[11px] font-semibold uppercase tracking-[0.18em] text-muted/85">{{ t('common.after') }}</div>
      <pre class="min-h-0 flex-1 overflow-auto whitespace-pre-wrap rounded-[16px] bg-panel p-3 text-sm leading-6 text-text">{{ after }}</pre>
    </section>
  </div>
</template>
