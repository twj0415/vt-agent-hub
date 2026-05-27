<script setup lang="ts">
withDefaults(
  defineProps<{
    variant?: 'detail' | 'preview'
    padded?: boolean
    fill?: boolean
    bodyClass?: string
  }>(),
  {
    variant: 'detail',
    padded: true,
    fill: false,
    bodyClass: '',
  },
)
</script>

<template>
  <section
    class="overflow-hidden rounded-[16px]"
    :class="[variant === 'preview' ? 'border border-line/50 bg-panel/90' : 'border border-line/60 bg-panel-strong/92', padded ? 'p-4' : '', (!padded || fill) ? 'flex min-h-0 flex-col' : '', fill ? 'flex-1' : '']"
  >
    <header v-if="$slots.header" class="shrink-0 border-b border-line/40 px-4 py-3">
      <slot name="header" />
    </header>
    <div :class="[(!padded || fill) ? 'flex min-h-0 flex-1 flex-col' : '', bodyClass]">
      <slot />
    </div>
  </section>
</template>
