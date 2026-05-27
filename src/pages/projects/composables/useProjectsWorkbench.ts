import { onMounted, watch } from 'vue'
import { useToolsStore } from '@/shared/stores/tools'
import { useToolContextStore } from '@/shared/stores/tool-context'
import { useProjectUi } from './useProjectUi'

export function useProjectsWorkbench() {
  const ui = useProjectUi()
  const toolsStore = useToolsStore()
  const workspaceStore = useToolContextStore()
  const { projectsStore, toolProjects, projectCards } = ui

  function syncProjectSelection() {
    if (!projectsStore.activeId && toolProjects.value[0]) {
      projectsStore.select(toolProjects.value[0].id)
    }
    if (projectsStore.activeId) {
      workspaceStore.setActiveProject(projectsStore.activeId)
    }
  }

  async function scanSelectedProject() {
    if (!projectsStore.activeId) return
    await projectsStore.scanOutput(workspaceStore.activeToolId, { silent: true })
  }

  function openProject(id: number) {
    projectsStore.select(id)
    projectsStore.setDetailOpen(true)
  }

  async function previewProject(id: number) {
    projectsStore.select(id)
    workspaceStore.setActiveProject(id)
    await projectsStore.openOutputPreview(workspaceStore.activeToolId, 'preview')
  }

  async function repairProject(id: number) {
    projectsStore.select(id)
    workspaceStore.setActiveProject(id)
    await projectsStore.openOutputPreview(workspaceStore.activeToolId, 'repair')
  }

  function manageRules(id: number) {
    const card = projectCards.value.find((item) => item.id === id)
    if (card && !card.canManageRules) return
    projectsStore.select(id)
    projectsStore.openRuleBinding()
  }

  watch(
    () => projectsStore.activeId,
    async () => {
      syncProjectSelection()
      await scanSelectedProject()
    },
    { immediate: true },
  )

  watch(
    () => workspaceStore.activeToolId,
    async () => {
      await toolsStore.hydrateDiagnostics()
      await scanSelectedProject()
    },
  )

  onMounted(syncProjectSelection)

  return {
    ...ui,
    manageRules,
    openProject,
    previewProject,
    repairProject,
  }
}
