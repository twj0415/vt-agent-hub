<script setup lang="ts">
import StatusBadge from '@/shared/components/feedback/StatusBadge.vue'
import type { MatrixRow } from '@/shared/types/ui'

defineProps<{
  rows: MatrixRow[]
}>()
</script>

<template>
  <div class="overflow-hidden rounded-[16px] border border-line/60 bg-panel-strong/92 shadow-surface">
    <div
      v-for="row in rows"
      :key="row.label"
      class="grid grid-cols-1 gap-2 border-b border-line/70 px-4 py-3 last:border-b-0 md:grid-cols-[148px_minmax(0,1fr)] md:gap-3"
    >
      <span class="text-[11px] font-semibold uppercase tracking-[0.18em] text-muted/85">{{ row.label }}</span>
      <div class="flex flex-wrap items-center gap-2">
        <span class="text-sm leading-6 text-text">{{ row.value }}</span>
        <StatusBadge
          v-if="row.tone"
          :tone="row.tone"
          :label="row.badgeLabel"
          :label-key="row.badgeKey"
        />
      </div>
    </div>
  </div>
</template>
