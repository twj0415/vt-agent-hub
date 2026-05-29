import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTaxonomyOptions } from '@/shared/composables/useTaxonomyOptions'
import { translateIfExists } from '@/shared/i18n/translate'
import { useRuleStore, type RuleItem } from '@/shared/stores/rules'
import { getToolById } from '@/shared/tool-registry'

function previewTags(values: string[]) {
  if (!values.length) return []
  const visible = values.slice(0, 4)
  return values.length > 4 ? [...visible, '...'] : visible
}

function joined(values: string[], empty: string) {
  return values.length ? values.join(' / ') : empty
}

function text(value: string, emptyText: string) {
  const trimmed = value.trim()
  if (!trimmed) return emptyText
  return translateIfExists(trimmed, trimmed)
}

export function useRuleCards() {
  const { t } = useI18n()
  const ruleStore = useRuleStore()
  const taxonomyOptions = useTaxonomyOptions()
  const currentRules = computed(() => ruleStore.filteredItems)

  // 规则分类字典：把后端分类编码转换为当前语言的展示文案。
  function categoryLabel(categoryCode: number) {
    return taxonomyOptions.ruleCategories.labelOf(categoryCode, t('common.empty'))
  }

  function toolNames(item: Pick<RuleItem, 'impact'>) {
    return item.impact?.globalToolIds.map((id) => t(getToolById(id)?.nameKey ?? 'common.codex')) ?? []
  }

  function projectNames(item: Pick<RuleItem, 'impact'>) {
    return item.impact?.projectNames ?? []
  }

  const ruleCards = computed(() =>
    currentRules.value.map((item) => {
      const emptyText = t('common.empty')
      const projects = projectNames(item)
      const tools = toolNames(item)

      return {
        ...item,
        categoryLabel: categoryLabel(item.categoryCode),
        versionLabel: `v${item.versionNo}`,
        bodyText: text(item.body, emptyText),
        projectTags: previewTags(projects),
        projectTitle: joined(projects, emptyText),
        toolTags: previewTags(tools),
        toolTitle: joined(tools, emptyText),
        summaryText: text(item.summary, emptyText),
      }
    }),
  )

  return {
    categoryLabel,
    currentRules,
    ruleCards,
    toolNames,
    projectNames,
  }
}
