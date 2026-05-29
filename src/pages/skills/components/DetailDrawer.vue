<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import VTDrawer from '@/shared/components/feedback/VTDrawer.vue'
import EntityTagList from '@/shared/components/feedback/EntityTagList.vue'
import StatusBadge from '@/shared/components/feedback/StatusBadge.vue'
import { useSkillStore } from '@/shared/stores/skills'
import { useSkillCards } from '../composables/useSkillCards'

const { t } = useI18n()
const skillStore = useSkillStore()
const { categoryLabel, statusLabel, statusTone, toolNames } = useSkillCards()

const activeToolTags = computed(() => {
  const active = skillStore.activeItem
  if (!active) return ['-']
  const names = toolNames(active)
  return names.length ? names : ['-']
})
</script>

<template>
  <VTDrawer
    :open="skillStore.detailOpen"
    title-key="pages.skills.drawerTitle"
    :width="560"
    @update:open="skillStore.setDetailOpen"
  >
    <div v-if="skillStore.activeItem" class="space-y-4">
      <div class="rounded-[16px] border border-line/60 bg-panel-strong/92 p-4 shadow-surface">
        <div class="flex flex-wrap items-center gap-3">
          <div class="min-w-0 flex-1 truncate text-lg font-semibold text-text">{{ skillStore.activeItem.name }}</div>
          <StatusBadge :tone="statusTone(skillStore.activeItem)" :label="statusLabel(skillStore.activeItem)" />
        </div>
        <div class="mt-3 grid gap-2 border-t border-line/70 pt-3 text-sm leading-6 text-text">
          <div class="grid grid-cols-[36px_minmax(0,1fr)] gap-3">
            <span class="text-muted">{{ t('catalog.detail.category') }}:</span>
            <span>{{ categoryLabel(skillStore.activeItem.categoryCode) }}</span>
          </div>
          <div class="grid grid-cols-[36px_minmax(0,1fr)] gap-3">
            <span class="text-muted">{{ t('common.version') }}:</span>
            <span>v{{ skillStore.activeItem.versionNo }}</span>
          </div>
          <div class="grid grid-cols-[36px_minmax(0,1fr)] gap-3">
            <span class="text-muted">{{ t('nav.tools') }}:</span>
            <EntityTagList :items="activeToolTags" />
          </div>
        </div>
      </div>

      <div class="rounded-[16px] border border-line/60 bg-panel-strong/92 p-4 text-sm leading-6 text-text shadow-surface">
        <div class="font-semibold text-text">{{ t('common.summary') }}</div>
        <div class="mt-2 text-muted">{{ skillStore.activeItem.summary || t('common.empty') }}</div>
      </div>

      <pre class="max-h-[48vh] overflow-auto whitespace-pre-wrap rounded-[16px] border border-line/45 bg-panel/80 p-4 text-sm leading-6 text-text">{{ skillStore.activeItem.body }}</pre>
    </div>
  </VTDrawer>
</template>
