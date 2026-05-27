<script setup lang="ts">
  withDefaults(
    defineProps<{
      content?: string;
      emptyText?: string;
      fill?: boolean;
      size?: 'xs' | 'sm';
    }>(),
    {
      content: '',
      emptyText: '',
      fill: true,
      size: 'sm',
    }
  );
</script>

<template>
  <div class="min-h-0" :class="fill ? 'flex flex-1 flex-col' : ''">
    <pre v-if="content" class="overflow-auto whitespace-pre-wrap break-words bg-panel p-3 leading-6 text-text" :class="[fill ? 'min-h-0 flex-1' : 'max-h-[520px]', size === 'xs' ? 'text-xs' : 'text-sm']">{{ content }}</pre>
    <div v-else class="grid place-items-center bg-bg/60 px-4 py-6 text-sm text-muted" :class="fill ? 'min-h-0 flex-1' : 'min-h-[180px]'">
      {{ emptyText }}
    </div>
  </div>
</template>
