import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { entityStateTone } from '@/shared/constants/status'
import { toolRegistry, type ToolRegistryItem } from '@/shared/tool-registry'
import { useSkillStore } from '@/shared/stores/skills'
import { useToolsStore } from '@/shared/stores/tools'
import { useToolRules } from './useToolRules'

function visibleRuleTags(names: string[]) {
  if (!names.length) return []
  const visible = names.slice(0, 4)
  return names.length > 4 ? [...visible, '...'] : visible
}

function joined(values: string[], empty: string) {
  return values.length ? values.join(' / ') : empty
}

function toolRootPath(liveConfigPath: string) {
  if (!liveConfigPath) return '-'
  return liveConfigPath.replace(/[\\/][^\\/]+$/, '') || liveConfigPath
}

export function useToolCards() {
  const { t } = useI18n()
  const skillStore = useSkillStore()
  const toolsStore = useToolsStore()
  const { boundRules, boundRuleNames } = useToolRules()

  const toolCards = computed(() =>
    toolRegistry.map((item: ToolRegistryItem) => {
      const isActive = item.id === toolsStore.activeId
      const ruleNames = isActive ? boundRules.value.map((rule) => rule.name) : []
      const skillNames = skillStore.items.filter((skill) => skill.toolIds.includes(item.id)).map((skill) => skill.name)
      const path = isActive ? toolRootPath(toolsStore.diagnostics.liveConfigPath) : '-'
      const version = isActive ? toolsStore.diagnostics.version || '-' : '-'
      return {
        ...item,
        name: t(item.nameKey),
        desc: t(item.descKey),
        isActive,
        path,
        version,
        ruleTags: visibleRuleTags(ruleNames),
        rulePreviewTitle: isActive ? boundRuleNames.value : t('pages.tools.binding.empty'),
        skillTags: visibleRuleTags(skillNames),
        skillCount: skillNames.length,
        skillPreviewTitle: joined(skillNames, t('pages.tools.binding.empty')),
        statusLabel: item.enabled ? t('common.ready') : t('common.disabled'),
        statusTone: item.enabled ? entityStateTone(item.status) : 'planned',
        statusNote: item.enabled ? t('pages.tools.binding.enabledNote') : t('pages.tools.binding.disabledNote'),
        canManageRules: item.enabled && item.capabilities.rules && item.capabilities.agentsOutput,
        canManageSkills: item.enabled && item.capabilities.skillInstall,
      }
    }),
  )

  const selectedToolCard = computed(() => toolCards.value.find((item) => item.id === toolsStore.activeId) ?? null)

  return {
    selectedToolCard,
    toolCards,
  }
}
