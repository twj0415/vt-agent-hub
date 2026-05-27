import { computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useRuleStore } from '@/shared/stores/rules'
import { useRuleCards } from './useRuleCards'

export function useRulesWorkbench() {
  const { t } = useI18n()
  const route = useRoute()
  const ruleStore = useRuleStore()
  const { ruleCards } = useRuleCards()
  const ruleListBusy = computed(() => false)

  onMounted(() => {
    if (route.query.action === 'create') ruleStore.openCreate()
  })

  return {
    ruleCards,
    ruleListBusy,
    ruleStore,
    t,
  }
}
