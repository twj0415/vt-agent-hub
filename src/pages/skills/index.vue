<script setup lang="ts">
import { ImportOutlined, PlusOutlined } from '@ant-design/icons-vue'
import EmptyState from '@/shared/components/feedback/EmptyState.vue'
import PageHeader from '@/shared/components/shell/PageHeader.vue'
import RepositoryImportModal from '@/features/imports/RepositoryImportModal.vue'
import SkillFormModal from '@/features/skills/SkillFormModal.vue'
import DetailDrawer from './components/DetailDrawer.vue'
import SkillCard from './components/SkillCard.vue'
import SkillToolBindModal from './components/SkillToolBindModal.vue'
import { useSkillsWorkbench } from './composables/useSkillsWorkbench'

const { categoryOptions, skillCards, skillListBusy, skillStore, t } = useSkillsWorkbench()
</script>

<template>
  <div class="workbench-page">
    <PageHeader :title="t('pages.skills.title')" :count="skillCards.length">
      <a-button @click="skillStore.setImportOpen(true)">
        <template #icon><ImportOutlined /></template>
        {{ t('pages.skills.importRepository') }}
      </a-button>
      <a-button type="primary" @click="skillStore.openCreate()">
        <template #icon><PlusOutlined /></template>
        {{ t('pages.skills.create') }}
      </a-button>
    </PageHeader>

    <div class="workbench-body">
      <div v-if="skillCards.length" class="entity-grid">
        <SkillCard v-for="item in skillCards" :key="item.id" :item="item" :busy="skillListBusy" />
      </div>
      <EmptyState
        v-else
        :title="'pages.skills.emptyState.title'"
        :description="'pages.skills.emptyState.description'"
        :action-label="'pages.skills.emptyState.cta'"
        @action="skillStore.openCreate()"
      />
      <div v-if="skillListBusy && skillCards.length" class="list-loading-mask">
        <a-spin />
      </div>
    </div>
  </div>

  <DetailDrawer />
  <SkillToolBindModal />

  <SkillFormModal
    :open="skillStore.formOpen"
    :draft="skillStore.draft"
    :category-options="categoryOptions as Array<{ value: number; label: string }>"
    @close="skillStore.setFormOpen(false)"
    @save="skillStore.saveDraft()"
    @field="skillStore.setDraftField($event.key, $event.value as never)"
  />

  <RepositoryImportModal
    :open="skillStore.importOpen"
    :draft="skillStore.importDraft"
    :loading="skillStore.importLoading"
    :report="skillStore.repositoryImportReport"
    :github-preview="skillStore.githubPreview"
    :github-import-result="skillStore.githubImportResult"
    :github-selections="skillStore.githubSelections"
    :github-selected-path="skillStore.githubSelectedPath"
    :github-step="skillStore.githubStep"
    @close="skillStore.setImportOpen(false)"
    @preview="skillStore.previewGitHubImport()"
    @apply="skillStore.applyGitHubImport()"
    @field="skillStore.setImportField($event.key, $event.value)"
    @toggle-github-skill="skillStore.toggleGitHubSkillSelection($event)"
    @github-resolution="skillStore.setGitHubSkillResolution($event.sourcePath, $event.resolution)"
    @github-rename="skillStore.setGitHubSkillRename($event.sourcePath, $event.renamedSkillId)"
    @github-selected-path="skillStore.setGitHubSelectedPath($event)"
  />
</template>
