import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useSkillStore } from './skills'
import * as tauriApi from '@/shared/api/tauri'

function buildLocalPreview() {
  return {
    rootPath: 'D:\\workspace\\my-skills',
    skills: [
      {
        sourcePath: 'foo',
        skillId: 'foo',
        skillName: 'foo',
        description: 'Foo description',
        rootDirectory: '/',
        skillDirectoryName: 'foo',
        conflict: null,
      },
      {
        sourcePath: 'bar',
        skillId: 'bar',
        skillName: 'bar',
        description: null,
        rootDirectory: '/',
        skillDirectoryName: 'bar',
        conflict: {
          existingSkillId: 99,
          existingName: 'bar',
        },
      },
    ],
  }
}

describe('skills store - local skill import', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.restoreAllMocks()
  })

  it('hydrates local preview state from previewLocalSkillImport response', async () => {
    vi.spyOn(tauriApi, 'previewLocalSkillImport').mockResolvedValue({
      success: true,
      data: buildLocalPreview(),
    })

    const store = useSkillStore()
    store.importDraft.localPath = 'D:\\workspace\\my-skills'

    await store.previewLocalImport()

    expect(store.localPreview?.skills.length).toBe(2)
    expect(store.localStep).toBe('preview')
    expect(store.localSelectedPath).toBe('foo')
    expect(store.localSelections.foo).toMatchObject({
      sourcePath: 'foo',
      selected: true,
      resolution: 'overwrite',
      renamedSkillId: 'foo',
    })
    // 冲突项默认 resolution 应该是 skip
    expect(store.localSelections.bar.resolution).toBe('skip')
  })

  it('refuses to preview local import when no path was picked', async () => {
    const previewSpy = vi.spyOn(tauriApi, 'previewLocalSkillImport')

    const store = useSkillStore()
    expect(store.importDraft.localPath).toBe('')

    await store.previewLocalImport()

    expect(previewSpy).not.toHaveBeenCalled()
    expect(store.actionError).toBe('请先选择本地 Skill 目录。')
  })

  it('keeps GitHub and local preview state isolated when sourceKind switches', async () => {
    vi.spyOn(tauriApi, 'previewLocalSkillImport').mockResolvedValue({
      success: true,
      data: buildLocalPreview(),
    })

    const store = useSkillStore()
    expect(store.importDraft.sourceKind).toBe('local')

    store.importDraft.localPath = 'D:\\workspace\\my-skills'
    await store.previewLocalImport()
    expect(store.localStep).toBe('preview')

    store.setSourceKind('github')
    // 切到 GitHub 不应清空已加载的 local 状态
    expect(store.localStep).toBe('preview')
    expect(store.localPreview?.skills.length).toBe(2)
    // GitHub 侧仍是初始 input
    expect(store.githubStep).toBe('input')
    expect(store.githubPreview).toBeNull()
  })

  it('previewSkillImport dispatches by sourceKind', async () => {
    const localSpy = vi
      .spyOn(tauriApi, 'previewLocalSkillImport')
      .mockResolvedValue({ success: true, data: buildLocalPreview() })
    const githubSpy = vi.spyOn(tauriApi, 'previewGitHubRepoImport')

    const store = useSkillStore()
    store.importDraft.localPath = 'D:\\workspace\\my-skills'
    await store.previewSkillImport()
    expect(localSpy).toHaveBeenCalledTimes(1)
    expect(githubSpy).not.toHaveBeenCalled()

    store.setSourceKind('github')
    store.importDraft.source = 'https://github.com/owner/repo'
    githubSpy.mockResolvedValue({
      success: true,
      data: {
        repo: {
          owner: 'owner',
          repo: 'repo',
          branch: 'main',
          normalizedUrl: 'https://github.com/owner/repo',
        },
        skills: [],
      },
    })
    await store.previewSkillImport()
    expect(githubSpy).toHaveBeenCalledTimes(1)
    expect(localSpy).toHaveBeenCalledTimes(1)
  })
})
