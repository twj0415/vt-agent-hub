import { message } from 'ant-design-vue'

type NotifyType = 'success' | 'error' | 'warning' | 'info'

function normalizeContent(content: string) {
  return content
    .split('\n')
    .map((line) => line.trim())
    .filter(Boolean)
    .join(' / ')
}

export function notify(type: NotifyType, content: string) {
  const normalized = normalizeContent(content)
  if (!normalized) return
  if (import.meta.env.MODE === 'test') return
  void message[type](normalized)
}

export const notifySuccess = (content: string) => notify('success', content)
export const notifyError = (content: string) => notify('error', content)
export const notifyWarning = (content: string) => notify('warning', content)
export const notifyInfo = (content: string) => notify('info', content)
