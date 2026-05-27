<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { BadgeTone } from '@/shared/types/ui'

const props = withDefaults(defineProps<{
  tone?: BadgeTone
  label?: string
  labelKey?: string
}>(), {
  tone: 'neutral',
})

const { t } = useI18n()

const text = computed(() => {
  if (props.labelKey) return t(props.labelKey)
  return props.label ?? ''
})

const toneClass = computed(() => {
  switch (props.tone) {
    case 'ready':
      return 'bg-success/10 text-success border-success/25'
    case 'warning':
      return 'bg-warning/10 text-warning border-warning/25'
    case 'error':
      return 'bg-danger/10 text-danger border-danger/25'
    case 'info':
      return 'bg-text/[0.06] text-text/80 border-text/10'
    case 'active':
      return 'bg-accent/10 text-accent border-accent/22'
    case 'planned':
    case 'neutral':
    default:
      return 'bg-text/[0.06] text-muted border-text/10'
  }
})
</script>

<template>
  <span
    class="inline-flex h-[20px] items-center rounded-[6px] border px-1.5 text-[11px] font-medium leading-none align-middle"
    :class="toneClass"
  >
    {{ text }}
  </span>
</template>
