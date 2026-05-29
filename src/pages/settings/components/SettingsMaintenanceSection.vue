<script setup lang="ts">
import { ReloadOutlined } from '@ant-design/icons-vue'
import { useI18n } from 'vue-i18n'
import BindingMatrix from '@/shared/components/forms/BindingMatrix.vue'
import { useConfirm } from '@/shared/composables/useConfirm'
import { useSettingsStore } from '@/shared/stores/settings'

const { t } = useI18n()
const settingsStore = useSettingsStore()
const { confirmAction } = useConfirm()

// 重置应用数据是高危操作，统一走 useConfirm 弹窗。
function confirmResetAppData() {
  confirmAction({
    danger: true,
    okText: t('ui.common.reset'),
    content: t('ui.common.confirmResetAppData'),
    onOk: () => settingsStore.resetApplicationData(),
  })
}
</script>

<template>
  <div class="grid gap-4">
    <BindingMatrix :rows="settingsStore.truthSourceRows" />
    <section class="rounded-vt-md border border-danger/30 bg-danger/8 p-4">
      <div class="flex flex-col gap-4 md:flex-row md:items-center md:justify-between">
        <div class="min-w-0">
          <div class="text-[13px] font-semibold text-text">{{ t('ui.common.resetAppData') }}</div>
          <div class="mt-1 max-w-3xl text-[13px] leading-6 text-muted">
            {{ t('ui.common.confirmResetAppData') }}
          </div>
        </div>
        <a-button danger @click="confirmResetAppData">
          <template #icon><ReloadOutlined /></template>
          {{ t('ui.common.reset') }}
        </a-button>
      </div>
    </section>
  </div>
</template>
