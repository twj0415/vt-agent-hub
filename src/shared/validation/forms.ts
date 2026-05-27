import { z } from 'zod'
import { providerReasoningOptions } from '@/shared/providers'
import {
  projectTypeOptions,
  ruleCategoryOptions,
  skillCategoryOptions,
} from '@/shared/taxonomy'

const projectTypeCodes = projectTypeOptions.map((item) => item.value)
const ruleCategoryCodes = ruleCategoryOptions.map((item) => item.value)
const skillCategoryCodes = skillCategoryOptions.map((item) => item.value)
const presetReasoningValues = providerReasoningOptions.map((item) => item.value)

export const projectDraftSchema = z.object({
  name: z.string().trim().optional(),
  path: z.string().trim().min(1, 'Project path is required.'),
  projectType: z.number().int().refine((value) => projectTypeCodes.includes(value as never), 'Project type is not supported.'),
})

export const ruleDraftSchema = z.object({
  name: z.string().trim().min(1, 'errors.ruleNameRequired'),
  code: z.number().int().refine((value) => ruleCategoryCodes.includes(value as never), 'errors.ruleCodeUnsupported'),
  categoryCode: z.number().int().refine((value) => ruleCategoryCodes.includes(value as never), 'errors.ruleCategoryUnsupported'),
  body: z.string().trim().min(1, 'errors.ruleBodyRequired'),
})

export const skillDraftSchema = z.object({
  name: z.string().trim().min(1, 'Skill name is required.'),
  code: z.number().int().refine((value) => skillCategoryCodes.includes(value as never), 'Skill code is not supported.'),
  categoryCode: z.number().int().refine((value) => skillCategoryCodes.includes(value as never), 'Skill category is not supported.'),
  body: z.string().trim().min(1, 'Skill body is required.'),
})

export const presetDraftSchema = z.object({
  name: z.string().trim().min(1, 'Preset name is required.'),
  provider: z.string().trim().min(1, 'Provider is required.'),
  model: z.string().trim().min(1, 'Model is required.'),
  reasoning: z.string().refine((value) => presetReasoningValues.includes(value as never), 'Reasoning is not supported.'),
  baseUrl: z.string().trim().url('Base URL must be a valid URL.'),
})

export const credentialSchema = z.object({
  token: z.string().trim().min(8, 'Credential token is too short.'),
})

export function firstIssue(error: unknown, fallback: string) {
  if (error instanceof z.ZodError) {
    return error.issues[0]?.message ?? fallback
  }
  return fallback
}
