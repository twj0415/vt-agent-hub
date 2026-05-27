import { defineStore } from 'pinia'
import { storageKeys } from '@/shared/constants/storage'

export type ThemePreset = 'apple' | 'warm' | 'clean' | 'dark'

export const themePresets = [
  { value: 'apple', labelKey: 'theme.preset.apple' },
  { value: 'warm', labelKey: 'theme.preset.warm' },
  { value: 'clean', labelKey: 'theme.preset.clean' },
  { value: 'dark', labelKey: 'theme.preset.dark' },
] as const

export function isThemePreset(value: string | null): value is ThemePreset {
  return value === 'warm' || value === 'clean' || value === 'apple' || value === 'dark'
}

function normalizeThemePreset(value: string | null): ThemePreset {
  if (isThemePreset(value)) return value
  return 'apple'
}

export const useThemeStore = defineStore('theme', {
  state: () => ({
    preset: 'apple' as ThemePreset,
  }),
  actions: {
    setPreset(preset: ThemePreset) {
      this.preset = preset
    },
    hydrate() {
      this.preset = normalizeThemePreset(localStorage.getItem(storageKeys.theme))
    },
    persist() {
      localStorage.setItem(storageKeys.theme, this.preset)
    },
  },
})
