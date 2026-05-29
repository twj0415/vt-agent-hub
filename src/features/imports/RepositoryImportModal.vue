<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import type { FormInstance, Rule } from 'ant-design-vue/es/form'
import { useI18n } from 'vue-i18n'
import type {
  GitHubRepoImportResult,
  GitHubRepoPreview,
  GitHubSkillImportSelection,
  RepositoryImportReport,
} from '@/shared/api/client'
import StatusBadge from '@/shared/components/feedback/StatusBadge.vue'
import VTModal from '@/shared/components/feedback/VTModal.vue'

type GitHubSelectionState = GitHubSkillImportSelection & { selected: boolean }
type GitHubImportStep = 'input' | 'preview' | 'result'

const props = defineProps<{
  open: boolean
  draft: {
    source: string
    branch: string
    conflictStrategy: 'skip' | 'rename' | 'overwrite'
  }
  loading: boolean
  report: RepositoryImportReport | null
  githubPreview: GitHubRepoPreview | null
  githubImportResult: GitHubRepoImportResult | null
  githubSelections: Record<string, GitHubSelectionState>
  githubSelectedPath: string
  githubStep: GitHubImportStep
}>()

const emit = defineEmits<{
  close: []
  preview: []
  apply: []
  field: [{ key: 'source' | 'branch' | 'conflictStrategy'; value: string }]
  toggleGithubSkill: [sourcePath: string]
  githubResolution: [{ sourcePath: string; resolution: 'skip' | 'overwrite' | 'rename' }]
  githubRename: [{ sourcePath: string; renamedSkillId: string }]
  githubSelectedPath: [sourcePath: string]
}>()

const { t } = useI18n()
const formRef = ref<FormInstance>()
const formRules = computed<Record<string, Rule[]>>(() => ({
  source: [{ required: true, whitespace: true, message: t('errors.repositorySourceRequired'), trigger: ['blur', 'change'] }],
}))
const selectedCount = computed(() => Object.values(props.githubSelections).filter((selection) => selection.selected).length)
const activeSkill = computed(() => props.githubPreview?.skills.find((skill) => skill.sourcePath === props.githubSelectedPath) ?? props.githubPreview?.skills[0] ?? null)
const activeSelection = computed(() => activeSkill.value ? props.githubSelections[activeSkill.value.sourcePath] : null)

watch(
  () => props.open,
  async (open) => {
    if (!open) return
    await nextTick()
    formRef.value?.clearValidate()
  },
)

async function submitPreview() {
  try {
    await formRef.value?.validate()
  } catch {
    return
  }
  emit('preview')
}

async function submitApply() {
  if (props.githubStep === 'input') {
    await submitPreview()
    return
  }
  emit('apply')
}
</script>

<template>
  <VTModal
    :open="open"
    :title="t('catalog.repositoryImportTitle')"
    :ok-text="githubStep === 'input' ? t('catalog.repositoryPreview') : t('catalog.repositoryApply')"
    :cancel-text="t('common.close')"
    :loading="loading"
    :width="980"
    @ok="submitApply"
    @close="emit('close')"
  >
    <a-form ref="formRef" layout="vertical" :model="draft" :rules="formRules">
      <div class="rounded-2xl border border-line bg-panel-strong/70 p-4 shadow-soft">
        <p class="m-0 text-sm leading-6 text-muted">{{ t('catalog.repositoryImportDesc') }}</p>
        <a-form-item name="source" :label="t('catalog.repositorySource')" class="!mb-0 mt-4">
          <a-input
            :value="draft.source"
            :placeholder="t('catalog.repositorySourcePlaceholder')"
            @update:value="emit('field', { key: 'source', value: String($event) })"
          />
        </a-form-item>
        <div class="mt-3 flex flex-wrap items-center gap-2">
          <a-button :loading="loading" @click="submitPreview">{{ t('catalog.repositoryPreview') }}</a-button>
          <StatusBadge v-if="githubPreview" tone="planned" :label="`${t('catalog.repositoryDetected')}: ${githubPreview.skills.length}`" />
          <StatusBadge v-if="githubPreview" tone="ready" :label="`Selected: ${selectedCount}`" />
        </div>
      </div>

      <div v-if="githubStep === 'preview' && githubPreview" class="mt-4 grid gap-4 lg:grid-cols-[320px_minmax(0,1fr)]">
        <div class="space-y-2">
          <button
            v-for="skill in githubPreview.skills"
            :key="skill.sourcePath"
            type="button"
            class="w-full rounded-2xl border p-3 text-left transition duration-fast hover:border-accent/50 hover:bg-panel-strong"
            :class="skill.sourcePath === (activeSkill?.sourcePath ?? '') ? 'border-accent bg-panel-strong shadow-soft' : 'border-line bg-panel'"
            @click="emit('githubSelectedPath', skill.sourcePath)"
          >
            <div class="flex items-start gap-3">
              <a-checkbox
                :checked="githubSelections[skill.sourcePath]?.selected"
                @click.stop
                @change="emit('toggleGithubSkill', skill.sourcePath)"
              />
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2">
                  <span class="truncate text-sm font-semibold text-text">{{ skill.skillName }}</span>
                  <StatusBadge v-if="skill.conflict" tone="warning" label="Conflict" />
                </div>
                <p class="m-0 mt-1 truncate text-xs text-muted">{{ skill.sourcePath }}</p>
              </div>
            </div>
          </button>
        </div>

        <div v-if="activeSkill && activeSelection" class="rounded-2xl border border-line bg-panel p-5 shadow-soft">
          <div class="flex flex-wrap items-start justify-between gap-3">
            <div>
              <p class="m-0 text-xs uppercase tracking-[0.18em] text-muted">GitHub Skill</p>
              <h3 class="m-0 mt-2 text-xl font-semibold text-text">{{ activeSkill.skillName }}</h3>
              <p class="m-0 mt-2 text-sm leading-6 text-muted">{{ activeSkill.description || activeSkill.sourcePath }}</p>
            </div>
            <StatusBadge :tone="activeSkill.conflict ? 'warning' : 'ready'" :label="activeSkill.conflict ? 'Conflict' : 'Ready'" />
          </div>

          <div class="mt-5 grid gap-3 rounded-2xl bg-bg/60 p-4 text-sm">
            <div class="flex items-center justify-between gap-4">
              <span class="text-muted">Source path</span>
              <span class="font-mono text-text">{{ activeSkill.sourcePath }}</span>
            </div>
            <div class="flex items-center justify-between gap-4">
              <span class="text-muted">Target skill id</span>
              <span class="font-mono text-text">{{ activeSelection.renamedSkillId || activeSkill.skillId }}</span>
            </div>
            <div v-if="activeSkill.conflict" class="flex items-center justify-between gap-4">
              <span class="text-muted">Existing skill</span>
              <span class="font-mono text-warning">{{ activeSkill.conflict.existingName }}</span>
            </div>
          </div>

          <div class="mt-5 grid gap-3">
            <a-radio-group
              :value="activeSelection.resolution"
              @update:value="emit('githubResolution', { sourcePath: activeSkill.sourcePath, resolution: $event as 'skip' | 'overwrite' | 'rename' })"
            >
              <a-radio-button value="skip">Skip</a-radio-button>
              <a-radio-button value="overwrite">Overwrite</a-radio-button>
              <a-radio-button value="rename">Rename</a-radio-button>
            </a-radio-group>

            <a-input
              v-if="activeSelection.resolution === 'rename'"
              :value="activeSelection.renamedSkillId"
              placeholder="new-skill-id"
              @update:value="emit('githubRename', { sourcePath: activeSkill.sourcePath, renamedSkillId: String($event) })"
            />
          </div>
        </div>
      </div>

      <div v-if="githubStep === 'result' && githubImportResult" class="mt-4 rounded-2xl border border-line bg-panel p-5 shadow-soft">
        <div class="flex items-center gap-2">
          <StatusBadge tone="ready" :label="`Imported: ${githubImportResult.importedSkills.length}`" />
          <StatusBadge tone="planned" :label="`Skipped: ${githubImportResult.skippedSkills.length}`" />
        </div>
        <div class="mt-4 grid gap-2">
          <div
            v-for="skill in githubImportResult.importedSkills"
            :key="`${skill.sourcePath}-${skill.assetId}`"
            class="rounded-xl bg-bg/70 px-4 py-3"
          >
            <div class="flex items-center justify-between gap-4">
              <span class="font-medium text-text">{{ skill.skillName }}</span>
              <span class="text-xs uppercase tracking-[0.16em] text-muted">{{ skill.operation }}</span>
            </div>
            <p class="m-0 mt-1 font-mono text-xs text-muted">{{ skill.sourcePath }}</p>
          </div>
        </div>
      </div>
    </a-form>
  </VTModal>
</template>
