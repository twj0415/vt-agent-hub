import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useTaxonomyOptions } from '@/shared/composables/useTaxonomyOptions'
import { skillHealthState, useSkillStore, type SkillItem } from '@/shared/stores/skills'
import { getToolById } from '@/shared/tool-registry'
import type { BadgeTone } from '@/shared/types/ui'

function previewTags(values: string[]) {
  if (!values.length) return []
  const visible = values.slice(0, 4)
  return values.length > 4 ? [...visible, '...'] : visible
}

function joined(values: string[], empty: string) {
  return values.length ? values.join(' / ') : empty
}

function text(value: string, emptyText: string) {
  return value.trim() || emptyText
}

export function useSkillCards() {
  const { t } = useI18n()
  const skillStore = useSkillStore()
  const taxonomyOptions = useTaxonomyOptions()
  const currentSkills = computed(() => skillStore.filteredItems)

  function categoryLabel(categoryCode: number) {
    return taxonomyOptions.skillCategories.labelOf(categoryCode, t('common.empty'))
  }

  function toolNames(item: Pick<SkillItem, 'toolIds'>) {
    return item.toolIds.map((id) => {
      const tool = getToolById(id)
      return tool ? t(tool.nameKey) : `#${id}`
    })
  }

  function statusLabel(item: Pick<SkillItem, 'runtime'>) {
    return skillHealthState(item) === 'normal' ? t('common.normal') : t('common.abnormal')
  }

  function statusTone(item: Pick<SkillItem, 'runtime'>): Extract<BadgeTone, 'ready' | 'error'> {
    return skillHealthState(item) === 'normal' ? 'ready' : 'error'
  }

  const skillCards = computed(() =>
    currentSkills.value.map((item) => {
      const emptyText = t('common.empty')
      const tools = toolNames(item)

      return {
        ...item,
        categoryLabel: categoryLabel(item.categoryCode),
        versionLabel: `v${item.versionNo}`,
        summaryText: text(item.summary, emptyText),
        toolTags: previewTags(tools),
        toolCount: item.toolIds.length,
        toolTitle: joined(tools, emptyText),
        statusLabel: statusLabel(item),
        statusTone: statusTone(item),
      }
    }),
  )

  return {
    categoryLabel,
    currentSkills,
    skillCards,
    statusLabel,
    statusTone,
    toolNames,
  }
}
