<script setup lang="ts">
import BindingMatrix from '@/shared/components/forms/BindingMatrix.vue'
import { useSettingsStore } from '@/shared/stores/settings'

const settingsStore = useSettingsStore()
</script>

<template>
  <div class="grid gap-4">
    <BindingMatrix :rows="settingsStore.projectBoundaryRows" />
  </div>
</template>
