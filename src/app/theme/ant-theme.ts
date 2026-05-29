import { computed } from 'vue'
import { theme } from 'ant-design-vue'
import type { ThemeConfig } from 'ant-design-vue/es/config-provider/context'
import { useThemeStore, type ThemePreset } from '@/shared/stores/theme'

function readVar(name: string): string {
  if (typeof window === 'undefined') return ''
  const raw = getComputedStyle(document.documentElement).getPropertyValue(name).trim()
  if (!raw) return ''
  if (raw.startsWith('#') || raw.startsWith('rgb') || raw.startsWith('hsl')) return raw
  if (/^\d+\s+\d+\s+\d+$/.test(raw)) {
    return `rgb(${raw.split(/\s+/).join(', ')})`
  }
  const slash = raw.match(/^(\d+)\s+(\d+)\s+(\d+)\s*\/\s*([\d.]+)$/)
  if (slash) return `rgba(${slash[1]}, ${slash[2]}, ${slash[3]}, ${slash[4]})`
  return raw
}

function rgba(name: string, alpha: number): string {
  if (typeof window === 'undefined') return 'transparent'
  const raw = getComputedStyle(document.documentElement).getPropertyValue(name).trim()
  if (!raw) return 'transparent'
  const tuple = raw.match(/^(\d+)\s+(\d+)\s+(\d+)/)
  if (!tuple) return raw
  return `rgba(${tuple[1]}, ${tuple[2]}, ${tuple[3]}, ${alpha})`
}

function createAntThemeConfig(preset: ThemePreset): ThemeConfig {
  const isDark = preset === 'dark' || preset === 'graphite'

  const bg = readVar('--vt-color-bg')
  const panel = readVar('--vt-color-panel')
  const panelStrong = readVar('--vt-color-panel-strong')
  const surface2 = readVar('--vt-color-surface-2') || panel
  const line = readVar('--vt-color-line')
  const text = readVar('--vt-color-text')
  const muted = readVar('--vt-color-muted')
  const accent = readVar('--vt-color-accent')
  const accentStrong = readVar('--vt-color-accent-strong')
  const success = readVar('--vt-color-success')
  const warning = readVar('--vt-color-warning')
  const danger = readVar('--vt-color-danger')

  const hoverBg = readVar('--vt-color-state-hover') || rgba('--vt-color-text', 0.05)
  const activeBg = readVar('--vt-color-state-active') || rgba('--vt-color-text', 0.08)
  const selectedBg = readVar('--vt-color-state-selected') || rgba('--vt-color-accent', 0.12)

  const controlOutline = rgba('--vt-color-accent', 0.18)
  const fillSecondary = rgba('--vt-color-text', isDark ? 0.06 : 0.05)
  const fillTertiary = rgba('--vt-color-text', isDark ? 0.04 : 0.035)

  const modalShadow = isDark
    ? '0 24px 64px rgba(0, 0, 0, 0.60), 0 0 0 1px rgba(255, 255, 255, 0.08)'
    : '0 18px 48px rgba(15, 23, 42, 0.16), 0 0 0 1px rgba(0, 0, 0, 0.04)'

  return {
    algorithm: isDark ? theme.darkAlgorithm : theme.defaultAlgorithm,
    token: {
      colorPrimary: accent,
      colorInfo: accent,
      colorSuccess: success,
      colorWarning: warning,
      colorError: danger,
      colorBgBase: bg,
      colorBgContainer: panel,
      colorBgElevated: panelStrong,
      colorBgLayout: bg,
      colorFillSecondary: fillSecondary,
      colorFillTertiary: fillTertiary,
      colorBorder: line,
      colorBorderSecondary: line,
      colorText: text,
      colorTextSecondary: muted,
      colorTextTertiary: muted,
      borderRadius: 8,
      borderRadiusLG: 12,
      borderRadiusSM: 6,
      controlHeight: 32,
      controlHeightSM: 28,
      controlHeightLG: 40,
      fontSize: 13,
      controlPaddingHorizontal: 12,
      controlOutline,
      controlOutlineWidth: 2,
    },
    components: {
      Button: {
        borderRadius: 8,
        borderRadiusSM: 6,
        borderRadiusLG: 10,
        controlHeight: 32,
        controlHeightSM: 28,
        controlHeightLG: 38,
        controlOutline,
        controlOutlineWidth: 2,
        fontSize: 13,
        fontSizeSM: 12,
        fontSizeLG: 14,
        lineHeight: 1.5715,
      },
      Card: {
        colorBgContainer: panel,
        colorBorderSecondary: line,
        borderRadiusLG: 12,
      },
      Drawer: {
        colorBgElevated: panelStrong,
        colorText: text,
        colorTextHeading: text,
        colorSplit: line,
      },
      Dropdown: {
        colorBgElevated: panelStrong,
        colorText: text,
        controlItemBgHover: hoverBg,
        borderRadiusLG: 10,
      },
      Empty: {
        colorTextDescription: muted,
      },
      Modal: {
        colorIcon: muted,
        colorIconHover: text,
        colorBgElevated: panelStrong,
        colorTextHeading: text,
        borderRadiusLG: 12,
        boxShadow: modalShadow,
      },
      Menu: {
        colorItemText: text,
        colorItemTextHover: text,
        colorItemTextSelected: text,
        colorItemBgHover: hoverBg,
        colorItemBgSelected: selectedBg,
        colorDangerItemText: danger,
        colorDangerItemTextHover: danger,
        colorDangerItemBgActive: rgba('--vt-color-danger', 0.10),
        colorDangerItemBgSelected: rgba('--vt-color-danger', 0.10),
        radiusItem: 8,
        itemMarginInline: 4,
      },
      Input: {
        colorBgContainer: surface2,
        colorBorder: line,
        colorText: text,
        colorTextPlaceholder: muted,
        borderRadius: 8,
        controlOutlineWidth: 2,
        controlOutline,
      },
      Segmented: {
        colorBgLayout: surface2,
        colorBgElevated: panelStrong,
        colorText: text,
        colorTextLabel: muted,
      },
      Select: {
        colorBgContainer: surface2,
        colorBgElevated: panelStrong,
        colorBorder: line,
        colorText: text,
        controlOutline,
        controlOutlineWidth: 2,
      },
      Tabs: {
        colorPrimary: accent,
        colorPrimaryActive: accentStrong,
        colorPrimaryHover: accentStrong,
        colorText: muted,
      },
    },
  }
}

export function useAntTheme() {
  const themeStore = useThemeStore()
  return computed(() => createAntThemeConfig(themeStore.preset))
}
