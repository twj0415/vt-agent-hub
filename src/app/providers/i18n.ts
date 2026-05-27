import { watch } from 'vue'
import type { Pinia } from 'pinia'
import type { Router } from 'vue-router'
import { i18n } from '@/shared/i18n'
import { useI18nStore } from '@/shared/stores/i18n'

function syncDocumentTitle(router: Router) {
  const titleKey = String(router.currentRoute.value.meta.titleKey ?? 'app.name')
  const appName = i18n.global.t('app.name')

  document.title = `${i18n.global.t(titleKey)} | ${appName}`
}

export function setupI18nProvider(pinia: Pinia, router: Router) {
  const store = useI18nStore(pinia)

  store.hydrate()

  watch(
    () => store.locale,
    (locale) => {
      i18n.global.locale.value = locale
      document.documentElement.lang = locale
      store.persist()
      syncDocumentTitle(router)
    },
    { immediate: true },
  )

  router.afterEach(() => {
    syncDocumentTitle(router)
  })
}
