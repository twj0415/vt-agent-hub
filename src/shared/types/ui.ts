import type { Component } from 'vue'

export type BadgeTone = 'ready' | 'planned' | 'warning' | 'error' | 'info' | 'neutral' | 'active'

export type CardMoreMenuItem = {
  key: string
  label: string
  icon?: Component
  disabled?: boolean
  danger?: boolean
}

export type FormFieldType = 'text' | 'password' | 'select' | 'textarea'

export type FormFieldOption<TValue extends number | string = string> = {
  label?: string
  labelKey?: string
  value: TValue
}

export type FormField = {
  key: string
  label?: string
  labelKey?: string
  value?: string
  placeholder?: string
  placeholderKey?: string
  help?: string
  helpKey?: string
  type?: FormFieldType
  options?: readonly FormFieldOption[]
  rows?: number
  disabled?: boolean
  group?: string
  groupKey?: string
}

export type MatrixRow = {
  label: string
  value: string
  tone?: BadgeTone
  badgeLabel?: string
  badgeKey?: string
}

export type PresetSchemaField = {
  key: string
  type: FormFieldType
  options?: readonly FormFieldOption[]
  labelKey: string
  placeholderKey?: string
  helpKey?: string
  groupKey: string
  defaultValue: string
  rows?: number
}

export type PresetSchema = {
  toolId: number
  schemaVersion: number
  fields: PresetSchemaField[]
}
