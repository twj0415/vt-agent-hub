import type { App } from 'vue'
import Antd from 'ant-design-vue'
import { createPinia } from 'pinia'
import { createAppRouter } from '@/app/router'
import { i18n } from '@/shared/i18n'
import { useAppStore } from '@/shared/stores/app'
import { setupThemeProvider } from './theme'
import { setupI18nProvider } from './i18n'

import 'ant-design-vue/dist/reset.css'

export function createAppProviders(app: App<Element>) {
  const pinia = createPinia()
  const router = createAppRouter()

  app.use(pinia)
  app.use(router)
  app.use(i18n)
  app.use(Antd)

  setupThemeProvider(pinia)
  setupI18nProvider(pinia, router)
  void useAppStore(pinia).bootstrapAll()
}
