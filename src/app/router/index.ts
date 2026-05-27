import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router'
import AppShell from '@/shared/components/shell/AppShell.vue'
import ProjectsPage from '@/pages/projects/index.vue'
import ToolsPage from '@/pages/tools/index.vue'
import RulesPage from '@/pages/rules/index.vue'
import SkillsPage from '@/pages/skills/index.vue'
import PresetsPage from '@/pages/presets/index.vue'
import HistoryPage from '@/pages/history/index.vue'
import SettingsPage from '@/pages/settings/index.vue'
import { appRoutes } from '@/shared/config/routes'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    component: AppShell,
    children: [
      { path: '', redirect: appRoutes.projects },
      {
        path: 'projects',
        name: 'projects',
        component: ProjectsPage,
        meta: { titleKey: 'pages.projects.title' },
      },
      {
        path: 'tools',
        name: 'tools',
        component: ToolsPage,
        meta: { titleKey: 'pages.tools.title' },
      },
      {
        path: 'assets',
        children: [
          {
            path: 'rules',
            name: 'rules',
            component: RulesPage,
            meta: { titleKey: 'pages.rules.title' },
          },
          {
            path: 'skills',
            name: 'skills',
            component: SkillsPage,
            meta: { titleKey: 'pages.skills.title' },
          },
          {
            path: 'presets',
            name: 'presets',
            component: PresetsPage,
            meta: { titleKey: 'pages.providers.title' },
          },
        ],
      },
      {
        path: 'history',
        name: 'history',
        component: HistoryPage,
        meta: { titleKey: 'pages.history.title' },
      },
      {
        path: 'settings',
        name: 'settings',
        component: SettingsPage,
        meta: { titleKey: 'pages.settings.title' },
      },
    ],
  },
]

export function createAppRouter() {
  return createRouter({
    history: createWebHashHistory(),
    routes,
  })
}
