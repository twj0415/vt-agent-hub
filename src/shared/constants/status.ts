import type { BadgeTone } from '@/shared/types/ui'

export const targetStateCodes = {
  missing: 501,
  ready: 502,
  error: 503,
  planned: 504,
} as const

export const skillInstallStateCodes = {
  notInstalled: 601,
  installed: 602,
  stale: 603,
  sourceMissing: 604,
  conflict: 605,
  error: 606,
} as const

export const healthStateCodes = {
  normal: 701,
  attention: 702,
  warning: 703,
} as const

export type EntityState = 'missing' | 'ready' | 'error' | 'planned'
export type ProviderCheckStatus = 'unchecked' | 'healthy' | 'degraded' | 'failed'

export type SkillInstallState =
  | 'not_installed'
  | 'installed'
  | 'stale'
  | 'source_missing'
  | 'conflict'
  | 'error'

export function entityStateFromCode(code: number): EntityState {
  if (code === targetStateCodes.missing) return 'missing'
  if (code === targetStateCodes.ready) return 'ready'
  if (code === targetStateCodes.error) return 'error'
  return 'planned'
}

export function entityStateToCode(state: EntityState): number {
  if (state === 'missing') return targetStateCodes.missing
  if (state === 'ready') return targetStateCodes.ready
  if (state === 'error') return targetStateCodes.error
  return targetStateCodes.planned
}

export function entityStateTone(state: EntityState): BadgeTone {
  if (state === 'missing') return 'warning'
  if (state === 'ready') return 'ready'
  if (state === 'error') return 'error'
  return 'planned'
}

export function entityStateBadgeKey(state: EntityState): string {
  if (state === 'missing') return 'common.warning'
  if (state === 'ready') return 'common.ready'
  if (state === 'error') return 'common.warning'
  return 'common.planned'
}

export function skillInstallStateFromCode(code: number): SkillInstallState {
  if (code === skillInstallStateCodes.installed) return 'installed'
  if (code === skillInstallStateCodes.stale) return 'stale'
  if (code === skillInstallStateCodes.sourceMissing) return 'source_missing'
  if (code === skillInstallStateCodes.conflict) return 'conflict'
  if (code === skillInstallStateCodes.error) return 'error'
  return 'not_installed'
}

export function skillInstallStateToCode(state: SkillInstallState): number {
  if (state === 'installed') return skillInstallStateCodes.installed
  if (state === 'stale') return skillInstallStateCodes.stale
  if (state === 'source_missing') return skillInstallStateCodes.sourceMissing
  if (state === 'conflict') return skillInstallStateCodes.conflict
  if (state === 'error') return skillInstallStateCodes.error
  return skillInstallStateCodes.notInstalled
}

export function skillInstallTone(state: SkillInstallState): BadgeTone {
  if (state === 'installed') return 'ready'
  if (state === 'stale' || state === 'conflict') return 'warning'
  if (state === 'error') return 'error'
  return 'planned'
}

export function skillInstallBadgeKey(state: SkillInstallState): string {
  return `catalog.install.${state}`
}

export function targetStateTone(code: number): BadgeTone {
  return entityStateTone(entityStateFromCode(code))
}

export function healthTone(code: number): BadgeTone {
  if (code === healthStateCodes.normal) return 'ready'
  if (code === healthStateCodes.warning) return 'warning'
  return 'warning'
}

export function providerCheckTone(status: ProviderCheckStatus): BadgeTone {
  if (status === 'healthy') return 'ready'
  if (status === 'failed') return 'error'
  if (status === 'degraded') return 'warning'
  return 'planned'
}

export function normalizeProviderCheckStatus(value: string): ProviderCheckStatus {
  if (value === 'healthy' || value === 'degraded' || value === 'failed') return value
  return 'unchecked'
}
