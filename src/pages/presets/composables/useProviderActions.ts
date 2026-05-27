import { Modal } from 'ant-design-vue'
import { CopyOutlined, DeleteOutlined } from '@ant-design/icons-vue'
import { useI18n } from 'vue-i18n'
import { useProvidersStore } from '@/shared/stores/providers'
import type { CardMoreMenuItem } from '@/shared/types/ui'

export function useProviderActions() {
  const { t } = useI18n()
  const providerStore = useProvidersStore()

  const moreItems: CardMoreMenuItem[] = [
    { key: 'duplicate', label: t('catalog.action.duplicate'), icon: CopyOutlined },
    { key: 'delete', label: t('common.delete'), icon: DeleteOutlined, danger: true },
  ]

  function handleMore(key: string, id: number) {
    if (key === 'duplicate') {
      void providerStore.duplicateItem(id)
      return
    }
    if (key === 'delete') {
      Modal.confirm({
        title: t('pages.providers.deleteProviderTitle'),
        content: t('pages.providers.deleteProviderContent'),
        okText: t('common.delete'),
        cancelText: t('common.close'),
        okType: 'danger',
        onOk: () => providerStore.deleteItem(id),
      })
    }
  }

  return {
    handleMore,
    moreItems,
    providerStore,
  }
}
