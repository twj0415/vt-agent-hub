import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { AppstoreOutlined, DeleteOutlined, ProjectOutlined } from '@ant-design/icons-vue'
import { useConfirm } from '@/shared/composables/useConfirm'
import type { CardMoreMenuItem } from '@/shared/types/ui'
import { useRuleStore } from '@/shared/stores/rules'
import { useRuleDetail } from './useRuleDetail'

export function useRuleActions() {
  const { t } = useI18n()
  const ruleStore = useRuleStore()
  const { confirmAction } = useConfirm()
  const { openRuleDetail } = useRuleDetail()

  const moreItems = computed<CardMoreMenuItem[]>(() => [
    { key: 'bind-project', label: t('pages.rules.bindProject'), icon: ProjectOutlined },
    { key: 'bind-tool', label: t('pages.rules.bindTool'), icon: AppstoreOutlined },
    { key: 'delete', label: t('common.delete'), icon: DeleteOutlined, danger: true },
  ])

  // 规则删除确认：统一确认标题，正文只描述本次危险操作。
  async function confirmDelete(id: number, name: string) {
    const canDelete = await ruleStore.ensureRuleCanDelete(id)
    if (!canDelete) return

    confirmAction({
      danger: true,
      okText: t('common.delete'),
      content: t('pages.rules.confirmDeleteContent', { name }),
      onOk: () => ruleStore.deleteItem(id),
    })
  }

  function handleMore(key: string, item: { id: number; name: string }) {
    if (key === 'bind-project') ruleStore.openProjectBinding(item.id)
    if (key === 'bind-tool') ruleStore.openToolBinding(item.id)
    if (key === 'delete') confirmDelete(item.id, item.name)
  }

  return {
    handleMore,
    moreItems,
    openRuleDetail,
    ruleStore,
  }
}
