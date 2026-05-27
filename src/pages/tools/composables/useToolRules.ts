import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTaxonomyOptions } from '@/shared/composables/useTaxonomyOptions'
import { useRuleStore } from '@/shared/stores/rules'
import { useToolsStore } from '@/shared/stores/tools'
import { markdownDescription } from '@/shared/utils/markdown'

export function useToolRules() {
  const { t } = useI18n()
  const ruleStore = useRuleStore()
  const toolsStore = useToolsStore()
  const taxonomyOptions = useTaxonomyOptions()

  const availableRules = computed(() => {
    const boundRuleIds = new Set(
      toolsStore.globalRuleBinding?.items
        .filter((item) => item.itemType === 'rule')
        .map((item) => item.assetId) ?? [],
    )
    return ruleStore.items.filter((item) => {
      return !boundRuleIds.has(item.id)
    })
  })

  function categoryLabel(categoryCode: number) {
    return taxonomyOptions.ruleCategories.labelOf(categoryCode, String(categoryCode))
  }

  function ruleDescription(rule: { summary: string; body: string }) {
    const summary = rule.summary.trim()
    const value = summary && !summary.toLowerCase().startsWith('imported from') ? summary : markdownDescription(rule.body)
    return value || '-'
  }

  function bindingRuleDetails(binding = toolsStore.globalRuleBinding) {
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

  const boundRules = computed(() => bindingRuleDetails())
  const boundRuleNames = computed(() => {
    const names = boundRules.value.map((rule) => rule.name)
    return names.length ? names.join(' / ') : t('pages.tools.binding.empty')
  })

  return {
    availableRules,
    boundRules,
    boundRuleNames,
    categoryLabel,
    ruleDescription,
  }
}
