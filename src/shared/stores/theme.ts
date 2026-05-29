import { defineStore } from 'pinia'
import { storageKeys } from '@/shared/constants/storage'

export type ThemePreset = 'apple' | 'warm' | 'graphite' | 'dark'

export const themePresets = [
  { value: 'apple', labelKey: 'theme.preset.apple' },
  { value: 'warm', labelKey: 'theme.preset.warm' },
  { value: 'graphite', labelKey: 'theme.preset.graphite' },
  { value: 'dark', labelKey: 'theme.preset.dark' },
] as const

export function isThemePreset(value: string | null): value is ThemePreset {
  return value === 'warm' || value === 'graphite' || value === 'apple' || value === 'dark'
}

function normalizeThemePreset(value: string | null): ThemePreset {
  if (value === 'clean') return 'graphite'
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
