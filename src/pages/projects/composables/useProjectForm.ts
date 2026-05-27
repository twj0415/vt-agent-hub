import { useTaxonomyOptions } from '@/shared/composables/useTaxonomyOptions'
import { useProjectsStore } from '@/shared/stores/projects'
import { localizeMessage } from '@/shared/utils/message'
import { notifyError } from '@/shared/utils/notify'
import { pathName, repoName } from '@/shared/utils/path'

export function useProjectForm() {
  const projectsStore = useProjectsStore()
  const taxonomyOptions = useTaxonomyOptions()

  function updateDraftPath(value: string) {
    projectsStore.setDraftField('path', value)
    projectsStore.suggestDraftName(projectsStore.formIntent === 'import' && projectsStore.importMode === 'git' ? repoName(value) : pathName(value))
  }

  function updateGitTargetPath(value: string) {
    projectsStore.setDraftField('gitTargetPath', value)
    if (projectsStore.formIntent !== 'import' || projectsStore.importMode !== 'git') {
      projectsStore.suggestDraftName(pathName(value))
    }
  }

  function handlePathPickerError(message: string) {
    projectsStore.workflowError = localizeMessage(message || 'Folder picker failed.')
    notifyError(projectsStore.workflowError)
  }

  return {
    handlePathPickerError,
    projectTypeOptions: taxonomyOptions.projectTypes.options,
    updateDraftPath,
    updateGitTargetPath,
  }
}
