import { i18n } from './index'
import type { AppError } from '@/shared/api/client'

type TranslateParams = Record<string, string | number>

function hasKey(key?: string) {
  return Boolean(key) && i18n.global.te(key as string)
}

export function translateKey(key: string, params?: TranslateParams) {
  return i18n.global.t(key, params ?? {})
}

export function translateIfExists(key?: string, fallback = '', params?: TranslateParams) {
  if (!hasKey(key)) return fallback
  return i18n.global.t(key as string, params ?? {})
}

export function resolveAppError(error: AppError | undefined, fallbackKey: string, params?: TranslateParams) {
  if (hasKey(error?.i18nKey)) {
    return i18n.global.t(error?.i18nKey as string, params ?? {})
  }

  if (hasKey(fallbackKey)) {
    return i18n.global.t(fallbackKey, params ?? {})
  }

  return error?.message ?? fallbackKey
}

export function resolveUnknownError(error: unknown, fallbackKey: string, params?: TranslateParams) {
  if (error instanceof Error && hasKey(error.message)) {
    return i18n.global.t(error.message, params ?? {})
  }

  if (hasKey(fallbackKey)) {
    return i18n.global.t(fallbackKey, params ?? {})
  }

  if (error instanceof Error && error.message.trim()) {
    return error.message
  }

  return fallbackKey
}
