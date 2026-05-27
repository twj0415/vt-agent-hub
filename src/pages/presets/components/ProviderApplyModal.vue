<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import VTModal from '@/shared/components/feedback/VTModal.vue'
import DiffViewer from '@/shared/components/forms/DiffViewer.vue'
import { useProvidersStore } from '@/shared/stores/providers'

const { t } = useI18n()
const providerStore = useProvidersStore()
const previewFiles = computed(() => {
  const files = providerStore.applyPreview?.files ?? []
  if (files.length) return files
  const preview = providerStore.applyPreview
  if (!preview) return []
  return [{
    label: 'config.toml',
    beforeContent: preview.beforeContent,
    afterContent: preview.afterContent,
  }]
})
</script>

<template>
  <VTModal
    :open="providerStore.applyOpen"
    :title="t('pages.providers.applyTitle')"
    :ok-text="t('common.apply')"
    :cancel-text="t('common.close')"
    :width="920"
    @ok="providerStore.applyToLiveConfig(true)"
    @close="providerStore.setApplyOpen(false)"
  >
    <div class="space-y-3">
      <p v-if="providerStore.applyPreview?.warning" class="text-sm text-muted">
        {{ providerStore.applyPreview.warning }}
      </p>
      <section
        v-for="file in previewFiles"
        :key="file.label"
        class="grid gap-3"
      >
        <div class="text-xs font-semibold uppercase tracking-[0.18em] text-muted">
          {{ file.label }}
        </div>
        <DiffViewer
          :before="file.beforeContent"
          :after="file.afterContent"
        />
      </section>
    </div>
  </VTModal>
</template>
