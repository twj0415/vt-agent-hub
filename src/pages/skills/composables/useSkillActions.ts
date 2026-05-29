import { Modal } from 'ant-design-vue'
import { AppstoreOutlined, DeleteOutlined } from '@ant-design/icons-vue'
import { useI18n } from 'vue-i18n'
import type { CardMoreMenuItem } from '@/shared/types/ui'
import { useSkillStore } from '@/shared/stores/skills'

export function useSkillActions() {
  const { t } = useI18n()
  const skillStore = useSkillStore()

  function openSkillDetail(id: number) {
    skillStore.select(id)
    skillStore.setDetailOpen(true)
  }

  function confirmSkillDelete(id: number) {
    Modal.confirm({
      title: t('catalog.action.delete'),
      okText: t('common.delete'),
      cancelText: t('common.close'),
      okType: 'danger',
      onOk: () => skillStore.deleteItem(id),
    })
  }

  function handleMore(key: string, item: { id: number }) {
    if (key === 'bind-tool') skillStore.openToolBinding(item.id)
    if (key === 'delete') confirmSkillDelete(item.id)
  }

  const moreItems: CardMoreMenuItem[] = [
    { key: 'bind-tool', label: t('pages.skills.bindTool'), icon: AppstoreOutlined },
    { key: 'delete', label: t('common.delete'), icon: DeleteOutlined, danger: true },
  ]

  return {
    confirmSkillDelete,
    handleMore,
    moreItems,
    openSkillDetail,
    skillStore,
  }
}
