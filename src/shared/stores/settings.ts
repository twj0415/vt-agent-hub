import { defineStore } from 'pinia'
import type { SettingItem, SettingsPath, SettingsTruthSource } from '@/shared/api/client'
import type { MatrixRow } from '@/shared/types/ui'
import { getSettingsSnapshot, resetAppData } from '@/shared/api/tauri'
import { isTauriRuntime } from '@/shared/utils/runtime'
import { notifyError, notifySuccess } from '@/shared/utils/notify'
import { resolveAppError, resolveUnknownError, translateKey } from '@/shared/i18n/translate'
import { isThemePreset, useThemeStore, type ThemePreset } from './theme'
import { useI18nStore, type LocaleCode } from './i18n'

type SettingsState = {
  libraryRoot: string
  items: SettingItem[]
  paths: SettingsPath[]
  truthSources: SettingsTruthSource[]
  snapshotError: string
}

export const useSettingsStore = defineStore('settings', {
  state: () => ({
    libraryRoot: '',
    items: [] as SettingItem[],
    paths: [] as SettingsPath[],
    truthSources: [] as SettingsTruthSource[],
    snapshotError: '',
  } as SettingsState),
  getters: {
    themePreset(): ThemePreset {
      return useThemeStore().preset
    },
    themeMode(): ThemePreset {
      return this.themePreset
    },
    localeCode(): LocaleCode {
      return useI18nStore().locale
    },
    pathRows(state): MatrixRow[] {
      return state.paths.map((item) => ({
        label: item.key,
        value: `${item.path} | ${item.note}`,
        tone: item.key === 'storage_root' || item.key === 'library_root' ? 'ready' : undefined,
        badgeKey: item.key === 'storage_root' || item.key === 'library_root' ? 'common.ready' : undefined,
      }))
    },
    corePathRows(): MatrixRow[] {
      const orderedKeys = ['storage_root', 'app_db', 'library_root', 'backups', 'logs', 'snapshots', 'runtime', 'project_output']
      return orderedKeys
        .map((key) => this.paths.find((item) => item.key === key))
        .filter((item): item is SettingsPath => Boolean(item))
        .map((item) => ({
          label: item.key,
          value: `${item.path} | ${item.note}`,
          tone: item.key === 'project_output' ? 'warning' : 'ready',
          badgeKey: item.key === 'project_output' ? 'common.warning' : 'common.ready',
        }))
    },
    libraryPathRows(): MatrixRow[] {
      return this.paths
        .filter((item) => item.key.startsWith('library_'))
        .map((item) => ({
          label: item.key,
          value: `${item.path} | ${item.note}`,
          tone: 'ready',
          badgeKey: 'common.ready',
        }))
    },
    truthSourceRows(state): MatrixRow[] {
      return state.truthSources.map((item) => ({
        label: item.key,
        value: `${item.canonical} -> ${item.mirrors.join(', ') || translateKey('ui.common.none')} | ${item.note}`,
      }))
    },
    credentialRows(): MatrixRow[] {
      const credentialState = this.items.find((item) => item.name === 'tool_101_credential_state')
      return [
        {
          label: translateKey('ui.common.credentialStorage'),
          value: credentialState?.value === 'present'
            ? translateKey('ui.common.credentialStored')
            : translateKey('ui.common.credentialMissing'),
          tone: credentialState?.value === 'present' ? 'ready' : 'warning',
          badgeKey: credentialState?.value === 'present' ? 'common.ready' : 'common.warning',
        },
        {
          label: translateKey('ui.common.credentialBoundary'),
          value: translateKey('ui.common.credentialBoundaryDesc'),
        },
        {
          label: translateKey('ui.common.credentialActions'),
          value: translateKey('ui.common.credentialActionsDesc'),
        },
      ]
    },
    appearanceRows(): MatrixRow[] {
      return [
        {
          label: 'theme',
          value: this.themePreset,
          tone: 'ready',
          badgeKey: 'common.ready',
        },
        {
          label: 'language',
          value: this.localeCode,
          tone: 'ready',
          badgeKey: 'common.ready',
        },
      ]
    },
    storagePreview(): string {
      return this.corePathRows
        .map((row) => `${row.label}=${row.value.split(' | ')[0]}`)
        .join('\n')
    },
    boundaryPreview(): string {
      return this.truthSourceRows
        .map((row) => `${row.label}=${row.value}`)
        .join('\n')
    },
    credentialStateLabel(): string {
      const credentialState = this.items.find((item) => item.name === 'tool_101_credential_state')
      return credentialState?.value ?? translateKey('ui.common.missing')
    },
    hasSnapshotData(): boolean {
      return this.paths.length > 0 || this.truthSources.length > 0
    },
    pathCount(): number {
      return this.paths.length
    },
    truthSourceCount(): number {
      return this.truthSources.length
    },
    settingCount(): number {
      return this.items.length
    },
    systemSummaryRows(): MatrixRow[] {
      return [
        { label: translateKey('ui.common.settingsItems'), value: String(this.settingCount), tone: 'ready', badgeKey: 'common.ready' },
        { label: translateKey('ui.common.knownPaths'), value: String(this.pathCount), tone: 'ready', badgeKey: 'common.ready' },
        { label: translateKey('ui.common.truthSources'), value: String(this.truthSourceCount), tone: 'ready', badgeKey: 'common.ready' },
        { label: translateKey('ui.common.credentialState'), value: this.credentialStateLabel, tone: this.credentialStateLabel === 'present' ? 'ready' : 'warning', badgeKey: this.credentialStateLabel === 'present' ? 'common.ready' : 'common.warning' },
      ]
    },
    futureRows(): MatrixRow[] {
      return [
        {
          label: translateKey('ui.common.dangerousActions'),
          value: translateKey('ui.common.futureDangerousActions'),
          tone: 'warning',
          badgeKey: 'common.warning',
        },
      ]
    },
    storageRootPath(): string {
      return this.paths.find((item) => item.key === 'storage_root')?.path ?? ''
    },
    libraryRootPath(): string {
      return this.paths.find((item) => item.key === 'library_root')?.path ?? this.libraryRoot
    },
    projectOutputNote(): string {
      return this.paths.find((item) => item.key === 'project_output')?.note ?? ''
    },
    databasePath(): string {
      return this.paths.find((item) => item.key === 'app_db')?.path ?? ''
    },
    runtimePath(): string {
      return this.paths.find((item) => item.key === 'runtime')?.path ?? ''
    },
    backupsPath(): string {
      return this.paths.find((item) => item.key === 'backups')?.path ?? ''
    },
    snapshotsPath(): string {
      return this.paths.find((item) => item.key === 'snapshots')?.path ?? ''
    },
    logsPath(): string {
      return this.paths.find((item) => item.key === 'logs')?.path ?? ''
    },
    libraryNote(): string {
      return this.paths.find((item) => item.key === 'library_root')?.note ?? ''
    },
    appDbNote(): string {
      return this.paths.find((item) => item.key === 'app_db')?.note ?? ''
    },
    storageNote(): string {
      return this.paths.find((item) => item.key === 'storage_root')?.note ?? ''
    },
    runtimeNote(): string {
      return this.paths.find((item) => item.key === 'runtime')?.note ?? ''
    },
    backupsNote(): string {
      return this.paths.find((item) => item.key === 'backups')?.note ?? ''
    },
    snapshotsNote(): string {
      return this.paths.find((item) => item.key === 'snapshots')?.note ?? ''
    },
    logsNote(): string {
      return this.paths.find((item) => item.key === 'logs')?.note ?? ''
    },
    pageSummaryRows(): MatrixRow[] {
      return [
        { label: 'storage_root', value: this.storageRootPath || translateKey('ui.common.notLoaded') },
        { label: 'library_root', value: this.libraryRootPath || translateKey('ui.common.notLoaded') },
        { label: 'app_db', value: this.databasePath || translateKey('ui.common.notLoaded') },
        { label: 'project_output', value: this.projectOutputNote || translateKey('ui.common.notLoaded') },
      ]
    },
    pageNotes(): string[] {
      return [
        this.storageNote,
        this.libraryNote,
        this.appDbNote,
        this.backupsNote,
        this.logsNote,
        this.snapshotsNote,
        this.runtimeNote,
      ].filter(Boolean)
    },
    corePathsPreview(): string {
      return [
        this.storageRootPath && `storage_root=${this.storageRootPath}`,
        this.databasePath && `app_db=${this.databasePath}`,
        this.libraryRootPath && `library_root=${this.libraryRootPath}`,
        this.backupsPath && `backups=${this.backupsPath}`,
        this.logsPath && `logs=${this.logsPath}`,
        this.snapshotsPath && `snapshots=${this.snapshotsPath}`,
        this.runtimePath && `runtime=${this.runtimePath}`,
      ].filter(Boolean).join('\n')
    },
    truthSourcePreview(): string {
      return this.truthSources
        .map((item) => `${item.key}=${item.canonical} -> ${item.mirrors.join(', ') || translateKey('ui.common.none')}`)
        .join('\n')
    },
    credentialSummary(): string {
      return this.credentialRows.map((row) => `${row.label}=${row.value}`).join('\n')
    },
    appearancePreview(): string {
      return this.appearanceRows.map((row) => `${row.label}=${row.value}`).join('\n')
    },
    storageCardRows(): MatrixRow[] {
      return [
        { label: 'storage_root', value: `${this.storageRootPath} | ${this.storageNote}` },
        { label: 'app_db', value: `${this.databasePath} | ${this.appDbNote}` },
        { label: 'library_root', value: `${this.libraryRootPath} | ${this.libraryNote}` },
      ].filter((row) => row.value.trim() !== '|')
    },
    operationsCardRows(): MatrixRow[] {
      return [
        { label: 'backups', value: `${this.backupsPath} | ${this.backupsNote}` },
        { label: 'logs', value: `${this.logsPath} | ${this.logsNote}` },
        { label: 'snapshots', value: `${this.snapshotsPath} | ${this.snapshotsNote}` },
        { label: 'runtime', value: `${this.runtimePath} | ${this.runtimeNote}` },
      ].filter((row) => row.value.trim() !== '|')
    },
    projectBoundaryRows(): MatrixRow[] {
      return [
        {
          label: translateKey('ui.common.projectOutput'),
          value: this.projectOutputNote || translateKey('ui.common.projectOutputBoundary'),
          tone: 'warning',
          badgeKey: 'common.warning',
        },
      ]
    },
    themeLanguageRows(): MatrixRow[] {
      return [
        { label: 'theme', value: this.themePreset },
        { label: 'language', value: this.localeCode },
      ]
    },
    settingsRows(state): MatrixRow[] {
      return state.items.map((item) => ({
        label: item.name,
        value: item.value,
      }))
    },
  },
  actions: {
    async hydrateFromSnapshot() {
      try {
        const response = await getSettingsSnapshot()
        if (!response.success || !response.data) {
          if (isTauriRuntime()) throw new Error(resolveAppError(response.error, 'errors.settingsSnapshotFailed'))
          return
        }

        this.items = response.data.items
        const libraryRoot = response.data.items.find((item) => item.name === 'library_root')
        if (libraryRoot) {
          this.libraryRoot = libraryRoot.value
        }
        this.paths = response.data.paths
        this.truthSources = response.data.truthSources
        this.snapshotError = ''

        const theme = response.data.items.find((item) => item.name === 'theme')
        if (theme && isThemePreset(theme.value)) {
          useThemeStore().setPreset(theme.value)
        } else if (theme && (theme.value === 'light' || theme.value === 'system')) {
          useThemeStore().setPreset('apple')
        }
      } catch (error) {
        this.snapshotError = resolveUnknownError(error, 'errors.settingsSnapshotFailed')
        notifyError(this.snapshotError)
        if (isTauriRuntime()) throw error
      }
    },
    setLocaleCode(locale: LocaleCode) {
      useI18nStore().setLocale(locale)
    },
    setThemePreset(theme: ThemePreset) {
      useThemeStore().setPreset(theme)
    },
    async resetApplicationData() {
      try {
        const response = await resetAppData(true)
        if (!response.success || !response.data) {
          this.snapshotError = resolveAppError(response.error, 'errors.applicationResetFailed')
          notifyError(this.snapshotError)
          return
        }

        this.items = []
        this.paths = []
        this.truthSources = []
        this.libraryRoot = ''
        this.snapshotError = ''
        notifySuccess(response.data)
        window.location.reload()
      } catch (error) {
        this.snapshotError = resolveUnknownError(error, 'errors.applicationResetFailed')
        notifyError(this.snapshotError)
      }
    },
  },
})
