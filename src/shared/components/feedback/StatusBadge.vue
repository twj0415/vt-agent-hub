<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { BadgeTone } from '@/shared/types/ui'

const props = withDefaults(defineProps<{
  tone?: BadgeTone
  label?: string
  labelKey?: string
  variant?: 'dot' | 'tag'
}>(), {
  tone: 'neutral',
  variant: 'dot',
})

const { t } = useI18n()

const text = computed(() => {
  if (props.labelKey) return t(props.labelKey)
  return props.label ?? ''
})

const dotColorClass = computed(() => {
  switch (props.tone) {
    case 'ready':
      return 'vt-status-success'
    case 'warning':
      return 'vt-status-warning'
    case 'error':
      return 'vt-status-danger'
    case 'info':
    case 'active':
      return 'vt-status-info'
    case 'planned':
    case 'neutral':
    default:
      return 'vt-status-muted'
  }
})

const tagToneClass = computed(() => {
  switch (props.tone) {
    case 'ready':
      return 'vt-tag-success'
    case 'warning':
      return 'vt-tag-warning'
    case 'error':
      return 'vt-tag-danger'
    case 'info':
    case 'active':
      return 'vt-tag-accent'
    case 'planned':
    case 'neutral':
    default:
      return ''
  }
})
</script>

<template>
  <span
    v-if="variant === 'tag'"
    class="vt-tag align-middle"
    :class="tagToneClass"
  >
    {{ text }}
  </span>
  <span
    v-else
    class="inline-flex items-center gap-1.5 align-middle text-[11px] font-medium leading-none text-muted"
  >
    <span class="vt-status-dot" :class="dotColorClass" />
    {{ text }}
  </span>
</template>
