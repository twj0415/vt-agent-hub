import { describe, expect, it } from 'vitest'
import {
  entityStateFromCode,
  entityStateToCode,
  healthTone,
  skillInstallBadgeKey,
  skillInstallStateFromCode,
  skillInstallStateToCode,
  skillInstallTone,
  targetStateCodes,
} from './status'

describe('status mappings', () => {
  it('maps target entity states through numeric codes', () => {
    expect(entityStateFromCode(targetStateCodes.missing)).toBe('missing')
    expect(entityStateFromCode(targetStateCodes.ready)).toBe('ready')
    expect(entityStateFromCode(targetStateCodes.error)).toBe('error')
    expect(entityStateFromCode(targetStateCodes.planned)).toBe('planned')
    expect(entityStateToCode('missing')).toBe(501)
    expect(entityStateToCode('ready')).toBe(502)
    expect(entityStateToCode('error')).toBe(503)
    expect(entityStateToCode('planned')).toBe(504)
  })

  it('maps skill install states for display', () => {
    expect(skillInstallStateFromCode(602)).toBe('installed')
    expect(skillInstallStateFromCode(603)).toBe('stale')
    expect(skillInstallStateFromCode(605)).toBe('conflict')
    expect(skillInstallStateFromCode(606)).toBe('error')
    expect(skillInstallStateToCode('not_installed')).toBe(601)
    expect(skillInstallStateToCode('conflict')).toBe(605)
    expect(skillInstallStateToCode('error')).toBe(606)
    expect(skillInstallTone('stale')).toBe('warning')
    expect(skillInstallBadgeKey('installed')).toBe('catalog.install.installed')
  })

  it('maps health states to badge tones', () => {
    expect(healthTone(701)).toBe('ready')
    expect(healthTone(702)).toBe('warning')
    expect(healthTone(703)).toBe('warning')
  })
})
