import { defineStore } from 'pinia'
import { storageKeys } from '@/shared/constants/storage'

export type LocaleCode = 'zh-CN' | 'en-US'

export const useI18nStore = defineStore('i18n', {
  state: () => ({
    locale: 'zh-CN' as LocaleCode,
  }),
  actions: {
    setLocale(locale: LocaleCode) {
      this.locale = locale
    },
    hydrate() {
      const saved = localStorage.getItem(storageKeys.locale) as LocaleCode | null

      if (saved === 'zh-CN' || saved === 'en-US') {
        this.locale = saved
      }
    },
    persist() {
      localStorage.setItem(storageKeys.locale, this.locale)
    },
  },
})
