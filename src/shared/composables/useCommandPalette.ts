import { computed, ref, onMounted, onBeforeUnmount } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { appRoutes, type AppRouteKey } from '@/shared/config/routes'
import { useProjectsStore } from '@/shared/stores/projects'
import { useRuleStore } from '@/shared/stores/rules'
import { useSkillStore } from '@/shared/stores/skills'
import { useProvidersStore } from '@/shared/stores/providers'
import { useToolsStore } from '@/shared/stores/tools'
import { useToolContextStore } from '@/shared/stores/tool-context'
import { toolRegistry, type ToolId } from '@/shared/tool-registry'

// 命令面板的单条结果。type 用于显示组与命中后跳转逻辑。
export type CommandResult = {
  id: string
  group: CommandGroup
  label: string
  hint?: string
  to?: string
  payload?: unknown
}

export type CommandGroup = 'pages' | 'projects' | 'rules' | 'skills' | 'providers' | 'tools'

// 模块级单例：保证全局唯一一个 ⌘K 浮层。
const open = ref(false)
const query = ref('')
const activeIndex = ref(0)
let listenerInstalled = false
let listenerRefCount = 0

function isPaletteHotkey(event: KeyboardEvent) {
  return (event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k'
}

function handleKeydown(event: KeyboardEvent) {
  if (isPaletteHotkey(event)) {
    event.preventDefault()
    toggle()
  }
}

function toggle() {
  open.value = !open.value
  if (open.value) {
    query.value = ''
    activeIndex.value = 0
  }
}

function close() {
  open.value = false
}

function openPalette() {
  open.value = true
  query.value = ''
  activeIndex.value = 0
}

// 简单子串打分：完全匹配 > 前缀 > 包含。返回 -1 表示不命中。
function score(haystack: string, needle: string): number {
  if (!needle) return 0
  const lowerHaystack = haystack.toLowerCase()
  const lowerNeedle = needle.toLowerCase()
  if (lowerHaystack === lowerNeedle) return 100
  if (lowerHaystack.startsWith(lowerNeedle)) return 80
  const idx = lowerHaystack.indexOf(lowerNeedle)
  if (idx >= 0) return 60 - idx
  return -1
}

export function useCommandPalette() {
  const { t } = useI18n()
  const router = useRouter()
  const projectsStore = useProjectsStore()
  const ruleStore = useRuleStore()
  const skillStore = useSkillStore()
  const providersStore = useProvidersStore()
  const toolsStore = useToolsStore()
  const toolContextStore = useToolContextStore()

  // 每组上限避免长尾刷屏。
  const PER_GROUP_LIMIT = 5

  function buildPageResults(needle: string): CommandResult[] {
    const pages: Array<{ key: AppRouteKey; labelKey: string }> = [
      { key: 'projects', labelKey: 'nav.projects' },
      { key: 'tools', labelKey: 'nav.tools' },
      { key: 'rules', labelKey: 'nav.rules' },
      { key: 'skills', labelKey: 'nav.skills' },
      { key: 'presets', labelKey: 'nav.providers' },
      { key: 'history', labelKey: 'nav.history' },
      { key: 'settings', labelKey: 'nav.settings' },
    ]
    return pages
      .map((page) => ({
        id: `page:${page.key}`,
        group: 'pages' as const,
        label: t(page.labelKey),
        hint: appRoutes[page.key],
        to: appRoutes[page.key],
        _score: score(t(page.labelKey), needle),
      }))
      .filter((r) => r._score >= 0 || !needle)
      .sort((a, b) => b._score - a._score)
      .slice(0, PER_GROUP_LIMIT)
      .map(({ _score, ...rest }) => rest)
  }

  function buildResults<T extends { id: number | ToolId; name?: string; key?: string }>(
    group: CommandGroup,
    items: T[],
    needle: string,
    extractor: (item: T) => { label: string; hint?: string; payload?: unknown },
  ): CommandResult[] {
    return items
      .map((item) => {
        const { label, hint, payload } = extractor(item)
        const s = Math.max(score(label, needle), hint ? score(hint, needle) : -1)
        return { item, label, hint, payload, score: s }
      })
      .filter((r) => r.score >= 0 || !needle)
      .sort((a, b) => b.score - a.score)
      .slice(0, PER_GROUP_LIMIT)
      .map((r) => ({
        id: `${group}:${r.item.id}`,
        group,
        label: r.label,
        hint: r.hint,
        payload: r.payload ?? r.item.id,
      }))
  }

  const results = computed<CommandResult[]>(() => {
    const needle = query.value.trim()

    const pageResults = buildPageResults(needle)

    const projectResults = buildResults('projects', projectsStore.items, needle, (p) => ({
      label: p.name,
      hint: p.path,
    }))

    const ruleResults = buildResults('rules', ruleStore.items, needle, (r) => ({
      label: r.name,
      hint: r.summary,
    }))

    const skillResults = buildResults('skills', skillStore.items, needle, (s) => ({
      label: s.name,
      hint: s.summary,
    }))

    const providerResults = buildResults('providers', providersStore.items, needle, (p) => ({
      label: p.name,
      hint: p.category,
    }))

    const toolResults = toolRegistry
      .filter((tool) => tool.enabled)
      .map((tool) => ({
        item: tool,
        label: t(tool.nameKey),
        _score: score(t(tool.nameKey), needle),
      }))
      .filter((r) => r._score >= 0 || !needle)
      .sort((a, b) => b._score - a._score)
      .slice(0, PER_GROUP_LIMIT)
      .map((r) => ({
        id: `tools:${r.item.id}`,
        group: 'tools' as const,
        label: r.label,
        hint: r.item.key,
        payload: r.item.id,
      }))

    return [...pageResults, ...projectResults, ...ruleResults, ...skillResults, ...providerResults, ...toolResults]
  })

  function selectNext() {
    const total = results.value.length
    if (!total) return
    activeIndex.value = (activeIndex.value + 1) % total
  }

  function selectPrev() {
    const total = results.value.length
    if (!total) return
    activeIndex.value = (activeIndex.value - 1 + total) % total
  }

  async function execute(result?: CommandResult) {
    const target = result ?? results.value[activeIndex.value]
    if (!target) return

    close()

    if (target.group === 'pages' && target.to) {
      await router.push(target.to)
      return
    }

    if (target.group === 'projects') {
      const id = Number(target.payload)
      await router.push(appRoutes.projects)
      projectsStore.select(id)
      projectsStore.setDetailOpen(true)
      return
    }

    if (target.group === 'rules') {
      const id = Number(target.payload)
      await router.push(appRoutes.rules)
      ruleStore.select(id)
      ruleStore.setDetailOpen(true)
      return
    }

    if (target.group === 'skills') {
      const id = Number(target.payload)
      await router.push(appRoutes.skills)
      skillStore.select(id)
      skillStore.setDetailOpen(true)
      return
    }

    if (target.group === 'providers') {
      const id = Number(target.payload)
      await router.push(appRoutes.presets)
      providersStore.openEdit(id)
      return
    }

    if (target.group === 'tools') {
      const id = Number(target.payload) as ToolId
      await router.push(appRoutes.tools)
      toolsStore.select(id)
      toolContextStore.setActiveTool(id)
    }
  }

  // 全局键盘监听通过 refCount 共享，避免多组件同时挂载时重复绑定。
  onMounted(() => {
    if (!listenerInstalled) {
      window.addEventListener('keydown', handleKeydown)
      listenerInstalled = true
    }
    listenerRefCount += 1
  })

  onBeforeUnmount(() => {
    listenerRefCount -= 1
    if (listenerRefCount <= 0 && listenerInstalled) {
      window.removeEventListener('keydown', handleKeydown)
      listenerInstalled = false
      listenerRefCount = 0
    }
  })

  return {
    open,
    query,
    activeIndex,
    results,
    toggle,
    openPalette,
    close,
    selectNext,
    selectPrev,
    execute,
  }
}
