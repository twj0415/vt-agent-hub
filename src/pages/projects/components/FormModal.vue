<script setup lang="ts">
  import { computed, nextTick, ref, watch } from 'vue';
  import type { FormInstance, Rule } from 'ant-design-vue/es/form';
  import { projectImportModeOptions } from '@/shared/taxonomy';
  import VTModal from '@/shared/components/feedback/VTModal.vue';
  import PathPickerInput from '@/shared/components/forms/PathPickerInput.vue';
  import type { ProjectImportMode, ProjectType } from '@/shared/stores/projects-model';
  import { useProjectUi } from '../composables/useProjectUi';

  const { handlePathPickerError, projectTypeOptions, projectsStore, t, updateDraftPath, updateGitTargetPath } = useProjectUi();
  const formRef = ref<FormInstance>();

  const title = computed(() => {
    if (projectsStore.formIntent === 'edit') return t('catalog.action.edit');
    return projectsStore.importMode === 'git' ? t('pages.projects.importGit') : t('pages.projects.importTitle');
  });

  const okText = computed(() => {
    if (projectsStore.formIntent === 'edit') return t('catalog.action.save');
    return projectsStore.importMode === 'git' ? t('pages.projects.importGit') : t('catalog.action.import');
  });

  const isGitImport = computed(() => projectsStore.formIntent === 'import' && projectsStore.importMode === 'git');
  const formRules = computed<Record<string, Rule[]>>(() => ({
    path: [
      {
        required: true,
        whitespace: true,
        message: t(isGitImport.value ? 'errors.gitRepositoryRequired' : 'errors.projectPathRequired'),
        trigger: ['blur', 'change'],
      },
    ],
    gitTargetPath: isGitImport.value
      ? [
          {
            required: true,
            whitespace: true,
            message: t('errors.gitTargetPathRequired'),
            trigger: ['blur', 'change'],
          },
        ]
      : [],
    projectType: [
      {
        required: true,
        type: 'number',
        message: t('errors.projectTypeUnsupported'),
        trigger: 'change',
      },
    ],
  }));

  watch(
    () => projectsStore.formOpen,
    async (open) => {
      if (!open) return;
      await nextTick();
      formRef.value?.clearValidate();
    }
  );

  function updateProjectType(value: number) {
    projectsStore.setDraftField('projectType', value as ProjectType);
  }

  function setImportMode(value: string | number) {
    projectsStore.setImportMode(String(value) as ProjectImportMode);
  }

  async function submitForm() {
    try {
      await formRef.value?.validate();
    } catch {
      return;
    }
    await projectsStore.saveDraft();
  }
</script>

<template>
  <VTModal
    :open="projectsStore.formOpen"
    :title="title"
    :ok-text="okText"
    :cancel-text="t('common.close')"
    :loading="projectsStore.importLoading"
    :width="900"
    @ok="submitForm"
    @close="projectsStore.setFormOpen(false)"
  >
    <a-form ref="formRef" class="mx-auto max-w-[720px]" layout="vertical" :model="projectsStore.draft" :rules="formRules">
      <a-row :gutter="[16, 10]">
        <a-col v-if="projectsStore.formIntent === 'import'" :span="24">
          <a-form-item class="!mb-2">
            <a-segmented :value="projectsStore.importMode" block size="large" :options="projectImportModeOptions.map((item) => ({ value: item.value, label: t(item.labelKey) }))" @update:value="setImportMode" />
          </a-form-item>
        </a-col>

        <a-col v-if="isGitImport" :span="24">
          <a-form-item name="path" :label="t('pages.projects.gitRepository')" class="!mb-2">
            <a-input :value="projectsStore.draft.path" :placeholder="t('pages.projects.gitUrlPlaceholder')" @update:value="updateDraftPath(String($event))" />
          </a-form-item>
        </a-col>

        <a-col v-if="isGitImport" :span="24">
          <a-form-item name="gitTargetPath" :label="t('pages.projects.gitTargetPath')" class="!mb-2">
            <PathPickerInput
              :value="projectsStore.draft.gitTargetPath"
              :placeholder="t('pages.projects.gitTargetPathPlaceholder')"
              :button-text="t('common.selectFolder')"
              :disabled="projectsStore.importLoading"
              @update:value="updateGitTargetPath"
              @error="handlePathPickerError"
            />
          </a-form-item>
        </a-col>

        <a-col v-if="projectsStore.formIntent === 'edit' || projectsStore.importMode === 'local'" :span="24">
          <a-form-item name="path" :label="t('pages.projects.form.path')" class="!mb-2">
            <PathPickerInput
              :value="projectsStore.draft.path"
              :placeholder="t('pages.projects.pathPlaceholder')"
              :button-text="t('common.selectFolder')"
              :disabled="projectsStore.importLoading"
              @update:value="updateDraftPath"
              @error="handlePathPickerError"
            />
          </a-form-item>
        </a-col>

        <a-col :xs="24" :md="14">
          <a-form-item name="name" :label="t('pages.projects.form.name')" class="!mb-2">
            <a-input :value="projectsStore.draft.name" :placeholder="t('pages.projects.namePlaceholder')" @update:value="projectsStore.setDraftField('name', String($event))" />
          </a-form-item>
        </a-col>

        <a-col :xs="24" :md="10">
          <a-form-item name="projectType" :label="t('pages.projects.projectType')" class="!mb-2">
            <a-select :value="projectsStore.draft.projectType" :options="projectTypeOptions" @update:value="updateProjectType(Number($event))" />
          </a-form-item>
        </a-col>
      </a-row>
    </a-form>
  </VTModal>
</template>
