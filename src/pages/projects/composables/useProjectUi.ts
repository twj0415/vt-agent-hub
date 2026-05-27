import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useProjectsStore } from '@/shared/stores/projects'
import { useToolContextStore } from '@/shared/stores/tool-context'
import { getToolById } from '@/shared/tool-registry'
import { useProjectCards } from './useProjectCards'
import { useProjectForm } from './useProjectForm'
import { useProjectRules } from './useProjectRules'

export function useProjectUi() {
  const { t } = useI18n()
  const projectsStore = useProjectsStore()
  const workspaceStore = useToolContextStore()

  return {
    t,
    activeToolName: computed(() => t(getToolById(workspaceStore.activeToolId)?.nameKey ?? 'common.codex')),
    projectsStore,
    workspaceStore,
    ...useProjectCards(),
    ...useProjectRules(),
    ...useProjectForm(),
  }
}
