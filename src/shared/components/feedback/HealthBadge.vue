<script setup lang="ts">
import { computed } from 'vue'
import StatusBadge from './StatusBadge.vue'

const props = defineProps<{
  level: 'healthy' | 'attention' | 'risk'
}>()

const labelKey = computed(() => `health.${props.level}`)
const tone = computed(() => {
  if (props.level === 'healthy') return 'ready'
  if (props.level === 'attention') return 'warning'
  return 'error'
})
</script>

<template>
  <StatusBadge :label-key="labelKey" :tone="tone" />
</template>
