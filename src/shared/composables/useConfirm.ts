import { Modal } from 'ant-design-vue'
import { useI18n } from 'vue-i18n'

type ConfirmOptions = {
  content: string
  danger?: boolean
  okText?: string
  title?: string
  onOk: () => void | Promise<void>
}

export function useConfirm() {
  const { t } = useI18n()

  function confirmAction(options: ConfirmOptions) {
    Modal.confirm({
      title: options.title ?? t('common.confirmTitle'),
      content: options.content,
      okText: options.okText ?? t('common.confirm'),
      cancelText: t('common.close'),
      okType: options.danger ? 'danger' : 'primary',
      style: { top: '24vh' },// 确认弹窗比默认位置略低，避免遮挡顶部操作区。
      onOk: options.onOk,
    })
  }

  return { confirmAction }
}
