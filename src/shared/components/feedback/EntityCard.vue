<script setup lang="ts">
  defineProps<{
    title: string;
  }>();

  const emit = defineEmits<{
    open: [];
  }>();
</script>

<template>
  <article
    class="group relative isolate cursor-pointer overflow-hidden rounded-[16px] border border-line/60 bg-panel-strong/92 p-4 shadow-surface transition-colors duration-normal ease-standard hover:border-line-strong/60 hover:bg-panel-strong"
    @click="emit('open')"
  >
    <div class="relative z-10 flex flex-col gap-3">
      <div class="flex h-9 items-center justify-between gap-3">
        <div class="flex min-w-0 items-center gap-3">
          <h2 class="min-w-0 overflow-hidden truncate pr-2 text-[15px] font-semibold leading-snug tracking-[-0.005em] text-text transition group-hover:text-accent group-focus-within:text-accent">{{ title }}</h2>
          <slot name="status" />
        </div>

        <div class="flex min-w-[116px] shrink-0 items-center justify-end gap-1 whitespace-nowrap" @click.stop>
          <slot name="actions" />
        </div>
      </div>

      <div class="grid min-h-[88px] gap-3 border-t border-line/50 pt-2.5 text-sm md:grid-cols-[minmax(260px,0.72fr)_minmax(260px,1fr)]">
        <div class="grid min-w-0 content-start gap-2">
          <slot name="left" />
        </div>

        <div class="grid min-w-0 content-start gap-2">
          <slot name="right" />
        </div>
      </div>
    </div>
  </article>
</template>
