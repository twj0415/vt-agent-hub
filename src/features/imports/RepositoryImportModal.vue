<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import type { FormInstance, Rule } from 'ant-design-vue/es/form'
import { FolderOpenOutlined } from '@ant-design/icons-vue'
import { useI18n } from 'vue-i18n'
import type {
  GitHubRepoImportResult,
  GitHubRepoPreview,
  GitHubSkillImportSelection,
  GitHubSkillPreview,
  LocalSkillsImportResult,
  LocalSkillsPreview,
  RepositoryImportReport,
} from '@/shared/api/client'
import StatusBadge from '@/shared/components/feedback/StatusBadge.vue'
import VTModal from '@/shared/components/feedback/VTModal.vue'

type SelectionState = GitHubSkillImportSelection & { selected: boolean }
type ImportStep = 'input' | 'preview' | 'result'
type SourceKind = 'local' | 'github'

const props = defineProps<{
  open: boolean
  draft: {
    source: string
    branch: string
    conflictStrategy: 'skip' | 'rename' | 'overwrite'
    sourceKind: SourceKind
    localPath: string
  }
  loading: boolean
  report: RepositoryImportReport | null
  githubPreview: GitHubRepoPreview | null
  githubImportResult: GitHubRepoImportResult | null
  githubSelections: Record<string, SelectionState>
  githubSelectedPath: string
  githubStep: ImportStep
  localPreview: LocalSkillsPreview | null
  localImportResult: LocalSkillsImportResult | null
  localSelections: Record<string, SelectionState>
  localSelectedPath: string
  localStep: ImportStep
}>()

const emit = defineEmits<{
  close: []
  preview: []
  apply: []
  field: [{ key: 'source' | 'branch' | 'conflictStrategy'; value: string }]
  sourceKind: [kind: SourceKind]
  pickLocal: []
  toggleGithubSkill: [sourcePath: string]
  githubResolution: [{ sourcePath: string; resolution: 'skip' | 'overwrite' | 'rename' }]
  githubRename: [{ sourcePath: string; renamedSkillId: string }]
  githubSelectedPath: [sourcePath: string]
  toggleLocalSkill: [sourcePath: string]
  localResolution: [{ sourcePath: string; resolution: 'skip' | 'overwrite' | 'rename' }]
  localRename: [{ sourcePath: string; renamedSkillId: string }]
  localSelectedPath: [sourcePath: string]
}>()

const { t } = useI18n()
const formRef = ref<FormInstance>()
const formRules = computed<Record<string, Rule[]>>(() => ({
  source: [{ required: true, whitespace: true, message: t('errors.repositorySourceRequired'), trigger: ['blur', 'change'] }],
}))

const isLocal = computed(() => props.draft.sourceKind === 'local')
const activeStep = computed<ImportStep>(() => (isLocal.value ? props.localStep : props.githubStep))
const activeSkills = computed<GitHubSkillPreview[]>(() => (isLocal.value ? props.localPreview?.skills ?? [] : props.githubPreview?.skills ?? []))
const activeSelections = computed(() => (isLocal.value ? props.localSelections : props.githubSelections))
const activeSelectedPath = computed(() => (isLocal.value ? props.localSelectedPath : props.githubSelectedPath))
const activeImportResult = computed(() => (isLocal.value ? props.localImportResult : props.githubImportResult))
const activeImportedSkills = computed(() => activeImportResult.value?.importedSkills ?? [])
const activeSkippedSkills = computed(() => activeImportResult.value?.skippedSkills ?? [])
const selectedCount = computed(() => Object.values(activeSelections.value).filter((selection) => selection.selected).length)
const activeSkill = computed<GitHubSkillPreview | null>(() => {
  const list = activeSkills.value
  return list.find((skill) => skill.sourcePath === activeSelectedPath.value) ?? list[0] ?? null
})
const activeSelection = computed(() => (activeSkill.value ? activeSelections.value[activeSkill.value.sourcePath] : null))

const okText = computed(() => {
  if (activeStep.value === 'result') return t('common.close')
  if (activeStep.value === 'preview') return t('catalog.repositoryApply')
  return t('catalog.repositoryPreview')
})

watch(
  () => props.open,
  async (open) => {
    if (!open) return
    await nextTick()
    formRef.value?.clearValidate()
  },
)

watch(
  () => props.draft.sourceKind,
  async () => {
    await nextTick()
    formRef.value?.clearValidate()
  },
)

function handleSelectPath(sourcePath: string) {
  if (isLocal.value) emit('localSelectedPath', sourcePath)
  else emit('githubSelectedPath', sourcePath)
}

function handleToggleSkill(sourcePath: string) {
  if (isLocal.value) emit('toggleLocalSkill', sourcePath)
  else emit('toggleGithubSkill', sourcePath)
}

function handleResolution(sourcePath: string, resolution: 'skip' | 'overwrite' | 'rename') {
  if (isLocal.value) emit('localResolution', { sourcePath, resolution })
  else emit('githubResolution', { sourcePath, resolution })
}

function handleRename(sourcePath: string, renamedSkillId: string) {
  if (isLocal.value) emit('localRename', { sourcePath, renamedSkillId })
  else emit('githubRename', { sourcePath, renamedSkillId })
}

async function submitPreview() {
  if (isLocal.value) {
    emit('preview')
    return
  }
  try {
    await formRef.value?.validate()
  } catch {
    return
  }
  emit('preview')
}

async function submitApply() {
  if (activeStep.value === 'result') {
    emit('close')
    return
  }
  if (activeStep.value === 'input') {
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
    :ok-text="okText"
    :cancel-text="t('common.close')"
    :loading="loading"
    :width="980"
    @ok="submitApply"
    @close="emit('close')"
  >
    <a-form ref="formRef" layout="vertical" :model="draft" :rules="isLocal ? {} : formRules">
      <div class="rounded-2xl border border-line bg-panel-strong/70 p-4 shadow-soft">
        <a-radio-group
          :value="draft.sourceKind"
          button-style="solid"
          @change="emit('sourceKind', ($event.target.value as SourceKind))"
        >
          <a-radio-button value="local">{{ t('catalog.repositorySourceKindLocal') }}</a-radio-button>
          <a-radio-button value="github">{{ t('catalog.repositorySourceKindGitHub') }}</a-radio-button>
        </a-radio-group>

        <p class="m-0 mt-3 text-sm leading-6 text-muted">
          {{ isLocal ? t('catalog.repositoryLocalDescription') : t('catalog.repositoryImportDesc') }}
        </p>

        <template v-if="isLocal">
          <div class="mt-4 flex flex-wrap items-center gap-3">
            <a-button :loading="loading" @click="emit('pickLocal')">
              <template #icon><FolderOpenOutlined /></template>
              {{ t('catalog.repositorySelectLocalDir') }}
            </a-button>
            <span class="truncate font-mono text-xs text-muted">
              {{ draft.localPath || t('catalog.repositoryLocalDirEmpty') }}
            </span>
            <StatusBadge v-if="localPreview" tone="planned" :label="`${t('catalog.repositoryDetected')}: ${localPreview.skills.length}`" />
            <StatusBadge v-if="localPreview" tone="ready" :label="`Selected: ${selectedCount}`" />
          </div>
        </template>

        <template v-else>
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
        </template>
      </div>

      <div v-if="activeStep === 'preview' && activeSkills.length" class="mt-4 grid gap-4 lg:grid-cols-[320px_minmax(0,1fr)]">
        <div class="space-y-2">
          <button
            v-for="skill in activeSkills"
            :key="skill.sourcePath"
            type="button"
            class="w-full rounded-2xl border p-3 text-left transition duration-fast hover:border-accent/50 hover:bg-panel-strong"
            :class="skill.sourcePath === (activeSkill?.sourcePath ?? '') ? 'border-accent bg-panel-strong shadow-soft' : 'border-line bg-panel'"
            @click="handleSelectPath(skill.sourcePath)"
          >
            <div class="flex items-start gap-3">
              <a-checkbox
                :checked="activeSelections[skill.sourcePath]?.selected"
                @click.stop
                @change="handleToggleSkill(skill.sourcePath)"
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
              <p class="m-0 text-xs uppercase tracking-[0.18em] text-muted">{{ isLocal ? 'Local Skill' : 'GitHub Skill' }}</p>
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
              @update:value="handleResolution(activeSkill.sourcePath, $event as 'skip' | 'overwrite' | 'rename')"
            >
              <a-radio-button value="skip">Skip</a-radio-button>
              <a-radio-button value="overwrite">Overwrite</a-radio-button>
              <a-radio-button value="rename">Rename</a-radio-button>
            </a-radio-group>

            <a-input
              v-if="activeSelection.resolution === 'rename'"
              :value="activeSelection.renamedSkillId"
              placeholder="new-skill-id"
              @update:value="handleRename(activeSkill.sourcePath, String($event))"
            />
          </div>
        </div>
      </div>

      <div v-else-if="activeStep === 'preview'" class="mt-4 rounded-2xl border border-dashed border-line bg-panel/60 p-5 text-center text-sm text-muted">
        {{ t('feedback.repositoryPreviewDetected', { count: 0 }) }}
      </div>

      <div v-if="activeStep === 'result' && activeImportResult" class="mt-4 rounded-2xl border border-line bg-panel p-5 shadow-soft">
        <div class="flex items-center gap-2">
          <StatusBadge tone="ready" :label="`Imported: ${activeImportedSkills.length}`" />
          <StatusBadge tone="planned" :label="`Skipped: ${activeSkippedSkills.length}`" />
        </div>
        <div class="mt-4 grid gap-2">
          <div
            v-for="skill in activeImportedSkills"
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
