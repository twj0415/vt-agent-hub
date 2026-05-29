import type { EntityState } from '@/shared/constants/status'

const codexIcon = new URL('../../assets/tool-icons/codex.png', import.meta.url).href
const claudeIcon = new URL('../../assets/tool-icons/claude.svg', import.meta.url).href
const cursorIcon = new URL('../../assets/tool-icons/cursor.png', import.meta.url).href

export const toolIds = {
  codex: 101,
  claude: 102,
  cursor: 103,
} as const

export type ToolId = (typeof toolIds)[keyof typeof toolIds]

export type ToolCapabilityKey =
  | 'rules'
  | 'presets'
  | 'credentials'
  | 'skillInstall'
  | 'liveScan'
  | 'agentsOutput'

export type ToolRegistryItem = {
  id: ToolId
  key: 'codex' | 'claude' | 'cursor'
  nameKey: string
  descKey: string
  status: EntityState
  enabled: boolean
  iconText: string
  iconSrc?: string
  capabilities: Record<ToolCapabilityKey, boolean>
}

export const capabilityOrder: ToolCapabilityKey[] = [
  'rules',
  'presets',
  'credentials',
  'skillInstall',
  'liveScan',
  'agentsOutput',
]

export const toolRegistry: ToolRegistryItem[] = [
  {
    id: toolIds.codex,
    key: 'codex',
    nameKey: 'common.codex',
    descKey: 'pages.projects.toolCodexDesc',
    status: 'ready',
    enabled: true,
    iconText: 'CX',
    iconSrc: codexIcon,
    capabilities: {
      rules: true,
      presets: true,
      credentials: true,
      skillInstall: true,
      liveScan: true,
      agentsOutput: true,
    },
  },
  {
    id: toolIds.claude,
    key: 'claude',
    nameKey: 'common.claude',
    descKey: 'pages.projects.toolClaudeDesc',
    status: 'ready',
    enabled: true,
    iconText: 'CL',
    iconSrc: claudeIcon,
    capabilities: {
      rules: true,
      presets: true,
      credentials: false,
      skillInstall: true,
      liveScan: true,
      agentsOutput: true,
    },
  },
  {
    id: toolIds.cursor,
    key: 'cursor',
    nameKey: 'common.cursor',
    descKey: 'pages.projects.toolCursorDesc',
    status: 'planned',
    enabled: false,
    iconText: 'CU',
    iconSrc: cursorIcon,
    capabilities: {
      rules: true,
      presets: true,
      credentials: false,
      skillInstall: false,
      liveScan: true,
      agentsOutput: true,
    },
  },
]

export function getToolById(id: ToolId | number) {
  return toolRegistry.find((item) => item.id === id) ?? null
}
