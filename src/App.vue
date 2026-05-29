<template>
  <a-config-provider :theme="antTheme">
    <div v-if="appStore.error" class="flex h-full w-full flex-col items-center justify-center gap-4 p-10 text-center">
      <div class="text-[16px] font-semibold text-danger">{{ $t('errors.appBootstrapFailed') }}</div>
      <div class="max-w-md text-[13px] leading-6 text-muted">{{ appStore.error }}</div>
      <a-button type="primary" :loading="appStore.loading" @click="appStore.bootstrapAll()">{{ $t('ui.common.retry') }}</a-button>
    </div>
    <div v-else-if="!appStore.ready" class="flex h-full w-full items-center justify-center bg-bg text-text">
      <div class="flex flex-col items-center gap-3 rounded-[22px] border border-line/50 bg-panel/80 px-7 py-6 shadow-[0_18px_50px_rgb(0_0_0/0.08)]">
        <a-spin />
        <div class="text-[13px] font-medium text-muted">正在启动 VT Hub…</div>
      </div>
    </div>
    <template v-else>
      <RouterView />
      <CommandPalette />
      <FirstRunImportModal />
    </template>
  </a-config-provider>
</template>

<script setup lang="ts">
import { watch } from 'vue'
import { useAntTheme } from '@/app/theme/ant-theme'
import { useAppStore } from '@/shared/stores/app'
import { useFirstRunImportStore } from '@/shared/stores/first-run-import'
import CommandPalette from '@/shared/components/shell/CommandPalette.vue'
import FirstRunImportModal from '@/features/first-run-import/FirstRunImportModal.vue'

const antTheme = useAntTheme()
const appStore = useAppStore()
const firstRunImportStore = useFirstRunImportStore()

watch(
  () => [appStore.ready, appStore.error] as const,
  ([ready, error]) => {
    if (ready && !error) void firstRunImportStore.maybeOpenAfterBootstrap()
  },
  { immediate: true },
)
</script>
