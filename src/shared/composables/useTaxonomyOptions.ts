import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { projectTypeOptions, ruleCategoryOptions, skillCategoryOptions } from '@/shared/taxonomy'

type TaxonomyOption = { value: number; labelKey: string }

function useOptions(source: readonly TaxonomyOption[]) {
  const { t } = useI18n()
  const options = computed(() => source.map((item) => ({ value: item.value, label: t(item.labelKey) })))
  const labelOf = (value: number, fallback = '-') => options.value.find((item) => item.value === value)?.label ?? fallback

  return { options, labelOf }
}

export function useTaxonomyOptions() {
  return {
    ruleCategories: useOptions(ruleCategoryOptions),
    skillCategories: useOptions(skillCategoryOptions),
    projectTypes: useOptions(projectTypeOptions),
  }
}
