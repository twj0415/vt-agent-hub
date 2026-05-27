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
      },
      boxShadow: {
        shell: 'var(--vt-shadow-shell)',
        card: 'var(--vt-shadow-card)',
        soft: 'var(--vt-shadow-soft)',
        surface: 'var(--vt-shadow-surface)',
        'surface-lg': 'var(--vt-shadow-surface-lg)',
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
