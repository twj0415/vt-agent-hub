import { computed } from 'vue'
import { providerCheckTone } from '@/shared/constants/status'
import { translateKey } from '@/shared/i18n/translate'
import { providerCategoryOptions } from '@/shared/providers'
import { useProvidersStore } from '@/shared/stores/providers'

export function useProviderCards() {
  const providerStore = useProvidersStore()
  const emptyText = translateKey('common.empty')

  const providerCards = computed(() => providerStore.currentCards.map((item) => {
    const category = providerCategoryOptions.find((option) => option.value === item.category)
    return {
      ...item,
      categoryLabel: category ? translateKey(category.labelKey) : (item.category || emptyText),
      statusLabel: translateKey(`pages.providers.providerStatus.${item.status}`),
      statusTone: item.active ? 'ready' as const : providerCheckTone(item.status),
      activeLabel: item.active ? translateKey('pages.providers.activeProvider') : '',
      toolTitle: item.toolTitle || emptyText,
      toolTags: item.toolTags,
    }
  }))

  return {
    providerCards,
  }
}
