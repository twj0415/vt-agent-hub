<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import Sortable from 'sortablejs'
import { ImportOutlined, PlusOutlined } from '@ant-design/icons-vue'
import EmptyState from '@/shared/components/feedback/EmptyState.vue'
import PageHeader from '@/shared/components/shell/PageHeader.vue'
import { useDragBinding } from '@/shared/composables/useDragBinding'
import DetailDrawer from './components/DetailDrawer.vue'
import BindModal from './components/BindModal.vue'
import FormModal from './components/FormModal.vue'
import ImportModal from './components/ImportModal.vue'
import RuleCard from './components/RuleCard.vue'
import { useRulesWorkbench } from './composables/useRulesWorkbench'

const { ruleCards, ruleListBusy, ruleStore, t } = useRulesWorkbench()
const ruleListRef = ref<HTMLElement | null>(null)
const { beginPayload, bindPayloadToProject, end, setHoverProject } = useDragBinding()
let sortable: Sortable | null = null
let lastPointer = { x: 0, y: 0 }

function projectIdFromPoint(point: { x: number; y: number }) {
  const elements = document.elementsFromPoint(point.x, point.y)
  for (const element of elements) {
    const target = element instanceof HTMLElement ? element.closest<HTMLElement>('[data-project-id]') : null
    const id = Number(target?.dataset.projectId)
    if (Number.isFinite(id) && id > 0) return id
  }
  return null
}

function syncHoverProject() {
  setHoverProject(projectIdFromPoint(lastPointer))
}

function trackPointer(event: PointerEvent) {
  lastPointer = { x: event.clientX, y: event.clientY }
  syncHoverProject()
}

function styleDragPreview(element: HTMLElement) {
  element.style.width = '240px'
  element.style.maxWidth = '240px'
  element.style.minWidth = '240px'
  element.style.borderRadius = '14px'
  element.style.overflow = 'hidden'
  element.style.opacity = '0.82'
  element.style.boxShadow = '0 18px 42px rgb(0 0 0 / 0.18)'
  element.querySelectorAll<HTMLElement>('[data-no-drag], [data-rule-meta]').forEach((child) => {
    child.style.display = 'none'
  })
}

function cleanupDragArtifacts() {
  document.querySelectorAll('.sortable-fallback, .sortable-ghost, .sortable-drag').forEach((element) => {
    element.classList.remove('sortable-ghost', 'sortable-drag')
    if (element.classList.contains('sortable-fallback')) element.remove()
  })
}

function cleanupDragArtifactsSoon() {
  cleanupDragArtifacts()
  window.setTimeout(cleanupDragArtifacts, 0)
  window.setTimeout(cleanupDragArtifacts, 120)
}

function bindSortable() {
  sortable?.destroy()
  sortable = null
  cleanupDragArtifacts()
  if (!ruleListRef.value) return

  sortable = Sortable.create(ruleListRef.value, {
    animation: 150,
    sort: false,
    forceFallback: true,
    fallbackOnBody: true,
    fallbackTolerance: 6,
    touchStartThreshold: 6,
    chosenClass: 'opacity-60',
    ghostClass: 'bg-accent/[0.08]',
    dragClass: 'shadow-surface-lg',
    handle: '[data-drag-handle]',
    filter: '[data-no-drag]',
    preventOnFilter: false,
    onStart(event) {
      const item = ruleCards.value[event.oldIndex ?? -1]
      if (!item) return
      beginPayload({ type: 'rule', id: item.id, name: item.name })
      requestAnimationFrame(() => {
        const fallback = document.querySelector<HTMLElement>('.sortable-fallback')
        if (fallback) styleDragPreview(fallback)
      })
    },
    onMove() {
      requestAnimationFrame(syncHoverProject)
      const fallback = document.querySelector<HTMLElement>('.sortable-fallback')
      if (fallback) styleDragPreview(fallback)
      return false
    },
    async onEnd(event) {
      const item = ruleCards.value[event.oldIndex ?? -1]
      const projectId = projectIdFromPoint(lastPointer)
      setHoverProject(projectId)
      cleanupDragArtifactsSoon()
      if (item && projectId) {
        await bindPayloadToProject(projectId, { type: 'rule', id: item.id, name: item.name })
        cleanupDragArtifactsSoon()
        return
      }
      end()
      cleanupDragArtifactsSoon()
    },
  })
}

onMounted(() => {
  window.addEventListener('pointermove', trackPointer)
  nextTick(bindSortable)
})

watch(() => ruleCards.value.length, () => {
  nextTick(bindSortable)
})

onBeforeUnmount(() => {
  window.removeEventListener('pointermove', trackPointer)
  sortable?.destroy()
  sortable = null
  cleanupDragArtifacts()
})
</script>

<template>
  <!-- 规则页面：父级只负责工具栏、列表和业务弹窗/抽屉挂载。 -->
  <div class="workbench-page">
    <PageHeader :title="t('pages.rules.title')" :count="ruleCards.length">
      <a-button @click="ruleStore.setImportOpen(true)">
        <template #icon><ImportOutlined /></template>
        {{ t('pages.rules.import') }}
      </a-button>
      <a-button type="primary" @click="ruleStore.openCreate()">
        <template #icon><PlusOutlined /></template>
        {{ t('pages.rules.create') }}
      </a-button>
    </PageHeader>

    <div
      v-if="ruleCards.length"
      class="vt-card overflow-hidden"
    >
      <div ref="ruleListRef" class="divide-y divide-line/40">
        <RuleCard v-for="item in ruleCards" :key="item.id" :item="item" :busy="ruleListBusy" />
      </div>
    </div>

    <EmptyState
      v-else
      :title="'pages.rules.emptyState.title'"
      :description="'pages.rules.emptyState.description'"
      :action-label="'pages.rules.emptyState.cta'"
      @action="ruleStore.openCreate()"
    />
  </div>

  <DetailDrawer />
  <BindModal />
  <FormModal />
  <ImportModal />
</template>
