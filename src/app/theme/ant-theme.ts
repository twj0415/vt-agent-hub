import { computed } from 'vue'
import { theme } from 'ant-design-vue'
import type { ThemeConfig } from 'ant-design-vue/es/config-provider/context'
import { useThemeStore, type ThemePreset } from '@/shared/stores/theme'

type ThemePalette = {
  bg: string
  panel: string
  panelStrong: string
  line: string
  text: string
  muted: string
  accent: string
  accentStrong: string
  success: string
  warning: string
  danger: string
}

const palettes: Record<ThemePreset, ThemePalette> = {
  warm: {
    bg: '#f4efe7',
    panel: '#fffdf8',
    panelStrong: '#ffffff',
    line: '#e6dccb',
    text: '#1f1c17',
    muted: '#857263',
    accent: '#c8612d',
    accentStrong: '#a54b21',
    success: '#2f855f',
    warning: '#c9771b',
    danger: '#c24135',
  },
  clean: {
    bg: '#1e1e1e',
    panel: '#252526',
    panelStrong: '#2d2d30',
    line: '#3e3e42',
    text: '#e2e4e8',
    muted: '#969ca8',
    accent: '#4ec9b0',
    accentStrong: '#7ed8c4',
    success: '#6a9955',
    warning: '#dcdcaa',
    danger: '#f44747',
  },
  apple: {
    bg: '#f5f5f7',
    panel: '#fafafc',
    panelStrong: '#ffffff',
    line: '#e0e0e2',
    text: '#1d1d1f',
    muted: '#7a7a7f',
    accent: '#0066cc',
    accentStrong: '#0071e3',
    success: '#34a853',
    warning: '#bf7c1c',
    danger: '#d14036',
  },
  dark: {
    bg: '#0c0d12',
    panel: '#111219',
    panelStrong: '#181922',
    line: '#2d303e',
    text: '#e8ecf7',
    muted: '#8f97ab',
    accent: '#768eff',
    accentStrong: '#9aaaff',
    success: '#4fc58a',
    warning: '#dea352',
    danger: '#ec6767',
  },
}

function createAntThemeConfig(preset: ThemePreset): ThemeConfig {
  const palette = palettes[preset]
  const isDark = preset === 'dark' || preset === 'clean'
  const isApple = preset === 'apple'
  const subtleBg = isDark ? '#151722' : isApple ? '#f5f5f7' : palette.bg
  const hoverBg = isDark ? '#202333' : isApple ? '#f2f2f4' : `${palette.accent}12`
  const activeBg = isDark ? '#263052' : isApple ? '#e8f1fb' : `${palette.accent}18`
  const modalShadow = isDark
    ? '0 28px 76px rgba(0, 0, 0, 0.58), 0 3px 12px rgba(0, 0, 0, 0.42)'
    : isApple
      ? '0 24px 72px rgba(0, 0, 0, 0.16), 0 2px 8px rgba(0, 0, 0, 0.05)'
      : '0 20px 56px rgba(0, 0, 0, 0.18), 0 1px 4px rgba(0, 0, 0, 0.06)'

  return {
    algorithm: isDark ? theme.darkAlgorithm : theme.defaultAlgorithm,
    token: {
      colorPrimary: palette.accent,
      colorInfo: palette.accent,
      colorSuccess: palette.success,
      colorWarning: palette.warning,
      colorError: palette.danger,
      colorBgBase: palette.bg,
      colorBgContainer: palette.panel,
      colorBgElevated: palette.panelStrong,
      colorBgLayout: palette.bg,
      colorFillSecondary: isDark ? '#2a2a2c' : `${palette.text}0f`,
      colorFillTertiary: isDark ? '#242426' : `${palette.text}0a`,
      colorBorder: palette.line,
      colorBorderSecondary: palette.line,
      colorText: palette.text,
      colorTextSecondary: palette.muted,
      colorTextTertiary: palette.muted,
      borderRadius: 10,
      borderRadiusLG: 14,
      controlHeight: 32,
      controlHeightSM: 28,
      controlHeightLG: 40,
      fontSize: 13,
      controlPaddingHorizontal: 12,
      controlOutline: `${palette.accent}2e`,
      controlOutlineWidth: 2,
    },
    components: {
      Button: {
        borderRadius: 999,
        borderRadiusSM: 999,
        borderRadiusLG: 999,
        controlHeight: 30,
        controlHeightSM: 26,
        controlHeightLG: 38,
        controlOutline: `${palette.accent}2e`,
        controlOutlineWidth: 2,
        fontSize: 13,
        fontSizeSM: 12,
        fontSizeLG: 14,
        lineHeight: 1.5715,
      },
      Card: {
        colorBgContainer: palette.panelStrong,
        colorBorderSecondary: palette.line,
      },
      Drawer: {
        colorBgElevated: palette.panelStrong,
        colorText: palette.text,
        colorTextHeading: palette.text,
        colorSplit: palette.line,
      },
      Dropdown: {
        colorBgElevated: palette.panelStrong,
        colorText: palette.text,
        controlItemBgHover: hoverBg,
        borderRadiusLG: 10,
      },
      Empty: {
        colorTextDescription: palette.muted,
      },
      Modal: {
        colorIcon: palette.muted,
        colorIconHover: palette.text,
        colorBgElevated: palette.panelStrong,
        colorTextHeading: palette.text,
        borderRadiusLG: 16,
        boxShadow: modalShadow,
      },
      Menu: {
        colorItemText: palette.text,
        colorItemTextHover: palette.text,
        colorItemTextSelected: palette.text,
        colorItemBgHover: hoverBg,
        colorItemBgSelected: activeBg,
        colorDangerItemText: palette.danger,
        colorDangerItemTextHover: palette.danger,
        colorDangerItemBgActive: `${palette.danger}1a`,
        colorDangerItemBgSelected: `${palette.danger}1a`,
        radiusItem: 9,
        itemMarginInline: 5,
      },
      Input: {
        colorBgContainer: subtleBg,
        colorBorder: palette.line,
        colorText: palette.text,
        colorTextPlaceholder: isDark ? '#78787d' : palette.muted,
        borderRadius: 10,
        controlOutlineWidth: 2,
        controlOutline: `${palette.accent}2e`,
      },
      Segmented: {
        colorBgLayout: subtleBg,
        colorBgElevated: palette.panelStrong,
        colorText: palette.text,
        colorTextLabel: palette.muted,
      },
      Select: {
        colorBgContainer: subtleBg,
        colorBgElevated: palette.panelStrong,
        colorBorder: palette.line,
        colorText: palette.text,
        controlOutline: `${palette.accent}2e`,
        controlOutlineWidth: 2,
      },
      Tabs: {
        colorPrimary: palette.accent,
        colorPrimaryActive: palette.accentStrong,
        colorPrimaryHover: palette.accentStrong,
        colorText: palette.muted,
      },
    },
  }
}

export function useAntTheme() {
  const themeStore = useThemeStore()

  return computed(() => createAntThemeConfig(themeStore.preset))
}
