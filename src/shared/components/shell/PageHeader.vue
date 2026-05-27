<script setup lang="ts">
import { useI18n } from 'vue-i18n'

defineProps<{
  title: string
  count?: number
  description?: string
}>()

const { t } = useI18n()
</script>

<template>
  <header class="flex h-14 shrink-0 items-end gap-4 border-b border-line/35 pb-3">
    <div class="min-w-0 flex-1">
      <h1
        class="truncate text-[22px] font-semibold leading-none tracking-[-0.026em] text-text"
      >
        {{ title }}
      </h1>
      <div
        v-if="count !== undefined || description"
        class="mt-1.5 truncate text-[11px] font-medium uppercase tracking-[0.1em] text-muted/65"
      >
        <span v-if="count !== undefined">{{ count }} {{ t('common.items') }}</span>
        <span v-if="count !== undefined && description" class="mx-2 opacity-40">·</span>
        <span v-if="description">{{ description }}</span>
      </div>
    </div>

    <div class="flex shrink-0 items-center gap-2">
      <slot />
    </div>
  </header>
</template>
