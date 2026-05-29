import { onMounted } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRuleStore } from '@/shared/stores/rules'
import { useSkillStore } from '@/shared/stores/skills'
import { useToolContextStore } from '@/shared/stores/tool-context'
import { useToolsStore } from '@/shared/stores/tools'
import type { ToolId } from '@/shared/tool-registry'
import { useToolCards } from './useToolCards'

export function useToolsWorkbench() {
  const { t } = useI18n()
  const toolsStore = useToolsStore()
  const workspaceStore = useToolContextStore()
  const ruleStore = useRuleStore()
  const skillStore = useSkillStore()
  const cards = useToolCards()

  function openTool(id: ToolId) {
    if (!selectTool(id)) return
    toolsStore.setDetailOpen(true)
  }

  function selectTool(id: ToolId) {
    if (!toolsStore.isToolEnabled(id)) return false
    toolsStore.select(id)
    workspaceStore.setActiveTool(id)
    return true
  }

  function manageRules(id: ToolId) {
    if (!selectTool(id)) return
    toolsStore.openRuleBinding()
  }

  function manageSkills(id: ToolId) {
    if (!selectTool(id)) return
    toolsStore.openSkillBinding()
  }

  onMounted(async () => {
    await Promise.all([
      toolsStore.hydrateFromSnapshot(),
      ruleStore.hydrateFromSnapshot(),
      skillStore.hydrateFromSnapshot(),
    ])
  })

  return {
    t,
    manageRules,
    manageSkills,
    openTool,
    ruleStore,
    skillStore,
    toolsStore,
    ...cards,
  }
}
