<script setup lang="ts">
import { ClearOutlined, SafetyCertificateOutlined, SaveOutlined, ToolOutlined } from '@ant-design/icons-vue'
import { useI18n } from 'vue-i18n'
import { useToolsStore } from '@/shared/stores/tools'

const { t } = useI18n()
const toolsStore = useToolsStore()
</script>

<template>
  <div class="grid gap-4 xl:grid-cols-2">
    <section class="rounded-vt-md border border-line bg-surface-2/40 p-4">
      <div class="text-[13px] font-semibold text-text">{{ t('ui.common.credential') }}</div>
      <a-input-password
        :value="toolsStore.draft.token"
        :placeholder="t('ui.common.credential')"
        :disabled="!toolsStore.activeToolEnabled"
        class="!mt-3"
        @update:value="toolsStore.setDraftField('token', String($event))"
      />
      <div class="mt-3 flex flex-wrap gap-2">
        <a-button :disabled="!toolsStore.activeToolEnabled" @click="toolsStore.saveCredential()">
          <template #icon><SaveOutlined /></template>
          {{ t('ui.common.save') }}
        </a-button>
        <a-button :disabled="!toolsStore.activeToolEnabled" @click="toolsStore.verifyCredential()">
          <template #icon><SafetyCertificateOutlined /></template>
          {{ t('ui.common.verify') }}
        </a-button>
        <a-button :disabled="!toolsStore.activeToolEnabled" @click="toolsStore.clearCredential()">
          <template #icon><ClearOutlined /></template>
          {{ t('ui.common.clear') }}
        </a-button>
      </div>
    </section>

    <section class="rounded-vt-md border border-line bg-surface-2/40 p-4">
      <div class="text-[13px] font-semibold text-text">{{ t('ui.common.repairTool') }}</div>
      <div class="mt-1 text-[13px] leading-6 text-muted">{{ toolsStore.diagnostics.repairHint }}</div>
      <div class="mt-3 grid gap-2 text-[13px] text-muted">
        <div class="break-all">Live Config: {{ toolsStore.diagnostics.liveConfigPath || '-' }}</div>
        <div>{{ t('ui.common.status') }}: {{ toolsStore.diagnostics.repairState || '-' }}</div>
      </div>
      <a-button class="!mt-3" :disabled="!toolsStore.activeToolEnabled" @click="toolsStore.repair()">
        <template #icon><ToolOutlined /></template>
        {{ t('ui.common.repairTool') }}
      </a-button>
    </section>
  </div>
</template>
