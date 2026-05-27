import { ref } from 'vue'
import { useProjectsStore } from '@/shared/stores/projects'
import { useToolContextStore } from '@/shared/stores/tool-context'
import { notifyInfo, notifySuccess, notifyWarning, notifyError } from '@/shared/utils/notify'
import { translateKey } from '@/shared/i18n/translate'
import type { ToolId } from '@/shared/tool-registry'

// 拖拽 payload 通过 ref 与 dataTransfer 双轨：
// - ref 给 UI（Dock 显隐、卡片半透等）订阅；
// - dataTransfer 满足 HTML5 拖放协议，保证 drop 跨容器正常。
export type DragPayload = {
  type: 'rule' | 'skill'
  id: number
  name: string
}

const DATA_KEY = 'application/x-vt-binding'

const dragState = ref<DragPayload | null>(null)
const dockExpanded = ref(false)
const hoverProjectId = ref<number | null>(null)
const pulseProjectId = ref<number | null>(null)

let pulseTimer: number | null = null
let payloadTimer: number | null = null
let lastPayload: DragPayload | null = null

function pulseTarget(projectId: number) {
  pulseProjectId.value = projectId
  if (pulseTimer !== null) window.clearTimeout(pulseTimer)
  pulseTimer = window.setTimeout(() => {
    pulseProjectId.value = null
    pulseTimer = null
  }, 220)
}

function normalizePayload(value: unknown): DragPayload | null {
  if (!value || typeof value !== 'object') return null
  const payload = value as Partial<DragPayload>
  if (payload.type !== 'rule' && payload.type !== 'skill') return null
  if (typeof payload.id !== 'number' || !Number.isFinite(payload.id)) return null
  if (typeof payload.name !== 'string' || !payload.name.trim()) return null
  return { type: payload.type, id: payload.id, name: payload.name }
}

function parsePayload(raw: string | undefined): DragPayload | null {
  if (!raw) return null
  try {
    return normalizePayload(JSON.parse(raw))
  } catch {
    return null
  }
}

export function useDragBinding() {
  const projectsStore = useProjectsStore()
  const toolContextStore = useToolContextStore()

  function beginPayload(payload: DragPayload) {
    dragState.value = payload
    lastPayload = payload
    if (payloadTimer !== null) {
      window.clearTimeout(payloadTimer)
      payloadTimer = null
    }
    dockExpanded.value = true
  }

  function begin(payload: DragPayload, event: DragEvent) {
    beginPayload(payload)
    if (event.dataTransfer) {
      const serialized = JSON.stringify(payload)
      event.dataTransfer.effectAllowed = 'link'
      event.dataTransfer.setData(DATA_KEY, serialized)
      event.dataTransfer.setData('text/plain', serialized)
    }
  }

  function setHoverProject(id: number | null) {
    hoverProjectId.value = id
  }

  function end() {
    dragState.value = null
    hoverProjectId.value = null
    if (payloadTimer !== null) window.clearTimeout(payloadTimer)
    payloadTimer = window.setTimeout(() => {
      lastPayload = null
      payloadTimer = null
    }, 1200)
    // 拖拽结束后 Dock 不立即收起，保留一会儿便于多次连续拖拽。
    window.setTimeout(() => {
      if (!dragState.value) dockExpanded.value = false
    }, 400)
  }

  function expand() {
    dockExpanded.value = true
  }

  function collapse() {
    dockExpanded.value = false
  }

  function readPayloadFromEvent(event: DragEvent): DragPayload | null {
    return parsePayload(event.dataTransfer?.getData(DATA_KEY))
      ?? parsePayload(event.dataTransfer?.getData('text/plain'))
      ?? dragState.value
      ?? lastPayload
  }

  async function bindPayloadToProject(projectId: number, payload: DragPayload | null) {
    end()
    if (!payload) return

    const project = projectsStore.items.find((p) => p.id === projectId)
    if (!project) return

    if (payload.type === 'skill') {
      notifyInfo(translateKey('dock.skillSoon'))
      return
    }

    // rule -> project：复用现有 saveProjectRuleIdsAndSync，不新增后端接口。
    const currentRuleIds = project.ruleBindings
      .find((b) => b.toolId == null)
      ?.items.filter((i) => i.itemType === 'rule')
      .map((i) => i.assetId) ?? []

    if (currentRuleIds.includes(payload.id)) {
      notifyWarning(translateKey('dock.alreadyBound', { rule: payload.name, project: project.name }))
      return
    }

    const next = [...currentRuleIds, payload.id]
    const activeToolId = toolContextStore.activeToolId as ToolId
    const result = await projectsStore.saveProjectRuleIdsAndSync(projectId, next, activeToolId, {
      notify: false,
      refreshRules: true,
      refreshSnapshot: true,
      scanActiveProject: false,
    })

    if (result === 'failed') {
      notifyError(projectsStore.workflowError || translateKey('errors.projectRuleBindingsFailed'))
      return
    }

    notifySuccess(translateKey('dock.boundToast', { rule: payload.name, project: project.name }))
    pulseTarget(projectId)
  }

  async function dropOnProject(projectId: number, event: DragEvent) {
    await bindPayloadToProject(projectId, readPayloadFromEvent(event))
  }

  return {
    dragState,
    dockExpanded,
    hoverProjectId,
    pulseProjectId,
    begin,
    beginPayload,
    setHoverProject,
    end,
    expand,
    collapse,
    bindPayloadToProject,
    dropOnProject,
  }
}
