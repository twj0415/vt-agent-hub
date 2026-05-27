import { computed, ref } from 'vue'
import { useRuleStore } from '@/shared/stores/rules'
import { getToolById, toolIds, type ToolId } from '@/shared/tool-registry'
import { useProjectsStore } from '@/shared/stores/projects'
import { useToolsStore } from '@/shared/stores/tools'
import { useToolContextStore } from '@/shared/stores/tool-context'
import { useI18n } from 'vue-i18n'
import { entityStateTone } from '@/shared/constants/status'
import { useTaxonomyOptions } from '@/shared/composables/useTaxonomyOptions'
import { projectStatusMeta, projectStatusNoteKey } from '@/pages/projects/utils/status'
import { useRuleCards } from './useRuleCards'

const detailTab = ref<'body' | 'projects' | 'tools'>('body')

export function useRuleDetail() {
  const { t } = useI18n()
  const ruleStore = useRuleStore()
  const projectsStore = useProjectsStore()
  const toolsStore = useToolsStore()
  const toolContextStore = useToolContextStore()
  const taxonomyOptions = useTaxonomyOptions()
  const { projectNames } = useRuleCards()
  const activeRule = computed(() => ruleStore.activeItem)
  const relationActionLoading = computed(() => projectsStore.bindLoading || toolsStore.bindLoading)
  const detailProjectRelations = computed(() => {
    if (!activeRule.value) return []
    const names = new Set(projectNames(activeRule.value))
    return projectsStore.items
      .filter((project) => names.has(project.name))
      .map((project) => {
        const meta = projectStatusMeta(project.outputScan)
        return {
          id: project.id,
          name: project.name,
          description: project.path,
          ruleIds: project.ruleBindings
            .find((binding) => binding.toolId == null)
            ?.items.filter((item) => item.itemType === 'rule')
            .map((item) => item.assetId) ?? [],
          meta: taxonomyOptions.projectTypes.labelOf(project.projectType, String(project.projectType)),
          statusLabel: t(meta.labelKey),
          statusNote: t(projectStatusNoteKey(project.outputScan)),
          statusTone: meta.tone,
        }
      })
  })
  const detailToolRelations = computed(() => {
    if (!activeRule.value) return []
    return activeRule.value.impact?.globalToolIds.flatMap((id) => {
      const tool = getToolById(id)
      if (!tool) return []
      const isActive = tool.id === toolsStore.activeId
      return [{
        id: tool.id,
        name: t(tool?.nameKey ?? 'common.codex'),
        description: isActive ? toolsStore.diagnostics.liveConfigPath || t('common.empty') : t(tool?.descKey ?? 'common.empty'),
        ruleIds: toolsStore.globalRuleBinding?.items
          .filter((item) => item.itemType === 'rule')
          .map((item) => item.assetId) ?? [],
        meta: isActive ? toolsStore.diagnostics.version || '-' : '-',
        statusLabel: tool?.enabled ? t('common.ready') : t('common.disabled'),
        statusNote: tool?.enabled ? t('pages.tools.binding.enabledNote') : t('pages.tools.binding.disabledNote'),
        statusTone: tool?.enabled ? entityStateTone(tool.status) : 'planned',
      }]
    }) ?? []
  })

  function openRuleDetail(id: number) {
    detailTab.value = 'body'
    ruleStore.select(id)
    ruleStore.setDetailOpen(true)
  }

  async function unbindProjectRelation(projectId: number) {
    if (!activeRule.value) return
    const relation = detailProjectRelations.value.find((item) => item.id === projectId)
    if (!relation) return
    await projectsStore.saveProjectRuleIdsAndSync(
      projectId,
      relation.ruleIds.filter((id) => id !== activeRule.value?.id),
      toolContextStore.activeToolId,
      {
        notify: true,
        refreshRules: true,
        successAppliedKey: 'feedback.projectRuleUnboundApplied',
        successSkippedKey: 'feedback.projectRuleUnboundSkipped',
      },
    )
  }

  async function unbindToolRelation(toolId: ToolId) {
    if (!activeRule.value || !toolsStore.isToolEnabled(toolId)) return
    const relation = detailToolRelations.value.find((item) => item.id === toolId)
    if (!relation) return
    await toolsStore.saveToolRuleIdsAndSync(toolId, relation.ruleIds.filter((id) => id !== activeRule.value?.id), {
      notify: true,
      refreshRules: true,
    })
  }

  return {
    activeRule,
    detailProjectRelations,
    detailTab,
    detailToolRelations,
    openRuleDetail,
    relationActionLoading,
    unbindProjectRelation,
    unbindToolRelation,
  }
}
