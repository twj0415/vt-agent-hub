import type { Config } from 'tailwindcss'

export default {
  content: ['./index.html', './src/**/*.{vue,ts}'],
  theme: {
    extend: {
      fontFamily: {
        mono: [
          'SF Mono',
          'ui-monospace',
          'Menlo',
          'Monaco',
          'Cascadia Mono',
          'Segoe UI Mono',
          'monospace',
        ],
      },
      colors: {
        bg: 'rgb(var(--vt-color-bg) / <alpha-value>)',
        panel: 'rgb(var(--vt-color-panel) / <alpha-value>)',
        'panel-strong': 'rgb(var(--vt-color-panel-strong) / <alpha-value>)',
        line: 'rgb(var(--vt-color-line) / <alpha-value>)',
        'line-strong': 'rgb(var(--vt-color-line-strong) / <alpha-value>)',
        text: 'rgb(var(--vt-color-text) / <alpha-value>)',
        muted: 'rgb(var(--vt-color-muted) / <alpha-value>)',
        accent: 'rgb(var(--vt-color-accent) / <alpha-value>)',
        'accent-strong': 'rgb(var(--vt-color-accent-strong) / <alpha-value>)',
        success: 'rgb(var(--vt-color-success) / <alpha-value>)',
        warning: 'rgb(var(--vt-color-warning) / <alpha-value>)',
        danger: 'rgb(var(--vt-color-danger) / <alpha-value>)',
        'surface-1': 'rgb(var(--vt-color-surface-1) / <alpha-value>)',
        'surface-2': 'rgb(var(--vt-color-surface-2) / <alpha-value>)',
        'surface-3': 'rgb(var(--vt-color-surface-3) / <alpha-value>)',
      },
      fontSize: {
        'vt-xs': ['var(--vt-text-xs)', { lineHeight: '16px' }],
        'vt-sm': ['var(--vt-text-sm)', { lineHeight: '18px' }],
        'vt-base': ['var(--vt-text-base)', { lineHeight: '20px' }],
        'vt-lg': ['var(--vt-text-lg)', { lineHeight: '24px' }],
        'vt-xl': ['var(--vt-text-xl)', { lineHeight: '28px' }],
      },
      borderRadius: {
        'vt-xl': 'var(--vt-radius-xl)',
        'vt-lg': 'var(--vt-radius-lg)',
        'vt-md': 'var(--vt-radius-md)',
        'vt-sm': 'var(--vt-radius-sm)',
      },
      boxShadow: {
        shell: 'var(--vt-shadow-shell)',
        card: 'var(--vt-shadow-card)',
        soft: 'var(--vt-shadow-soft)',
        surface: 'var(--vt-shadow-surface)',
        'surface-lg': 'var(--vt-shadow-surface-lg)',
        'elevation-1': 'var(--vt-shadow-elevation-1)',
        'elevation-2': 'var(--vt-shadow-elevation-2)',
        'elevation-3': 'var(--vt-shadow-elevation-3)',
        'glow-accent': 'var(--vt-shadow-glow-accent)',
        'glow-accent-soft': 'var(--vt-shadow-glow-accent-soft)',
      },
      transitionTimingFunction: {
        standard: 'var(--vt-ease-standard)',
        spring: 'var(--vt-ease-spring)',
      },
      transitionDuration: {
        fast: 'var(--vt-duration-fast)',
        normal: 'var(--vt-duration-normal)',
        slow: 'var(--vt-duration-slow)',
      },
    },
  },
  plugins: [],
} satisfies Config
