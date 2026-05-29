import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useProjectsStore } from './projects'
import { useRuleStore } from './rules'
import * as tauriApi from '@/shared/api/tauri'

describe('projects store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.restoreAllMocks()
  })

  it('imports project entities without storing tool_ids', async () => {
    vi.spyOn(tauriApi, 'saveProjectEntity').mockResolvedValue({
      success: true,
      data: {
        id: 10,
        name: 'imported',
        path: 'C:\\imported',
        projectType: 201,
        ruleBindings: [],
        lastOperation: '',
        latestBackup: '',
      },
    })
    vi.spyOn(tauriApi, 'getProjectContextSnapshot').mockResolvedValue({
      success: true,
      data: {
        activeProjectId: 10,
        activeToolId: 101,
        projects: [
          {
            id: 10,
            name: 'imported',
            path: 'C:\\imported',
            projectType: 201,
            ruleBindings: [],
            lastOperation: 'Imported project profile into entity list.',
            latestBackup: 'No backup yet',
          },
        ],
      },
    })

    const store = useProjectsStore()

    store.openImport()
    store.setDraftField('path', 'C:\\imported')
    await store.saveDraft()

    const imported = store.items.find((item) => item.path === 'C:\\imported')
    expect(imported?.name).toBe('imported')
    expect('tool_ids' in (imported ?? {})).toBe(false)
  })

  it('imports project entities from git clone input', async () => {
    const importSpy = vi.spyOn(tauriApi, 'importProjectFromGit').mockResolvedValue({
      success: true,
      data: {
        id: 11,
        name: 'repo',
        path: 'C:\\projects\\repo',
        projectType: 201,
        ruleBindings: [],
        lastOperation: 'Project imported.',
        latestBackup: '',
      },
    })
    vi.spyOn(tauriApi, 'getProjectContextSnapshot').mockResolvedValue({
      success: true,
      data: {
        activeProjectId: 11,
        activeToolId: 101,
        projects: [
          {
            id: 11,
            name: 'repo',
            path: 'C:\\projects\\repo',
            projectType: 201,
            ruleBindings: [],
            lastOperation: 'Project imported.',
            latestBackup: 'No backup yet',
          },
        ],
      },
    })

    const store = useProjectsStore()

    store.openImport('git')
    store.setDraftField('path', 'https://github.com/example/repo.git')
    store.setDraftField('gitTargetPath', 'C:\\projects')
    await store.saveDraft()

    expect(importSpy).toHaveBeenCalledWith({
      repoUrl: 'https://github.com/example/repo.git',
      targetPath: 'C:\\projects\\repo',
      name: 'repo',
      branch: undefined,
      projectType: 201,
    })
    expect(store.activeId).toBe(11)
    expect(store.items[0]?.path).toBe('C:\\projects\\repo')
  })

  it('blocks importing a project when the local path already exists', async () => {
    const saveSpy = vi.spyOn(tauriApi, 'saveProjectEntity').mockResolvedValue({
      success: true,
      data: {
        id: 11,
        name: 'duplicate',
        path: 'C:\\imported',
        projectType: 201,
        ruleBindings: [],
        lastOperation: '',
        latestBackup: '',
      },
    })
    const store = useProjectsStore()
    store.items = [
      {
        id: 10,
        name: 'imported',
        path: 'C:\\imported',
        projectType: 201,
        ruleBindings: [],
        lastOperation: '',
        latestBackup: '',
      },
    ]

    store.openImport()
    store.setDraftField('path', 'C:\\imported\\')
    await store.saveDraft()

    expect(saveSpy).not.toHaveBeenCalled()
    expect(store.workflowError).toContain('项目路径已存在')
    expect(store.workflowError).toContain('C:\\imported')
  })

  it('blocks importing a git project when the final target path already exists', async () => {
    const importSpy = vi.spyOn(tauriApi, 'importProjectFromGit').mockResolvedValue({
      success: true,
      data: {
        id: 12,
        name: 'repo',
        path: 'C:\\projects\\repo',
        projectType: 201,
        ruleBindings: [],
        lastOperation: '',
        latestBackup: '',
      },
    })
    const store = useProjectsStore()
    store.items = [
      {
        id: 10,
        name: 'repo',
        path: 'C:\\projects\\repo',
        projectType: 201,
        ruleBindings: [],
        lastOperation: '',
        latestBackup: '',
      },
    ]

    store.openImport('git')
    store.setDraftField('path', 'https://github.com/example/repo.git')
    store.setDraftField('gitTargetPath', 'C:\\projects')
    await store.saveDraft()

    expect(importSpy).not.toHaveBeenCalled()
    expect(store.workflowError).toContain('项目路径已存在')
    expect(store.workflowError).toContain('C:\\projects\\repo')
  })

  it('hydrates output scans for every project from the project snapshot', async () => {
    vi.spyOn(tauriApi, 'getProjectContextSnapshot').mockResolvedValue({
      success: true,
      data: {
        activeProjectId: 1,
        activeToolId: 101,
        projects: [
          {
            id: 1,
            name: 'Ready Project',
            path: 'C:\\projects\\ready',
            projectType: 201,
            ruleBindings: [],
            lastOperation: '',
            latestBackup: '',
            outputScan: {
              projectId: 1,
              toolId: 101,
              projectName: 'Ready Project',
              targetPath: 'C:\\projects\\ready\\AGENTS.md',
              targetExists: true,
              managed: true,
              ruleCount: 1,
              status: 'ready',
              statusCode: 502,
              issues: [],
            },
          },
          {
            id: 2,
            name: 'Missing Project',
            path: 'C:\\projects\\missing',
            projectType: 201,
            ruleBindings: [],
            lastOperation: '',
            latestBackup: '',
            outputScan: {
              projectId: 2,
              toolId: 101,
              projectName: 'Missing Project',
              targetPath: 'C:\\projects\\missing\\AGENTS.md',
              targetExists: false,
              managed: false,
              ruleCount: 0,
              status: 'missing',
              statusCode: 503,
              issues: ['project_path_missing'],
            },
          },
        ],
      },
    })

    const store = useProjectsStore()
    await store.hydrateFromSnapshot()

    expect(store.items[0]?.outputScan?.status).toBe('ready')
    expect(store.items[1]?.outputScan?.status).toBe('missing')
    expect(store.outputScan?.projectId).toBe(1)

    store.select(2)
    expect(store.outputScan?.status).toBe('missing')
  })

  it('updates bound rules through project rule binding actions', async () => {
    vi.spyOn(tauriApi, 'saveProjectRuleBindings').mockResolvedValue({
      success: true,
      data: true,
    })
    const scanSpy = vi.spyOn(tauriApi, 'scanProjectOutput').mockResolvedValue({
      success: true,
      data: {
        projectId: 1,
        toolId: 102,
        projectName: 'Example Project',
        targetPath: 'C:\\Users\\Example\\Desktop\\ExampleProject\\CLAUDE.md',
        targetExists: false,
        managed: false,
        ruleCount: 1,
        status: 'planned',
        statusCode: 504,
        issues: [],
      },
    })
    const applySpy = vi.spyOn(tauriApi, 'applyProjectOutput').mockResolvedValue({
      success: true,
      data: {
        projectId: 1,
        toolId: 102,
        operation: 'project.apply_agents',
        targetPath: 'C:\\Users\\Example\\Desktop\\ExampleProject\\CLAUDE.md',
        backupPath: null,
        managed: true,
        created: true,
        message: 'Project CLAUDE.md applied.',
      },
    })
    vi.spyOn(tauriApi, 'getProjectContextSnapshot').mockResolvedValue({
      success: true,
      data: {
        activeProjectId: 1,
        activeToolId: 101,
        projects: [
          {
            id: 1,
            name: 'Example Project',
            path: 'C:\\Users\\Example\\Desktop\\ExampleProject',
            projectType: 203,
            ruleBindings: [
              {
                toolId: null,
                packId: 2,
                packName: 'Codex Project Rules',
                packType: 'project_rules',
                packVersionId: 2,
                packVersionNo: 1,
                updatePolicy: 'notify',
                enabled: true,
                items: [
                  { itemType: 'rule', assetId: 2, assetVersionId: 2, sortOrder: 0, required: true },
                ],
              },
            ],
            lastOperation: 'Updated 1 bound rules.',
            latestBackup: 'No backup yet',
          },
        ],
      },
    })

    const store = useProjectsStore()
    const ruleStore = useRuleStore()
    ruleStore.items = [
      {
        id: 2,
        versionId: 2,
        versionNo: 1,
        key: 'codex-project-rules',
        name: 'Codex Project Rule',
        code: 304,
        state: 'ready',
        sortOrder: 0,
        summary: 'Rule summary',
        categoryCode: 304,
        body: 'Rule body',
        impact: null,
      },
    ]
    await store.hydrateFromSnapshot()

    store.openRuleBinding()
    store.setBindingTargetToolId(102)
    await store.applyRuleBinding()

    expect(store.activeItem?.ruleBindings[0]?.packVersionId).toBe(2)
    expect(store.bindingDraft.targetToolId).toBe(102)
    expect(tauriApi.saveProjectRuleBindings).toHaveBeenCalledWith(1, null, [2])
    expect(scanSpy).toHaveBeenCalledWith(1, 102)
    expect(applySpy).toHaveBeenCalledWith(1, 102, true)
  })

  it('rolls project rule bindings back when project output write fails', async () => {
    const saveSpy = vi.spyOn(tauriApi, 'saveProjectRuleBindings').mockResolvedValue({
      success: true,
      data: true,
    })
    vi.spyOn(tauriApi, 'scanProjectOutput').mockResolvedValue({
      success: true,
      data: {
        projectId: 1,
        toolId: 101,
        projectName: 'Example Project',
        targetPath: 'C:\\Users\\Example\\Desktop\\ExampleProject\\AGENTS.md',
        targetExists: false,
        managed: false,
        ruleCount: 2,
        status: 'planned',
        statusCode: 504,
        issues: [],
      },
    })
    vi.spyOn(tauriApi, 'applyProjectOutput').mockResolvedValue({
      success: false,
      error: {
        code: 'project_output_write_failed',
        message: 'Project output write failed.',
        i18nKey: 'errors.projectOutputWriteFailed',
      },
    })
    vi.spyOn(tauriApi, 'getProjectContextSnapshot').mockResolvedValue({
      success: true,
      data: {
        activeProjectId: 1,
        activeToolId: 101,
        projects: [
          {
            id: 1,
            name: 'Example Project',
            path: 'C:\\Users\\Example\\Desktop\\ExampleProject',
            projectType: 203,
            ruleBindings: [
              {
                toolId: null,
                packId: 2,
                packName: 'Codex Project Rules',
                packType: 'project_rules',
                packVersionId: 2,
                packVersionNo: 1,
                updatePolicy: 'notify',
                enabled: true,
                items: [
                  { itemType: 'rule', assetId: 1, assetVersionId: 1, sortOrder: 0, required: true },
                ],
              },
            ],
            lastOperation: 'Updated 1 bound rules.',
            latestBackup: 'No backup yet',
          },
        ],
      },
    })

    const store = useProjectsStore()
    await store.hydrateFromSnapshot()

    const result = await store.saveProjectRuleIdsAndSync(1, [1, 2], 101)

    expect(result).toBe('failed')
    expect(saveSpy).toHaveBeenNthCalledWith(1, 1, null, [1, 2])
    expect(saveSpy).toHaveBeenNthCalledWith(2, 1, null, [1])
    expect(store.workflowError.length).toBeGreaterThan(0)
  })

  it('blocks saving bindings when no rules are selected', async () => {
    const saveSpy = vi.spyOn(tauriApi, 'saveProjectRuleBindings').mockResolvedValue({
      success: true,
      data: true,
    })
    const store = useProjectsStore()
    store.items = [
      {
        id: 1,
        name: 'Example Project',
        path: 'C:\\Users\\Example\\Desktop\\ExampleProject',
        projectType: 203,
        ruleBindings: [],
        lastOperation: '',
        latestBackup: '',
      },
    ]
    store.select(1)
    store.bindingDraft.selectedRuleIds = []

    await store.applyRuleBinding()

    expect(saveSpy).not.toHaveBeenCalled()
    expect(store.workflowError.length).toBeGreaterThan(0)
  })

  it('runs cleanup and reset actions through project output workflow', async () => {
    vi.spyOn(tauriApi, 'scanProjectOutput').mockResolvedValue({
      success: true,
      data: {
        projectId: 1,
        toolId: 101,
        projectName: 'Example Project',
        targetPath: 'C:\\Users\\Example\\Desktop\\ExampleProject\\AGENTS.md',
        targetExists: true,
        managed: true,
        ruleCount: 2,
        status: 'ready',
        statusCode: 502,
        issues: [],
      },
    })
    const cleanupSpy = vi.spyOn(tauriApi, 'cleanupProjectOutput').mockResolvedValue({
      success: true,
      data: {
        projectId: 1,
        toolId: 101,
        operation: 'project.cleanup_agents',
        targetPath: 'C:\\Users\\Example\\Desktop\\ExampleProject\\AGENTS.md',
        backupPath: 'C:\\Users\\Example\\.vt-agent-hub\\backups\\project-1\\123-AGENTS.md',
        managed: false,
        created: false,
        message: 'Project AGENTS.md removed.',
      },
    })
    const resetSpy = vi.spyOn(tauriApi, 'resetProjectOutput').mockResolvedValue({
      success: true,
      data: {
        projectId: 1,
        toolId: 101,
        operation: 'project.reset_agents',
        targetPath: 'C:\\Users\\Example\\Desktop\\ExampleProject\\AGENTS.md',
        backupPath: 'C:\\Users\\Example\\.vt-agent-hub\\backups\\project-1\\124-AGENTS.md',
        managed: false,
        created: false,
        message: 'Project AGENTS.md reset to unmanaged state.',
      },
    })
    vi.spyOn(tauriApi, 'getProjectContextSnapshot').mockResolvedValue({
      success: true,
      data: {
        activeProjectId: 1,
        activeToolId: 101,
        projects: [
          {
            id: 1,
            name: 'Example Project',
            path: 'C:\\Users\\Example\\Desktop\\ExampleProject',
            projectType: 203,
            ruleBindings: [
              {
                toolId: null,
                packId: 2,
                packName: 'Codex Project Rules',
                packType: 'project_rules',
                packVersionId: 2,
                packVersionNo: 1,
                updatePolicy: 'notify',
                enabled: true,
                items: [
                  { itemType: 'rule', assetId: 2, assetVersionId: 2, sortOrder: 0, required: true },
                ],
              },
            ],
            lastOperation: 'Project AGENTS cleanup completed.',
            latestBackup: '123-AGENTS.md',
          },
        ],
      },
    })

    const store = useProjectsStore()
    await store.hydrateFromSnapshot()

    await store.openOutputRemoval(101, 'cleanup')
    expect(store.outputAction).toBe('cleanup')
    expect(store.previewOpen).toBe(true)

    await store.confirmOutput(101)
    expect(cleanupSpy).toHaveBeenCalledWith(1, 101, true)
    expect(store.outputResult?.operation).toBe('project.cleanup_agents')

    await store.openOutputRemoval(101, 'reset')
    await store.confirmOutput(101)
    expect(resetSpy).toHaveBeenCalledWith(1, 101, true)
    expect(store.outputResult?.operation).toBe('project.reset_agents')
  })

  it('labels missing local project folders as a project path problem', async () => {
    vi.spyOn(tauriApi, 'scanProjectOutput').mockResolvedValue({
      success: true,
      data: {
        projectId: 1,
        toolId: 101,
        projectName: 'Missing Project',
        targetPath: 'C:\\missing\\AGENTS.md',
        targetExists: false,
        managed: false,
        ruleCount: 0,
        status: 'missing',
        statusCode: 503,
        issues: ['project_path_missing', 'missing_target'],
      },
    })

    const store = useProjectsStore()
    store.items = [
      {
        id: 1,
        name: 'Missing Project',
        path: 'C:\\missing',
        projectType: 203,
        ruleBindings: [],
        lastOperation: '',
        latestBackup: '',
      },
    ]
    store.select(1)

    await store.scanOutput(101, { silent: true })

    expect(store.outputScan?.issues).toContain('project_path_missing')
    expect(store.outputScan?.status).toBe('missing')
  })
})
