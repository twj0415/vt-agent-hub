import { useRuleStore } from '@/shared/stores/rules'
import { repositoryConflictStrategyOptions } from '@/shared/taxonomy'
import { localizeMessage } from '@/shared/utils/message'
import { notifyError } from '@/shared/utils/notify'
import { useRuleForm } from './useRuleForm'

export type RuleImportField = 'sourcePath' | 'name' | 'summary' | 'categoryCode' | 'conflictStrategy'

export function useRuleImport() {
  const ruleStore = useRuleStore()
  const { categoryOptions } = useRuleForm()
  // 冲突策略字典：导入同名规则时决定跳过、重命名或覆盖。
  const conflictOptions = repositoryConflictStrategyOptions

  function setImportField(key: RuleImportField, value: string | number | null) {
    ruleStore.setImportField(key, value)
  }

  function handleFilePickerError(message: string) {
    ruleStore.actionError = localizeMessage(message || 'Rule file picker failed.')
    notifyError(ruleStore.actionError)
  }

  return {
    categoryOptions,
    conflictOptions,
    handleFilePickerError,
    ruleStore,
    setImportField,
  }
}
