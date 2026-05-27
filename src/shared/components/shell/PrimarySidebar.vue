<script setup lang="ts">
defineProps<{
  sticky?: boolean
  compact?: boolean
}>()
</script>

<template>
  <aside
    class="rounded-[20px] border border-line/60 bg-panel-strong/88 shadow-surface backdrop-blur-sm"
    :class="[sticky ? 'xl:sticky xl:top-8' : '', compact ? 'p-3' : 'p-4']"
  >
    <slot />
  </aside>
</template>
