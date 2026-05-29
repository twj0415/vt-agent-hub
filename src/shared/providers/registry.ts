import { toolIds, toolRegistry, type ToolId } from '@/shared/tool-registry'
import type { FormFieldOption, PresetSchema } from '@/shared/types/ui'

export type ProviderCategory = 'official' | 'aggregator' | 'custom_gateway' | 'local'
export type ProviderReasoning = 'none' | 'low' | 'medium' | 'high' | 'xhigh'

export type ProviderToolSchema = PresetSchema & {
  toolId: ToolId
}

export type ProviderImportPartSchema = {
  role: string
  labelKey: string
  placeholderKey: string
  helpKey?: string
  required: boolean
  rows: number
}

export const providerCategoryOptions = [
  { labelKey: 'pages.providers.providerCategory.official', value: 'official' },
  { labelKey: 'pages.providers.providerCategory.aggregator', value: 'aggregator' },
  { labelKey: 'pages.providers.providerCategory.customGateway', value: 'custom_gateway' },
  { labelKey: 'pages.providers.providerCategory.local', value: 'local' },
] as const satisfies readonly FormFieldOption<ProviderCategory>[]

export const defaultProviderCategory = providerCategoryOptions[0].value

export const providerReasoningOptions = [
  { labelKey: 'pages.providers.reasoning.none', value: 'none' },
  { labelKey: 'pages.providers.reasoning.low', value: 'low' },
  { labelKey: 'pages.providers.reasoning.medium', value: 'medium' },
  { labelKey: 'pages.providers.reasoning.high', value: 'high' },
  { labelKey: 'pages.providers.reasoning.xhigh', value: 'xhigh' },
] as const satisfies readonly FormFieldOption<ProviderReasoning>[]

export const codexModelOptions = [
  { label: 'GPT-5.5', value: 'gpt-5.5' },
  { label: 'GPT-5.4', value: 'gpt-5.4' },
  { label: 'GPT-5.4 mini', value: 'gpt-5.4-mini' },
  { label: 'GPT-5.4 nano', value: 'gpt-5.4-nano' },
] as const satisfies readonly FormFieldOption<string>[]

export const codexProviderSchema: ProviderToolSchema = {
  toolId: toolIds.codex,
  schemaVersion: 1,
  fields: [
    {
      key: 'displayName',
      type: 'text',
      labelKey: 'pages.providers.displayName',
      placeholderKey: 'pages.providers.displayNamePlaceholder',
      groupKey: 'pages.providers.form.group.meta',
      defaultValue: '',
    },
    {
      key: 'model',
      type: 'select',
      labelKey: 'pages.providers.form.modelLabel',
      helpKey: 'pages.providers.form.help.model',
      groupKey: 'pages.providers.form.group.runtime',
      defaultValue: 'gpt-5.5',
      options: codexModelOptions,
    },
    {
      key: 'reasoning',
      type: 'select',
      labelKey: 'pages.providers.form.reasoningLabel',
      helpKey: 'pages.providers.form.help.reasoning',
      groupKey: 'pages.providers.form.group.runtime',
      defaultValue: 'medium',
      options: providerReasoningOptions,
    },
    {
      key: 'baseUrl',
      type: 'text',
      labelKey: 'pages.providers.form.baseUrlLabel',
      groupKey: 'pages.providers.form.group.runtime',
      defaultValue: 'https://api.openai.com/v1',
    },
  ],
}

export const claudeModelOptions = [
  { label: 'Claude Opus 4.7', value: 'claude-opus-4-7' },
  { label: 'Claude Sonnet 4.6', value: 'claude-sonnet-4-6' },
  { label: 'Claude Haiku 4.5', value: 'claude-haiku-4-5-20251001' },
] as const satisfies readonly FormFieldOption<string>[]

export const claudeProviderSchema: ProviderToolSchema = {
  toolId: toolIds.claude,
  schemaVersion: 1,
  fields: [
    {
      key: 'displayName',
      type: 'text',
      labelKey: 'pages.providers.form.displayNameLabel',
      placeholderKey: 'pages.providers.displayNamePlaceholder',
      groupKey: 'pages.providers.form.group.meta',
      defaultValue: '',
    },
    {
      key: 'model',
      type: 'select',
      labelKey: 'pages.providers.form.modelLabel',
      helpKey: 'pages.providers.form.help.model',
      groupKey: 'pages.providers.form.group.runtime',
      defaultValue: 'claude-opus-4-7',
      options: claudeModelOptions,
    },
    {
      key: 'reasoning',
      type: 'select',
      labelKey: 'pages.providers.form.reasoningLabel',
      helpKey: 'pages.providers.form.help.reasoning',
      groupKey: 'pages.providers.form.group.runtime',
      defaultValue: 'medium',
      options: providerReasoningOptions,
    },
    {
      key: 'baseUrl',
      type: 'text',
      labelKey: 'pages.providers.form.baseUrlLabel',
      groupKey: 'pages.providers.form.group.runtime',
      defaultValue: 'https://api.anthropic.com',
    },
  ],
}

export const codexProviderImportParts: ProviderImportPartSchema[] = [
  {
    role: 'config',
    labelKey: 'pages.providers.importParts.configToml',
    placeholderKey: 'pages.providers.importParts.configTomlPlaceholder',
    helpKey: 'pages.providers.importParts.configTomlHelp',
    required: true,
    rows: 14,
  },
  {
    role: 'auth',
    labelKey: 'pages.providers.importParts.authJson',
    placeholderKey: 'pages.providers.importParts.authJsonPlaceholder',
    helpKey: 'pages.providers.importParts.authJsonHelp',
    required: false,
    rows: 8,
  },
]

export const claudeProviderImportParts: ProviderImportPartSchema[] = [
  {
    role: 'config',
    labelKey: 'pages.providers.importParts.settingsJson',
    placeholderKey: 'pages.providers.importParts.settingsJsonPlaceholder',
    helpKey: 'pages.providers.importParts.settingsJsonHelp',
    required: true,
    rows: 14,
  },
]

export const providerToolOptions = toolRegistry
  .filter((tool) => tool.enabled && tool.capabilities.presets)
  .map((tool) => ({ labelKey: tool.nameKey, value: tool.id }))

export function getProviderToolSchema(toolId: number, schemaVersion = 1) {
  if (toolId === toolIds.codex && schemaVersion === 1) return codexProviderSchema
  if (toolId === toolIds.claude && schemaVersion === 1) return claudeProviderSchema
  return null
}

export function getProviderImportParts(toolId: number) {
  if (toolId === toolIds.codex) return codexProviderImportParts
  if (toolId === toolIds.claude) return claudeProviderImportParts
  return []
}

export function listProviderModelOptions(toolId: number) {
  if (toolId === toolIds.codex) return codexModelOptions
  if (toolId === toolIds.claude) return claudeProviderSchema.fields.find((field) => field.key === 'model')?.options ?? []
  return []
}

export function isSupportedProviderCategory(value: string): value is ProviderCategory {
  return providerCategoryOptions.some((option) => option.value === value)
}

export function isSupportedProviderReasoning(value: string): value is ProviderReasoning {
  return providerReasoningOptions.some((option) => option.value === value)
}

export function isSupportedProviderModel(toolId: number, value: string) {
  if (toolId === toolIds.claude) return value.trim().length > 0
  return listProviderModelOptions(toolId).some((option) => option.value === value)
}
