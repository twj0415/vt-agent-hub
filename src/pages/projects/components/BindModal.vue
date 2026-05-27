<script setup lang="ts">
import { computed } from 'vue'
import { InfoCircleOutlined } from '@ant-design/icons-vue'
import SelectableList from '@/shared/components/feedback/SelectableList.vue'
import VTModal from '@/shared/components/feedback/VTModal.vue'
import { useProjectUi } from '../composables/useProjectUi'

const { availableRules, categoryLabel, projectsStore, ruleDescription, selectedProject, t, workspaceStore } = useProjectUi()
const loading = computed(() => projectsStore.bindLoading)
const activeToolId = computed(() => workspaceStore.activeToolId as 101 | 102 | 103)
</script>

<template>
  <VTModal
    :open="projectsStore.bindOpen"
    :width="760"
    :title="t('pages.projects.bindTitle')"
    :ok-text="t('pages.projects.binding.saveAndApply')"
    :cancel-text="t('ui.common.cancel')"
    :body-style="{ maxHeight: '72vh', overflow: 'hidden', padding: '18px 20px' }"
    :loading="loading"
    :ok-button-props="{ disabled: !projectsStore.selectedRuleCount }"
    @ok="projectsStore.applyRuleBinding(activeToolId)"
    @close="projectsStore.setBindOpen(false)"
  >
    <div class="flex min-h-0 flex-col gap-4">
      <a-alert type="info" show-icon :message="t('pages.projects.binding.projectRulesHint', { name: selectedProject?.name ?? '-' })">
        <template #icon><InfoCircleOutlined /></template>
      </a-alert>

      <SelectableList :items="availableRules" :empty-text="t('ui.common.emptyData')">
        <button
          v-for="rule in availableRules"
          :key="rule.id"
          type="button"
          class="flex w-full items-start gap-3 border-b border-line/80 px-4 py-3 text-left transition last:border-b-0 hover:bg-accent/5"
          :disabled="loading"
          @click="projectsStore.toggleRuleSelection(rule.id)"
        >
          <a-checkbox :checked="projectsStore.bindingDraft.selectedRuleIds.includes(rule.id)" :disabled="loading" class="mt-1" @click.stop @change="projectsStore.toggleRuleSelection(rule.id)" />
          <span class="flex min-w-0 flex-1 items-start justify-between gap-3">
            <span class="min-w-0">
              <span class="flex min-w-0 flex-wrap items-baseline gap-2">
                <span class="min-w-0 truncate text-sm font-semibold text-text">{{ rule.name }}</span>
                <span class="shrink-0 text-[11px] font-normal leading-4 text-muted/60">{{ t('ui.common.bindRulesVersion', { version: rule.versionNo }) }}</span>
              </span>
              <span class="mt-1 line-clamp-2 text-xs leading-5 text-muted">{{ ruleDescription(rule) }}</span>
            </span>
            <span class="flex shrink-0 items-center gap-1.5">
              <span class="shrink-0 rounded-[4px] border border-accent/55 bg-accent/20 px-1.5 py-[1px] text-[10px] font-medium leading-5 text-accent shadow-sm">
                {{ categoryLabel(rule.categoryCode) }}
              </span>
            </span>
          </span>
        </button>
      </SelectableList>
    </div>
  </VTModal>
</template>
