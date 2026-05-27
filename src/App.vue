<template>
  <a-config-provider :theme="antTheme">
    <div v-if="appStore.error" class="flex h-full w-full flex-col items-center justify-center gap-4 p-10 text-center">
      <div class="text-[16px] font-semibold text-danger">{{ $t('errors.appBootstrapFailed') }}</div>
      <div class="max-w-md text-[13px] leading-6 text-muted">{{ appStore.error }}</div>
      <a-button type="primary" :loading="appStore.loading" @click="appStore.bootstrapAll()">{{ $t('ui.common.retry') }}</a-button>
    </div>
    <template v-else>
      <RouterView />
      <CommandPalette />
    </template>
  </a-config-provider>
</template>

<script setup lang="ts">
import { useAntTheme } from '@/app/theme/ant-theme'
import { useAppStore } from '@/shared/stores/app'
import CommandPalette from '@/shared/components/shell/CommandPalette.vue'

const antTheme = useAntTheme()
const appStore = useAppStore()
</script>
