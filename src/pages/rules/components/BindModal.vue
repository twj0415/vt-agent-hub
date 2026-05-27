<script setup lang="ts">
import { computed } from 'vue'
import { InfoCircleOutlined } from '@ant-design/icons-vue'
import { useI18n } from 'vue-i18n'
import SelectableList from '@/shared/components/feedback/SelectableList.vue'
import VTModal from '@/shared/components/feedback/VTModal.vue'
import { useConfirm } from '@/shared/composables/useConfirm'
import { useProjectsStore } from '@/shared/stores/projects'
import { useRuleStore } from '@/shared/stores/rules'
import { useToolsStore } from '@/shared/stores/tools'
import { getToolById } from '@/shared/tool-registry'

const { t } = useI18n()
const ruleStore = useRuleStore()
const projectsStore = useProjectsStore()
const toolsStore = useToolsStore()
const { confirmAction } = useConfirm()

const loading = computed(() => ruleStore.bindLoading)
const activeRule = computed(() => ruleStore.items.find((item) => item.id === ruleStore.bindDraft.ruleId) ?? null)
const isProjectMode = computed(() => ruleStore.bindDraft.mode === 'project')
const activeToolName = computed(() => t(getToolById(toolsStore.activeId)?.nameKey ?? 'common.codex'))

async function saveBinding() {
  const result = await ruleStore.saveBindingDraft()
  if (result !== 'needsToolOverwriteConfirm') return

  confirmAction({
    title: t('pages.rules.overwriteToolFileTitle'),
    content: t('pages.rules.overwriteToolFileContent', { tool: activeToolName.value }),
    okText: t('pages.rules.overwriteToolFileOk'),
    danger: true,
    onOk: async () => {
      await ruleStore.saveBindingDraft({ forceToolOverwrite: true })
    },
  })
}
</script>

<template>
  <VTModal
    :open="ruleStore.bindOpen"
    :width="760"
    :title="isProjectMode ? t('pages.rules.bindProject') : t('pages.rules.bindTool')"
    :ok-text="t('catalog.action.save')"
    :cancel-text="t('common.close')"
    :body-style="{ maxHeight: '72vh', overflow: 'hidden', padding: '18px 20px' }"
    :loading="loading"
    @ok="saveBinding"
    @close="ruleStore.setBindOpen(false)"
  >
    <div class="flex min-h-0 flex-col gap-4">
      <a-alert
        type="info"
        show-icon
        :message="isProjectMode ? t('pages.rules.bindProjectDesc') : t('pages.rules.bindToolDesc', { tool: activeToolName })"
      >
        <template #icon><InfoCircleOutlined /></template>
      </a-alert>

      <SelectableList :items="isProjectMode ? projectsStore.items : [{ id: toolsStore.activeId }]" :empty-text="t('common.emptyData')">
        <button
          v-if="isProjectMode"
          v-for="project in projectsStore.items"
          :key="project.id"
          type="button"
          class="flex w-full items-center justify-between gap-3 border-b border-line/80 px-4 py-3 text-left transition last:border-b-0 hover:bg-accent/5"
          :disabled="loading"
          @click="ruleStore.toggleBindProject(project.id)"
        >
          <span class="flex min-w-0 items-center gap-3">
            <a-checkbox
              :checked="ruleStore.bindDraft.selectedProjectIds.includes(project.id)"
              :disabled="loading"
              @click.stop
              @change="ruleStore.toggleBindProject(project.id)"
            />
            <span class="min-w-0">
              <span class="block truncate text-sm font-semibold text-text">{{ project.name }}</span>
              <span class="block truncate text-xs leading-5 text-muted">{{ project.path }}</span>
            </span>
          </span>
        </button>

        <button
          v-else
          type="button"
          class="flex w-full items-center justify-between gap-3 px-4 py-3 text-left transition hover:bg-accent/5"
          :disabled="loading"
          @click="ruleStore.toggleBindTool(toolsStore.activeId)"
        >
          <span class="flex min-w-0 items-center gap-3">
            <a-checkbox
              :checked="ruleStore.bindDraft.selectedToolIds.includes(toolsStore.activeId)"
              :disabled="loading"
              @click.stop
              @change="ruleStore.toggleBindTool(toolsStore.activeId)"
            />
            <span class="min-w-0">
              <span class="block truncate text-sm font-semibold text-text">{{ activeToolName }}</span>
              <span class="block truncate text-xs leading-5 text-muted">{{ t('pages.rules.bindToolOnlyCurrent') }}</span>
            </span>
          </span>
        </button>

      </SelectableList>
    </div>
  </VTModal>
</template>
