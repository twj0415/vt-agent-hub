<script setup lang="ts">
defineProps<{
  loading?: boolean
}>()
</script>

<template>
  <div class="relative flex h-full min-h-0 flex-1 flex-col">
    <div class="flex h-full min-h-0 flex-col gap-3 overflow-hidden">
      <slot />
    </div>
    <div v-if="loading" class="absolute inset-0 z-10 grid place-items-center rounded-[16px] bg-panel-strong/55 backdrop-blur-[1px]">
      <a-spin />
    </div>
  </div>
</template>
