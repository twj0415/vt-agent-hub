import { computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useProvidersStore } from '@/shared/stores/providers'
import { useProviderCards } from './useProviderCards'

export function useProvidersWorkbench() {
  const { t } = useI18n()
  const route = useRoute()
  const providerStore = useProvidersStore()
  const { providerCards } = useProviderCards()
  const providerListBusy = computed(() => providerStore.loading || providerStore.saving)

  onMounted(() => {
    void providerStore.hydrate()
    if (route.query.action === 'create') providerStore.openCreate()
  })

  return {
    providerCards,
    providerListBusy,
    providerStore,
    t,
  }
}
