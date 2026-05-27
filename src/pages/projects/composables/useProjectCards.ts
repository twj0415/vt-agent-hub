import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTaxonomyOptions } from '@/shared/composables/useTaxonomyOptions'
import { useProjectsStore } from '@/shared/stores/projects'
import { useRuleStore } from '@/shared/stores/rules'
import { useToolsStore } from '@/shared/stores/tools'
import { projectStatusMeta, projectStatusNoteKey } from '../utils/status'
import { useProjectRules } from './useProjectRules'

export function useProjectCards() {
  const { t } = useI18n()
  const ruleStore = useRuleStore()
  const projectsStore = useProjectsStore()
  const toolsStore = useToolsStore()
  const taxonomyOptions = useTaxonomyOptions()
  const { bindingRuleNames } = useProjectRules()
  const toolProjects = computed(() => projectsStore.items)
  const selectedProject = computed(() => projectsStore.activeItem)

  function projectTypeLabel(projectType: number) {
    return taxonomyOptions.projectTypes.labelOf(projectType, String(projectType))
  }

  function uniqueRuleCount(bindings: Array<{ items: Array<{ itemType: string; assetId: number }> }>) {
    return new Set(
      bindings.flatMap((binding) => binding.items.filter((item) => item.itemType === 'rule').map((item) => item.assetId)),
    ).size
  }

  function projectRuleNames(bindings: Array<{ items: Array<{ itemType: string; assetId: number }> }>) {
    return bindings
      .flatMap((binding) => binding.items.filter((item) => item.itemType === 'rule').map((item) => item.assetId))
      .filter((assetId, index, values) => values.indexOf(assetId) === index)
      .map((assetId) => ruleStore.items.find((rule) => rule.id === assetId)?.name ?? `#${assetId}`)
  }

  function previewRuleTags(bindings: Array<{ items: Array<{ itemType: string; assetId: number }> }>) {
    const names = projectRuleNames(bindings)
    if (!names.length) return []
    const visible = names.slice(0, 4)
    return names.length > 4 ? [...visible, '...'] : visible
  }

  function projectRuleBindings(bindings: typeof projectsStore.items[number]['ruleBindings']) {
    return bindings.filter((binding) => binding.toolId == null)
  }

  const projectCards = computed(() =>
    toolProjects.value.map((item) => {
      const isActive = item.id === projectsStore.activeId
      const outputScan = item.outputScan ?? (isActive ? projectsStore.outputScan : null)
      const ruleBindings = projectRuleBindings(item.ruleBindings)
      const meta = projectStatusMeta(outputScan)
      const statusNote = t(projectStatusNoteKey(outputScan))
      const hasRules = Boolean(uniqueRuleCount(ruleBindings))
      const hasBlockingIssue = outputScan?.status === 'missing'
      const canSyncEmptyRules = Boolean(outputScan?.targetExists && outputScan.managed)
      const canWriteProjectOutput = Boolean(!hasBlockingIssue && (hasRules || canSyncEmptyRules) && toolsStore.activeItem?.capabilities.agentsOutput)

      return {
        ...item,
        isActive,
        projectTypeLabel: projectTypeLabel(item.projectType),
        targetPath: outputScan?.targetPath ?? t('pages.projects.binding.targetPending'),
        ruleCount: outputScan?.ruleCount || uniqueRuleCount(ruleBindings) || '-',
        ruleTags: previewRuleTags(ruleBindings),
        rulePreviewTitle: bindingRuleNames({ items: ruleBindings.flatMap((binding) => binding.items) }),
        status: outputScan?.status ?? 'pending',
        statusLabel: t(meta.labelKey),
        statusNote,
        statusTone: meta.tone,
        statusLine: `${t('pages.projects.noteLabel')}: ${statusNote}`,
        canPreview: canWriteProjectOutput,
        canManageRules: !hasBlockingIssue,
        needsRepair: Boolean(!hasBlockingIssue && outputScan?.status === 'attention'),
      }
    }),
  )

  const selectedProjectCard = computed(() => projectCards.value.find((item) => item.id === selectedProject.value?.id) ?? null)

  return {
    projectCards,
    selectedProject,
    selectedProjectCard,
    toolProjects,
    toolsStore,
  }
}
