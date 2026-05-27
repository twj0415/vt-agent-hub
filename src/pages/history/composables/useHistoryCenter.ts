import { computed, onMounted, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { appRoutes } from '@/shared/config/routes'
import { useHistoryStore } from '@/shared/stores/history'

type HistoryTargetSource = {
  navigationTarget: string
  projectId?: number | null
  toolId?: number | null
  relatedRuleId?: number | null
}

export function useHistoryCenter() {
  const historyStore = useHistoryStore()
  const router = useRouter()
  const { t } = useI18n()
  const activeTab = ref<'records' | 'backups' | 'diagnostics'>('records')

  onMounted(() => {
    void historyStore.hydrateFromSnapshot()
  })

  const tabOptions = computed(() => [
    { key: 'records' as const, label: t('pages.history.tabs.records') },
    { key: 'backups' as const, label: t('pages.history.tabs.backups') },
    { key: 'diagnostics' as const, label: t('pages.history.tabs.diagnostics') },
  ])

  const summaryRows = computed(() => [
    { label: t('pages.history.summary.projects'), value: String(historyStore.filters.projectIds.length || 0), tone: 'ready' as const, badgeKey: 'common.ready' },
    { label: t('pages.history.summary.rules'), value: String(historyStore.items.filter((item) => item.relatedRuleId != null).length), tone: 'ready' as const, badgeKey: 'common.ready' },
    { label: t('pages.history.summary.skills'), value: String(historyStore.libraryDiagnostics?.skillCount ?? 0), tone: 'ready' as const, badgeKey: 'common.ready' },
    { label: t('pages.history.summary.issues'), value: String(historyStore.libraryDiagnostics?.issueCount ?? 0), tone: (historyStore.libraryDiagnostics?.issueCount ?? 0) > 0 ? 'warning' as const : 'ready' as const, badgeKey: (historyStore.libraryDiagnostics?.issueCount ?? 0) > 0 ? 'common.warning' : 'common.ready' },
  ])

  function openHistoryTarget(target: string) {
    if (!target) return
    void router.push(target)
  }

  function resolveHistoryTarget(item: HistoryTargetSource) {
    if (item.navigationTarget) return item.navigationTarget
    if (item.projectId) return appRoutes.projects
    if (item.relatedRuleId) return appRoutes.rules
    if (item.toolId) return appRoutes.settings
    return appRoutes.history
  }

  function relatedAssetLabel(item: Pick<HistoryTargetSource, 'navigationTarget' | 'relatedRuleId'>) {
    if (!item.relatedRuleId) return ''
    return resolveHistoryTarget(item).startsWith(appRoutes.skills)
      ? t('pages.history.buttons.skill', { id: item.relatedRuleId })
      : t('pages.history.buttons.rule', { id: item.relatedRuleId })
  }

  return {
    t,
    activeTab,
    historyStore,
    tabOptions,
    summaryRows,
    openHistoryTarget,
    resolveHistoryTarget,
    relatedAssetLabel,
  }
}
