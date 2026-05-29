import { describe, expect, it } from 'vitest'
import { capabilityOrder, getToolById, toolIds, toolRegistry } from './index'

describe('toolRegistry', () => {
  it('enables Codex and Claude while keeping Cursor planned', () => {
    const enabledIds = toolRegistry.filter((item) => item.enabled).map((item) => item.id)

    expect(enabledIds).toEqual([toolIds.codex, toolIds.claude])
    expect(toolRegistry.map((item) => item.id)).toEqual([toolIds.codex, toolIds.claude, toolIds.cursor])
    expect(getToolById(toolIds.codex)?.capabilities).toBeDefined()
    expect(getToolById(toolIds.claude)?.capabilities).toEqual({
      rules: true,
      presets: true,
      credentials: false,
      skillInstall: true,
      liveScan: true,
      agentsOutput: true,
    })
    expect(getToolById(toolIds.cursor)?.enabled).toBe(false)
  })

  it('keeps the full capability matrix shape for every tool', () => {
    for (const item of toolRegistry) {
      expect(Object.keys(item.capabilities)).toEqual(capabilityOrder)
    }
  })
})
