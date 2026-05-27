import { appRoutes } from '@/shared/config/routes'

export const primaryNav = [
  { key: 'projects', labelKey: 'nav.projects', to: appRoutes.projects },
  { key: 'tools', labelKey: 'nav.tools', to: appRoutes.tools },
  { key: 'rules', labelKey: 'nav.rules', to: appRoutes.rules },
  { key: 'skills', labelKey: 'nav.skills', to: appRoutes.skills },
  { key: 'presets', labelKey: 'nav.providers', to: appRoutes.presets },
  { key: 'history', labelKey: 'nav.history', to: appRoutes.history },
] as const
