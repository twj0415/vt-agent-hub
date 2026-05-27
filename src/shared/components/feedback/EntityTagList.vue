<script setup lang="ts">
  import StatusBadge from './StatusBadge.vue';

  withDefaults(
    defineProps<{
      items: string[];
      tone?: 'ready' | 'planned' | 'warning' | 'error';
    }>(),
    {
      tone: 'planned',
    }
  );
</script>

<template>
  <div class="flex min-w-0 flex-wrap items-center gap-1.5">
    <template v-if="items.length">
      <StatusBadge v-for="item in items" :key="item" :tone="tone" :label="item" shape="soft" :strong="false" :uppercase="false" />
    </template>
    <span v-else class="text-sm leading-6 text-muted">-</span>
  </div>
</template>
