<script setup lang="ts">
  defineProps<{
    label: string;
    title?: string;
  }>();
</script>

<template>
  <div class="grid min-w-0 grid-cols-[auto_minmax(0,1fr)] items-start gap-x-2" :title="title">
    <span class="min-w-[4.5rem] whitespace-nowrap text-xs font-semibold leading-6 text-muted transition group-hover:text-text/70 group-focus-within:text-text/70">{{ label }}:</span>
    <div class="min-w-0 leading-6 text-text">
      <slot />
    </div>
  </div>
</template>
