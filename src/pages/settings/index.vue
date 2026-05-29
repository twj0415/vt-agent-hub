<script setup lang="ts">
import { computed, ref } from 'vue'
import {
  DatabaseOutlined,
  ExperimentOutlined,
  FolderOpenOutlined,
  GlobalOutlined,
  ImportOutlined,
  ProjectOutlined,
  ToolOutlined,
} from '@ant-design/icons-vue'
import { useI18n } from 'vue-i18n'
import EmptyState from '@/shared/components/feedback/EmptyState.vue'
import { useSettingsStore } from '@/shared/stores/settings'
import SettingsGeneralSection from './components/SettingsGeneralSection.vue'
import SettingsAppearanceSection from './components/SettingsAppearanceSection.vue'
import SettingsStorageSection from './components/SettingsStorageSection.vue'
import SettingsToolsSection from './components/SettingsToolsSection.vue'
import SettingsRulesSection from './components/SettingsRulesSection.vue'
import SettingsImportsSection from './components/SettingsImportsSection.vue'
import SettingsMaintenanceSection from './components/SettingsMaintenanceSection.vue'

type SectionKey = 'general' | 'appearance' | 'storage' | 'tools' | 'rules' | 'imports' | 'maintenance'

const { t } = useI18n()
const settingsStore = useSettingsStore()
const activeSection = ref<SectionKey>('general')

const sectionOptions = computed(() => [
  { key: 'general' as const, title: t('ui.settings.sections.general'), icon: ProjectOutlined, component: SettingsGeneralSection },
  { key: 'appearance' as const, title: t('ui.settings.sections.appearance'), icon: GlobalOutlined, component: SettingsAppearanceSection },
  { key: 'storage' as const, title: t('ui.settings.sections.storage'), icon: DatabaseOutlined, component: SettingsStorageSection },
  { key: 'tools' as const, title: t('ui.settings.sections.tools'), icon: ToolOutlined, component: SettingsToolsSection },
  { key: 'rules' as const, title: t('ui.settings.sections.rules'), icon: FolderOpenOutlined, component: SettingsRulesSection },
  { key: 'imports' as const, title: t('ui.settings.sections.imports'), icon: ImportOutlined, component: SettingsImportsSection },
  { key: 'maintenance' as const, title: t('ui.settings.sections.maintenance'), icon: ExperimentOutlined, component: SettingsMaintenanceSection },
])

const currentSection = computed(() => sectionOptions.value.find((item) => item.key === activeSection.value) ?? sectionOptions.value[0])
</script>

<template>
  <div class="-mx-2 -mb-2 flex h-[calc(100%+8px)] min-h-0 flex-col md:-mx-3 md:-mb-3 md:h-[calc(100%+12px)]">
    <EmptyState
      v-if="settingsStore.snapshotError"
      size="sm"
      :fill="false"
      :description="settingsStore.snapshotError"
      class="mb-4"
    />

    <div class="grid min-h-0 flex-1 gap-3 lg:grid-cols-[224px_minmax(0,1fr)]">
      <aside class="vt-card flex min-h-0 flex-col p-3">
        <div class="px-2 pb-3 pt-1">
          <div class="text-[14px] font-semibold tracking-[-0.005em] text-text">{{ t('ui.common.settings') }}</div>
        </div>
        <nav class="grid gap-1">
          <button
            v-for="item in sectionOptions"
            :key="item.key"
            type="button"
            class="settings-nav-btn group relative flex w-full items-center gap-3 rounded-vt-md px-3 py-2.5 text-left transition-all duration-fast ease-standard"
            :class="activeSection === item.key ? 'settings-nav-btn-active text-text' : 'text-muted hover:bg-text/[0.04] hover:text-text'"
            @click="activeSection = item.key"
          >
            <span
              v-if="activeSection === item.key"
              class="absolute left-1 top-1/2 h-3.5 w-[2px] -translate-y-1/2 rounded-full bg-accent"
              aria-hidden="true"
            />
            <component
              :is="item.icon"
              class="shrink-0 text-[14px]"
              :class="activeSection === item.key ? 'text-accent' : 'text-muted'"
            />
            <span class="min-w-0 truncate text-[13px] font-semibold">{{ item.title }}</span>
          </button>
        </nav>
      </aside>

      <section class="vt-card flex min-h-0 min-w-0 flex-col overflow-hidden">
        <header class="shrink-0 border-b border-line/60 px-5 py-3.5">
          <div class="flex items-center gap-2 text-[10px] font-semibold uppercase tracking-[0.14em] text-muted">
            <component :is="currentSection.icon" class="text-accent" />
            {{ t('ui.common.settings') }}
          </div>
          <h3 class="mt-1 text-[16px] font-semibold tracking-[-0.018em] text-text">{{ currentSection.title }}</h3>
        </header>

        <div class="min-h-0 flex-1 overflow-auto p-5">
          <component :is="currentSection.component" />
        </div>
      </section>
    </div>
  </div>
</template>

<style scoped>
.settings-nav-btn-active {
  background: rgb(var(--vt-color-accent) / 0.10);
  box-shadow: inset 0 0 0 1px rgb(var(--vt-color-accent) / 0.18);
}
.settings-nav-btn-active::after {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: inherit;
  pointer-events: none;
  background: linear-gradient(90deg, rgb(var(--vt-color-accent) / 0.06) 0%, transparent 60%);
}
</style>
