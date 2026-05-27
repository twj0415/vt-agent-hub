import { defineStore } from 'pinia'
import { getAppBootstrap } from '@/shared/api/tauri'
import { productConfig } from '@/shared/config/product'
import { getToolById, toolIds } from '@/shared/tool-registry'
import { isTauriRuntime } from '@/shared/utils/runtime'
import { notifyError } from '@/shared/utils/notify'
import { useRuleStore } from './rules'
import { useSkillStore } from './skills'
import { useHistoryStore } from './history'
import { useProjectsStore } from './projects'
import { useSettingsStore } from './settings'
import { useToolsStore } from './tools'
import { useToolContextStore } from './tool-context'

export const useAppStore = defineStore('app', {
  state: () => ({
    ready: false,
    loading: false,
    bootstrapped: false,
    appName: productConfig.name as string,
    error: '',
    bootstrapErrors: [] as string[],
  }),
  actions: {
    setBootstrapError(message: string) {
      this.error = message
      this.bootstrapErrors.push(message)
      notifyError(message)
    },
    async bootstrapCore() {
      try {
        const response = await getAppBootstrap()
        if (response.success && response.data) {
          this.appName = response.data.appName ?? response.data.projectName ?? productConfig.name
          const toolContext = useToolContextStore()
          if (!toolContext.loadPersistedTool()) {
            const tool = getToolById(response.data.activeToolId)
            toolContext.setActiveTool(tool?.id ?? toolIds.codex)
          }
          this.bootstrapped = true
          return
        }

        throw new Error(response.error?.message ?? 'bootstrap failed')
      } catch (error) {
        throw new Error(error instanceof Error ? error.message : 'bootstrap failed')
      }
    },
    async bootstrapAll() {
      this.loading = true
      this.ready = false
      this.error = ''
      this.bootstrapErrors = []

      try {
        await this.bootstrapCore()
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Application bootstrap failed.'
        this.setBootstrapError(message)
        this.loading = false
        this.ready = true
        return
      }

      const steps = [
        () => useToolContextStore().hydrateFromSnapshot(),
        () => useToolsStore().hydrateFromSnapshot(),
        () => useRuleStore().hydrateFromSnapshot(),
        () => useSkillStore().hydrateFromSnapshot(),
        () => useHistoryStore().hydrateFromSnapshot(),
        () => useSettingsStore().hydrateFromSnapshot(),
      ]

      for (const step of steps) {
        try {
          await step()
        } catch (error) {
          const message = error instanceof Error ? error.message : 'Application bootstrap failed.'
          this.setBootstrapError(message)
          if (isTauriRuntime()) break
        }
      }

      useProjectsStore().activeId = useToolContextStore().activeProjectId ?? useProjectsStore().activeId
      this.loading = false
      this.ready = true
    },
  },
})
