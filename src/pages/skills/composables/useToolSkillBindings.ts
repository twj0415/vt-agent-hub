import { computed } from 'vue'
import { useTaxonomyOptions } from '@/shared/composables/useTaxonomyOptions'
import { skillHealthState, useSkillStore } from '@/shared/stores/skills'
import { useToolsStore } from '@/shared/stores/tools'
import { markdownDescription } from '@/shared/utils/markdown'

export function useToolSkillBindings() {
  const skillStore = useSkillStore()
  const toolsStore = useToolsStore()
  const taxonomyOptions = useTaxonomyOptions()

  const boundSkillIds = computed(() => new Set(
    skillStore.items
      .filter((item) => item.toolIds.includes(toolsStore.activeId))
      .map((item) => item.id),
  ))

  const bindableSkills = computed(() => skillStore.items)
  const availableSkills = computed(() => skillStore.items.filter((item) => !boundSkillIds.value.has(item.id)))

  function categoryLabel(categoryCode: number) {
    return taxonomyOptions.skillCategories.labelOf(categoryCode, String(categoryCode))
  }

  function skillDescription(skill: { summary: string; body: string }) {
    const summary = skill.summary.trim()
    const value = summary && !summary.toLowerCase().startsWith('imported from') ? summary : markdownDescription(skill.body)
    return value || '-'
  }

  const boundSkills = computed(() =>
    skillStore.items
      .filter((skill) => skill.toolIds.includes(toolsStore.activeId))
      .map((skill) => ({
        id: skill.id,
        name: skill.name,
        description: skillDescription(skill),
        categoryLabel: categoryLabel(skill.categoryCode),
        versionNo: skill.versionNo || null,
        versionId: skill.versionId,
        healthState: skillHealthState(skill),
      })),
  )

  return {
    availableSkills,
    bindableSkills,
    boundSkills,
    categoryLabel,
    skillDescription,
  }
}
