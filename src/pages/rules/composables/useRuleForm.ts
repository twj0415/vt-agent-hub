import { useTaxonomyOptions } from '@/shared/composables/useTaxonomyOptions'
import { useRuleStore } from '@/shared/stores/rules'

export type RuleField = 'name' | 'code' | 'summary' | 'categoryCode' | 'body'

export function useRuleForm() {
  const ruleStore = useRuleStore()
  const taxonomyOptions = useTaxonomyOptions()
  const categoryOptions = taxonomyOptions.ruleCategories.options

  function setDraftField(key: RuleField, value: string | number) {
    if (key === 'code' || key === 'categoryCode') ruleStore.setDraftField(key, Number(value))
    else ruleStore.setDraftField(key, String(value))
  }

  return {
    categoryOptions,
    ruleStore,
    setDraftField,
  }
}
