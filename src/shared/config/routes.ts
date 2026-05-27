export const appRoutes = {
  projects: '/projects',
  tools: '/tools',
  rules: '/assets/rules',
  skills: '/assets/skills',
  presets: '/assets/presets',
  history: '/history',
  settings: '/settings',
} as const

export type AppRouteKey = keyof typeof appRoutes

