<script setup lang="ts">
import { computed } from 'vue'
import { CheckCircleFilled, InfoCircleOutlined } from '@ant-design/icons-vue'
import SelectableList from '@/shared/components/feedback/SelectableList.vue'
import VTModal from '@/shared/components/feedback/VTModal.vue'
import { toolIds } from '@/shared/tool-registry'
import type { RuleBindingTargetToolId } from '@/shared/stores/projects-model'
import { useProjectUi } from '../composables/useProjectUi'

const { availableRules, categoryLabel, projectsStore, ruleDescription, selectedProject, t } = useProjectUi()
const loading = computed(() => projectsStore.bindLoading)
const targetTools: Array<{ id: RuleBindingTargetToolId; name: string; output: string }> = [
  { id: toolIds.codex, name: 'Codex', output: 'AGENTS.md' },
  { id: toolIds.claude, name: 'Claude', output: 'CLAUDE.md' },
]
</script>

<template>
  <VTModal
    :open="projectsStore.bindOpen"
    :width="760"
    :title="t('pages.projects.bindTitle')"
    :ok-text="t('pages.projects.binding.saveAndApplyToTool', { tool: targetTools.find((item) => item.id === projectsStore.bindingDraft.targetToolId)?.name ?? 'Codex' })"
    :cancel-text="t('ui.common.cancel')"
    :body-style="{ maxHeight: '72vh', overflow: 'hidden', padding: '18px 20px' }"
    :loading="loading"
    :ok-button-props="{ disabled: !projectsStore.selectedRuleCount }"
    @ok="projectsStore.applyRuleBinding()"
    @close="projectsStore.setBindOpen(false)"
  >
    <div class="flex min-h-0 flex-col gap-4">
      <a-alert type="info" show-icon :message="t('pages.projects.binding.projectRulesHint', { name: selectedProject?.name ?? '-' })">
        <template #icon><InfoCircleOutlined /></template>
      </a-alert>

      <div class="rounded-[18px] border border-line/70 bg-bg/70 p-3 shadow-[inset_0_1px_0_rgb(255_255_255/0.68)]">
        <div class="mb-2 flex items-center justify-between gap-3 px-1">
          <span class="text-xs font-semibold uppercase tracking-[0.18em] text-muted/70">{{ t('pages.projects.binding.targetTool') }}</span>
          <span class="text-[11px] text-muted">{{ t('pages.projects.binding.targetToolHint') }}</span>
        </div>
        <div class="grid grid-cols-2 gap-2">
          <button
            v-for="tool in targetTools"
            :key="tool.id"
            type="button"
            class="group flex min-w-0 items-center justify-between gap-3 rounded-[14px] border px-4 py-3 text-left transition"
            :class="projectsStore.bindingDraft.targetToolId === tool.id
              ? 'border-accent/70 bg-accent/[0.12] text-text shadow-[0_10px_28px_rgb(0_0_0/0.08)]'
              : 'border-line/80 bg-panel/80 text-muted hover:border-accent/35 hover:bg-accent/5 hover:text-text'"
            :disabled="loading"
            @click="projectsStore.setBindingTargetToolId(tool.id)"
          >
            <span class="min-w-0">
              <span class="block text-sm font-semibold leading-5">{{ tool.name }}</span>
              <span class="mt-0.5 block truncate text-[11px] leading-4 text-muted">{{ tool.output }}</span>
            </span>
            <CheckCircleFilled
              class="shrink-0 text-base transition"
              :class="projectsStore.bindingDraft.targetToolId === tool.id ? 'text-accent opacity-100' : 'text-muted/30 opacity-40 group-hover:opacity-70'"
            />
          </button>
        </div>
      </div>

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
