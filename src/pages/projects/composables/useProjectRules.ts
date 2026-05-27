import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTaxonomyOptions } from '@/shared/composables/useTaxonomyOptions'
import { useRuleStore } from '@/shared/stores/rules'
import { useProjectsStore } from '@/shared/stores/projects'
import { markdownDescription } from '@/shared/utils/markdown'

export function useProjectRules() {
  const { t } = useI18n()
  const ruleStore = useRuleStore()
  const projectsStore = useProjectsStore()
  const taxonomyOptions = useTaxonomyOptions()
  const selectedProject = computed(() => projectsStore.activeItem)
  const availableRules = computed(() => {
    const keyword = projectsStore.ruleSearch.trim().toLowerCase()
    return ruleStore.items.filter((item) => {
      if (!keyword) return true
      return item.name.toLowerCase().includes(keyword)
        || item.summary.toLowerCase().includes(keyword)
        || item.key.toLowerCase().includes(keyword)
        || String(item.code).includes(keyword)
    })
  })

  function categoryLabel(categoryCode: number) {
    return taxonomyOptions.ruleCategories.labelOf(categoryCode, String(categoryCode))
  }

  function bindingRuleNames(binding: { items: Array<{ itemType: string; assetId: number }> } | null | undefined) {
    if (!binding) return t('pages.projects.binding.empty')
    const names = binding.items
      .filter((item) => item.itemType === 'rule')
      .map((item) => ruleStore.items.find((rule) => rule.id === item.assetId)?.name ?? `#${item.assetId}`)
    return names.length ? names.join(' / ') : t('pages.projects.binding.empty')
  }

  function bindingRuleDetails(binding: { items: Array<{ itemType: string; assetId: number; assetVersionId: number; assetVersionNo?: number }> } | null | undefined) {
    if (!binding) return []
    return binding.items
      .filter((item) => item.itemType === 'rule')
      .map((item) => {
        const rule = ruleStore.items.find((entry) => entry.id === item.assetId)
        return {
          id: item.assetId,
          name: rule?.name ?? `#${item.assetId}`,
          description: rule ? ruleDescription(rule) : '-',
          categoryLabel: rule ? categoryLabel(rule.categoryCode) : '-',
          versionNo: item.assetVersionNo ?? (rule?.versionId === item.assetVersionId ? rule.versionNo : null),
          versionId: item.assetVersionId,
          hasUpdate: Boolean(rule && rule.versionId !== item.assetVersionId),
          latestVersionNo: rule?.versionNo ?? null,
        }
      })
  }

  function ruleDescription(rule: { summary: string; body: string }) {
    const summary = rule.summary.trim()
    const value = summary && !summary.toLowerCase().startsWith('imported from') ? summary : markdownDescription(rule.body)
    return value || '-'
  }

  const commonRules = computed(() => bindingRuleDetails(selectedProject.value?.ruleBindings.find((item) => item.toolId == null)))

  return {
    availableRules,
    bindingRuleDetails,
    bindingRuleNames,
    categoryLabel,
    commonRules,
    ruleDescription,
  }
}
