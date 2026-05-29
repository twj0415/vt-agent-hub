import type {
  ProjectOutputPreview,
  ProjectOutputScan,
  ProjectOutputWriteResult,
  ProjectRuleBinding,
} from '@/shared/api/client'
import { defaultProjectTypeCode, projectTypeCodes } from '@/shared/taxonomy'
import { toolIds } from '@/shared/tool-registry'

export type ProjectType = (typeof projectTypeCodes)[keyof typeof projectTypeCodes]

export type RuleBindingTargetToolId = typeof toolIds.codex | typeof toolIds.claude

export type RuleBindingSelection = {
  selectedRuleIds: number[]
  targetToolId: RuleBindingTargetToolId
}

export type ProjectItem = {
  id: number
  name: string
  path: string
  projectType: ProjectType
  ruleBindings: ProjectRuleBinding[]
  lastOperation: string
  latestBackup: string
  outputScan: ProjectOutputScan | null
}

export type ProjectDraft = {
  id: number | null
  name: string
  path: string
  projectType: ProjectType
  gitTargetPath: string
}

export type ProjectImportMode = 'local' | 'git'
export type ProjectFormIntent = 'import' | 'edit'

export function createOutputScan(): ProjectOutputScan | null {
  return null
}

export function createOutputPreview(): ProjectOutputPreview | null {
  return null
}

export function createOutputWriteResult(): ProjectOutputWriteResult | null {
  return null
}

export function createProjectDraft(): ProjectDraft {
  return {
    id: null,
    name: '',
    path: '',
    projectType: defaultProjectTypeCode,
    gitTargetPath: '',
  }
}

export function createRuleBindingSelection(): RuleBindingSelection {
  return {
    selectedRuleIds: [],
    targetToolId: toolIds.codex,
  }
}
