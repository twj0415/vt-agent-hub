<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { projectTypeCodes } from '@/shared/taxonomy'
import { useProjectsStore } from '@/shared/stores/projects'
import { useDragBinding } from '@/shared/composables/useDragBinding'

const { t } = useI18n()
const projectsStore = useProjectsStore()
const { dragState, dockExpanded, hoverProjectId, pulseProjectId, expand, collapse, setHoverProject, dropOnProject } = useDragBinding()

// macOS 系统色，沿用 ProjectCard 的色映射保持一致性。
const typeColorMap: Record<number, string> = {
  [projectTypeCodes.web]: '#6fb08c',
  [projectTypeCodes.mini]: '#c79243',
  [projectTypeCodes.desktop]: '#a58ac2',
}

function colorOf(typeCode: number) {
  return typeColorMap[typeCode] ?? '#8e8e93'
}

const items = computed(() => projectsStore.items)
const total = computed(() => items.value.length)

const hoverOverProjectId = hoverProjectId

function isBindingDrag(event: DragEvent) {
  const types = event.dataTransfer?.types
  if (!types) return !!dragState.value
  const dragTypes = Array.from(types)
  return dragTypes.includes('application/x-vt-binding') || dragTypes.includes('text/plain') || !!dragState.value
}

function onItemDragOver(event: DragEvent, projectId: number) {
  if (!isBindingDrag(event)) return
  event.preventDefault()
  event.stopPropagation()
  if (event.dataTransfer) event.dataTransfer.dropEffect = 'link'
  setHoverProject(projectId)
}

function onItemDragLeave(projectId: number) {
  if (hoverOverProjectId.value === projectId) setHoverProject(null)
}

function onDockDragOver(event: DragEvent) {
  if (!isBindingDrag(event)) return
  event.preventDefault()
  if (event.dataTransfer) event.dataTransfer.dropEffect = 'link'
}

async function onDockDrop(event: DragEvent) {
  event.preventDefault()
  event.stopPropagation()
  setHoverProject(null)
}

async function onItemDrop(event: DragEvent, projectId: number) {
  event.preventDefault()
  event.stopPropagation()
  setHoverProject(null)
  await dropOnProject(projectId, event)
}

// 监听全局 dragstart：任意可拖拽元素开始拖拽时自动展开 Dock。
// 仅对带 application/x-vt-binding 类型的拖拽响应（即 RuleCard / SkillCard）。
function onGlobalDragStart(event: DragEvent) {
  const types = event.dataTransfer?.types
  if (!types) return
  if (Array.from(types).includes('application/x-vt-binding')) {
    expand()
  }
}

let hoverPillTimer: number | null = null
function onPillEnter() {
  if (hoverPillTimer !== null) window.clearTimeout(hoverPillTimer)
  hoverPillTimer = window.setTimeout(() => expand(), 200)
}
function onPillLeave() {
  if (hoverPillTimer !== null) {
    window.clearTimeout(hoverPillTimer)
    hoverPillTimer = null
  }
}

// Esc 收起 Dock（仅在展开时）。
function onKeydown(event: KeyboardEvent) {
  if (event.key === 'Escape' && dockExpanded.value && !dragState.value) {
    collapse()
  }
}

onMounted(() => {
  window.addEventListener('dragstart', onGlobalDragStart)
  window.addEventListener('keydown', onKeydown)
})

onBeforeUnmount(() => {
  window.removeEventListener('dragstart', onGlobalDragStart)
  window.removeEventListener('keydown', onKeydown)
  if (hoverPillTimer !== null) window.clearTimeout(hoverPillTimer)
})

// 拖拽结束后若用户没 drop 到任何项目，hoverOver 状态也要清。
watch(dragState, (next) => {
  if (!next) setHoverProject(null)
})
</script>

<template>
  <!-- 右下浮动 Project Dock：收起态 8px pill，展开态 268px 列表浮层。 -->
  <div class="pointer-events-none fixed bottom-6 right-0 z-[900]">
    <!-- 收起 pill -->
    <button
      v-show="!dockExpanded"
      type="button"
      class="pointer-events-auto group flex h-16 animate-pulse items-center rounded-l-full border border-r-0 border-accent/35 bg-accent/[0.16] shadow-[0_0_18px_rgb(var(--vt-color-accent)/0.16)] transition-all duration-slow ease-spring hover:h-20 hover:bg-accent/[0.22] hover:shadow-[0_0_26px_rgb(var(--vt-color-accent)/0.24)]"
      :style="{ width: '8px' }"
      :title="t('dock.title')"
      :aria-label="t('dock.title')"
      @mouseenter="onPillEnter"
      @mouseleave="onPillLeave"
      @click="expand"
    >
      <span
        class="ml-[-22px] hidden h-5 min-w-[20px] items-center justify-center rounded-full bg-accent px-1 text-[10px] font-semibold text-white shadow-[0_1px_2px_rgb(0_0_0/0.06)] group-hover:flex"
      >
        {{ total }}
      </span>
    </button>

    <!-- 展开浮层 -->
    <Transition
      enter-active-class="transition-all duration-slow ease-spring"
      leave-active-class="transition-all duration-normal ease-standard"
      enter-from-class="translate-x-full opacity-0"
      leave-to-class="translate-x-full opacity-0"
    >
      <div
        v-if="dockExpanded"
        class="pointer-events-auto flex flex-col overflow-hidden rounded-l-[18px] border border-r-0 border-line/55 bg-panel-strong/94 shadow-surface-lg"
        :style="{ width: '268px', maxHeight: 'min(560px, 70vh)', backdropFilter: 'blur(20px) saturate(180%)' }"
        @dragover="onDockDragOver"
        @dragenter="onDockDragOver"
        @drop="onDockDrop"
      >
        <header
          class="flex h-11 shrink-0 items-center justify-between border-b border-line/40 px-4"
        >
          <div class="flex items-center gap-2">
            <span class="text-[13px] font-semibold tracking-[-0.005em] text-text">
              {{ t('dock.title') }}
            </span>
            <span class="text-[11px] text-muted/70">{{ total }}</span>
          </div>
          <button
            type="button"
            class="flex h-6 w-6 items-center justify-center rounded-full text-[10px] text-muted transition-colors duration-normal ease-standard hover:bg-text/[0.06] hover:text-text"
            :aria-label="t('dock.closeAria')"
            @click="collapse"
          >
            <svg viewBox="0 0 10 10" class="h-3 w-3" fill="none">
              <path d="M2 2l6 6M8 2L2 8" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
            </svg>
          </button>
        </header>

        <div v-if="dragState" class="shrink-0 animate-pulse border-b border-accent/20 bg-accent/[0.1] px-4 py-2 text-[11px] font-medium text-text/90">
          {{ t('dock.dropHint', { name: dragState.name }) }}
        </div>

        <div v-if="!total" class="flex flex-1 items-center justify-center px-4 py-8 text-center text-[12px] text-muted/70">
          {{ t('dock.empty') }}
        </div>

        <div v-else class="min-h-0 flex-1 overflow-y-auto overflow-x-hidden">
          <div
            v-for="project in items"
            :key="project.id"
            :data-project-id="project.id"
            role="button"
            tabindex="0"
            class="relative flex w-full items-center gap-3 border-b border-line/30 px-4 py-3 text-left transition-all duration-fast ease-standard last:border-b-0"
            :class="[
              hoverOverProjectId === project.id
                ? 'bg-accent/[0.18] ring-2 ring-inset ring-accent/55 shadow-[inset_0_0_0_1px_rgb(var(--vt-color-accent)/0.2)]'
                : 'hover:bg-text/[0.045]',
              pulseProjectId === project.id ? 'animate-pulse' : '',
            ]"
            @dragover="onItemDragOver($event, project.id)"
            @dragenter="onItemDragOver($event, project.id)"
            @dragleave="onItemDragLeave(project.id)"
            @drop="onItemDrop($event, project.id)"
          >
            <span
              class="absolute bottom-0 left-0 top-0 transition-all duration-fast ease-standard"
              :style="{
                background: colorOf(project.projectType),
                width: hoverOverProjectId === project.id ? '6px' : '4px',
              }"
              aria-hidden="true"
            />
            <div class="min-w-0 flex-1 pl-1">
              <div class="truncate text-[13px] font-medium text-text">{{ project.name }}</div>
              <div class="mt-0.5 truncate font-mono text-[11px] leading-4 text-muted/75">{{ project.path }}</div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>
