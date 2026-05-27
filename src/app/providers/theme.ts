import { watch } from 'vue'
import type { Pinia } from 'pinia'
import { useThemeStore, type ThemePreset } from '@/shared/stores/theme'

function applyTheme(preset: ThemePreset) {
  document.documentElement.dataset.theme = preset
  document.documentElement.style.colorScheme = preset === 'dark' ? 'dark' : 'light'
}

export function setupThemeProvider(pinia: Pinia) {
  const store = useThemeStore(pinia)

  store.hydrate()

  watch(
    () => store.preset,
    (preset) => {
      applyTheme(preset)
      store.persist()
    },
    { immediate: true },
  )
}
