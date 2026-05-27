import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useSettingsStore } from './settings'
import * as tauriApi from '@/shared/api/tauri'

describe('settings store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.restoreAllMocks()
  })

  it('hydrates system paths, truth sources, and credential boundary from snapshot', async () => {
    vi.spyOn(tauriApi, 'getSettingsSnapshot').mockResolvedValue({
      success: true,
      data: {
        items: [
          { id: 1, name: 'theme', value: 'system' },
          { id: 2, name: 'library_root', value: 'C:\\Users\\Example\\.vt-agent-hub\\library' },
          { id: 3, name: 'tool_101_credential_state', value: 'present' },
        ],
        paths: [
          { key: 'storage_root', path: 'C:\\Users\\Example\\.vt-agent-hub', note: 'App data root.' },
          { key: 'app_db', path: 'C:\\Users\\Example\\.vt-agent-hub\\app.db', note: 'SQLite truth source.' },
          { key: 'library_root', path: 'C:\\Users\\Example\\.vt-agent-hub\\library', note: 'Skill library root used for local Skill source files.' },
          { key: 'backups', path: 'C:\\Users\\Example\\.vt-agent-hub\\backups', note: 'Managed backups.' },
          { key: 'logs', path: 'C:\\Users\\Example\\.vt-agent-hub\\logs', note: 'Runtime logs.' },
          { key: 'snapshots', path: 'C:\\Users\\Example\\.vt-agent-hub\\snapshots', note: 'Diagnostics exports.' },
          { key: 'runtime', path: 'C:\\Users\\Example\\.vt-agent-hub\\runtime', note: 'Runtime work area.' },
          { key: 'library_skills', path: 'C:\\Users\\Example\\.vt-agent-hub\\library\\skills', note: 'Skill assets.' },
          { key: 'project_output', path: 'Target project directory', note: 'Generated AGENTS.md files are written into registered project directories.' },
        ],
        truthSources: [
          { key: 'credentials', canonical: 'secure_storage', mirrors: ['sqlite'], note: 'Credential payloads stay in secure storage.' },
          { key: 'project_output', canonical: 'filesystem', mirrors: ['sqlite'], note: 'Generated AGENTS.md files are owned by the filesystem.' },
        ],
      },
    })

    const store = useSettingsStore()
    await store.hydrateFromSnapshot()

    expect(store.storageRootPath).toBe('C:\\Users\\Example\\.vt-agent-hub')
    expect(store.libraryRootPath).toBe('C:\\Users\\Example\\.vt-agent-hub\\library')
    expect(store.credentialStateLabel).toBe('present')
    expect(store.credentialRows[0].value).toContain('Windows 凭据管理器')
    expect(store.truthSourceRows[0].value).toContain('secure_storage')
    expect(store.storageCardRows.length).toBeGreaterThan(0)
  })
})
