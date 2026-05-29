<script setup lang="ts">
import { computed } from 'vue'
import { InfoCircleOutlined } from '@ant-design/icons-vue'
import { useI18n } from 'vue-i18n'
import SelectableList from '@/shared/components/feedback/SelectableList.vue'
import VTModal from '@/shared/components/feedback/VTModal.vue'
import { useSkillStore } from '@/shared/stores/skills'
import { toolRegistry } from '@/shared/tool-registry'

const { t } = useI18n()
const skillStore = useSkillStore()
const loading = computed(() => skillStore.bindLoading)
const activeSkill = computed(() => skillStore.items.find((item) => item.id === skillStore.bindDraft.skillId) ?? null)
const availableTools = computed(() => toolRegistry.filter((tool) => tool.enabled && tool.capabilities.skillInstall))
</script>

<template>
  <VTModal
    :open="skillStore.bindOpen"
    :width="760"
    :title="t('pages.skills.bindTool')"
    :ok-text="t('catalog.action.save')"
    :cancel-text="t('common.close')"
    :body-style="{ maxHeight: '72vh', overflow: 'hidden', padding: '18px 20px' }"
    :loading="loading"
    @ok="skillStore.saveToolBinding()"
    @close="skillStore.setBindOpen(false)"
  >
    <div class="flex min-h-0 flex-col gap-4">
      <a-alert
        type="info"
        show-icon
        :message="t('pages.skills.bindToolDesc', { name: activeSkill?.name ?? '-' })"
      >
        <template #icon><InfoCircleOutlined /></template>
      </a-alert>

      <SelectableList :items="availableTools" :empty-text="t('common.emptyData')">
        <button
          v-for="tool in availableTools"
          :key="tool.id"
          type="button"
          class="flex w-full items-center justify-between gap-3 border-b border-line/80 px-4 py-3 text-left transition last:border-b-0 hover:bg-accent/5"
          :disabled="loading"
          @click="skillStore.toggleBindTool(tool.id)"
        >
          <span class="flex min-w-0 items-center gap-3">
            <a-checkbox
              :checked="skillStore.bindDraft.selectedToolIds.includes(tool.id)"
              :disabled="loading"
              @click.stop
              @change="skillStore.toggleBindTool(tool.id)"
            />
            <span class="flex h-9 w-9 shrink-0 items-center justify-center overflow-hidden rounded-[10px] border border-line/45 bg-text/[0.04]">
              <img v-if="tool.iconSrc" :src="tool.iconSrc" :alt="t(tool.nameKey)" class="h-7 w-7 object-contain" />
              <span v-else class="text-[11px] font-bold text-text/80">{{ tool.iconText }}</span>
            </span>
            <span class="min-w-0">
              <span class="block truncate text-sm font-semibold text-text">{{ t(tool.nameKey) }}</span>
              <span class="block truncate text-xs leading-5 text-muted">{{ t(tool.descKey) }}</span>
            </span>
          </span>
        </button>
      </SelectableList>
    </div>
  </VTModal>
</template>
