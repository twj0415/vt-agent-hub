import { computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import { useI18n } from 'vue-i18n'
import { useProvidersStore } from '@/shared/stores/providers'
import { toolRegistry, type ToolId } from '@/shared/tool-registry'
import { useProviderCards } from './useProviderCards'

export function useProvidersWorkbench() {
  const { t } = useI18n()
  const route = useRoute()
  const providerStore = useProvidersStore()
  const { providerCards } = useProviderCards()
  const providerFilterOptions = computed(() => toolRegistry
    .filter((tool) => tool.enabled && tool.capabilities.presets)
    .map((tool) => ({
      iconSrc: tool.iconSrc,
      iconText: tool.iconText,
      label: t(tool.nameKey),
      value: tool.id,
    })),
  )
  const providerListBusy = computed(() => providerStore.loading || providerStore.saving)

  onMounted(() => {
    void providerStore.hydrate()
    if (route.query.action === 'create') providerStore.openCreate()
  })

  function setProviderFilter(value: string | number) {
    providerStore.setFilterToolId(Number(value) as ToolId)
  }

  return {
    providerCards,
    providerFilterOptions,
    providerListBusy,
    providerStore,
    setProviderFilter,
    t,
  }
}
