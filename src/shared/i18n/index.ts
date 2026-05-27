import { createI18n } from 'vue-i18n'
import zhCN from './messages/zh-CN'
import enUS from './messages/en-US'

export const i18n = createI18n({
  legacy: false,
  locale: 'zh-CN',
  fallbackLocale: 'en-US',
  messages: {
    'zh-CN': zhCN,
    'en-US': enUS,
  },
})
