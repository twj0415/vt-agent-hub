<script setup lang="ts">
import EmptyState from './EmptyState.vue'

defineProps<{
  items: Array<{ id: string | number }>
  emptyText: string
}>()
</script>

<template>
  <div class="max-h-[420px] min-h-[260px] overflow-auto rounded-[16px] border border-line/60 bg-panel-strong/92 shadow-surface">
    <slot />
    <div v-if="!items.length" class="flex min-h-[260px] items-center justify-center px-4">
      <EmptyState size="sm" :description="emptyText" :fill="false" />
    </div>
  </div>
</template>
