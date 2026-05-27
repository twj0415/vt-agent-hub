import { computed, onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute } from 'vue-router'
import { useSkillStore } from '@/shared/stores/skills'
import { skillCategoryOptions } from '@/shared/taxonomy'
import { useSkillCards } from './useSkillCards'

export function useSkillsWorkbench() {
  const { t } = useI18n()
  const route = useRoute()
  const skillStore = useSkillStore()
  const cards = useSkillCards()

  const categoryOptions = computed(() => skillCategoryOptions.map((item) => ({ value: item.value, label: t(item.labelKey) })))
  const skillListBusy = computed(() => skillStore.importLoading)

  onMounted(() => {
    if (route.query.action === 'create') {
      skillStore.openCreate()
    }
  })

  return {
    categoryOptions,
    skillListBusy,
    skillStore,
    t,
    ...cards,
  }
}
