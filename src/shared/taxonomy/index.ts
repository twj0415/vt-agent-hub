export type TaxonomyOption<TValue extends number | string = number> = {
  labelKey: string
  value: TValue
}

export const ruleCategoryCodes = {
  personal: 301,
  project: 302,
  base: 303,
  stack: 304,
  codeQuality: 305,
  git: 306,
  domain: 307,
  projectType: 308,
} as const

export const projectTypeCodes = {
  web: 201,
  mini: 202,
  desktop: 203,
} as const

export const skillCategoryCodes = {
  coding: 401,
  uiDesign: 402,
} as const

export const defaultRuleCategoryCode = ruleCategoryCodes.stack
export const defaultProjectTypeCode = projectTypeCodes.web
export const defaultSkillCategoryCode = skillCategoryCodes.coding

export const ruleCategoryOptions = [
  { labelKey: 'catalog.ruleCategory.personal', value: ruleCategoryCodes.personal },
  { labelKey: 'catalog.ruleCategory.project', value: ruleCategoryCodes.project },
  { labelKey: 'catalog.ruleCategory.base', value: ruleCategoryCodes.base },
  { labelKey: 'catalog.ruleCategory.stack', value: ruleCategoryCodes.stack },
  { labelKey: 'catalog.ruleCategory.codeQuality', value: ruleCategoryCodes.codeQuality },
  { labelKey: 'catalog.ruleCategory.git', value: ruleCategoryCodes.git },
  { labelKey: 'catalog.ruleCategory.domain', value: ruleCategoryCodes.domain },
  { labelKey: 'catalog.ruleCategory.projectType', value: ruleCategoryCodes.projectType },
] as const satisfies readonly TaxonomyOption[]

export const skillCategoryOptions = [
  { labelKey: 'catalog.skillCategory.coding', value: skillCategoryCodes.coding },
  { labelKey: 'catalog.skillCategory.uiDesign', value: skillCategoryCodes.uiDesign },
] as const satisfies readonly TaxonomyOption[]

export const projectTypeOptions = [
  { labelKey: 'pages.projects.web', value: projectTypeCodes.web },
  { labelKey: 'pages.projects.mini', value: projectTypeCodes.mini },
  { labelKey: 'pages.projects.desktop', value: projectTypeCodes.desktop },
] as const satisfies readonly TaxonomyOption[]

export const projectImportModeOptions = [
  { labelKey: 'pages.projects.importMode.local', value: 'local' },
  { labelKey: 'pages.projects.importMode.git', value: 'git' },
] as const satisfies readonly TaxonomyOption<string>[]

export const repositoryConflictStrategyOptions = [
  { labelKey: 'catalog.conflict.skip', value: 'skip' },
  { labelKey: 'catalog.conflict.rename', value: 'rename' },
  { labelKey: 'catalog.conflict.overwrite', value: 'overwrite' },
] as const satisfies readonly TaxonomyOption<string>[]

export const presetReasoningOptions = [
  { label: 'none', value: 'none' },
  { label: 'low', value: 'low' },
  { label: 'medium', value: 'medium' },
  { label: 'high', value: 'high' },
  { label: 'xhigh', value: 'xhigh' },
] as const

export const taxonomySortOptions = [
  { labelKey: 'catalog.sort.order', value: 'order' },
  { labelKey: 'catalog.sort.name', value: 'name' },
  { labelKey: 'catalog.sort.code', value: 'code' },
] as const satisfies readonly TaxonomyOption<string>[]
