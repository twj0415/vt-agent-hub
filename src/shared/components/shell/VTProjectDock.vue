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
      class="dock-pill pointer-events-auto group flex h-16 items-center rounded-l-full border border-r-0 border-accent/40 bg-accent/[0.14] transition-all duration-slow ease-spring hover:h-20 hover:bg-accent/[0.22]"
      :class="dragState ? 'is-dragging' : ''"
      :style="{ width: '8px' }"
      :title="t('dock.title')"
      :aria-label="t('dock.title')"
      @mouseenter="onPillEnter"
      @mouseleave="onPillLeave"
      @click="expand"
    >
      <span
        class="ml-[-22px] hidden h-5 min-w-[20px] items-center justify-center rounded-full bg-accent px-1 text-[10px] font-semibold text-white shadow-glow-accent-soft group-hover:flex"
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
        class="pointer-events-auto flex flex-col overflow-hidden rounded-l-vt-lg border border-r-0 border-line bg-panel-strong/95 shadow-surface-lg"
        :style="{ width: '268px', maxHeight: 'min(560px, 70vh)', backdropFilter: 'blur(22px) saturate(180%)' }"
        @dragover="onDockDragOver"
        @dragenter="onDockDragOver"
        @drop="onDockDrop"
      >
        <header
          class="flex h-10 shrink-0 items-center justify-between border-b border-line/60 px-4"
        >
          <div class="flex items-center gap-2">
            <span class="vt-status-dot vt-status-info" />
            <span class="text-[13px] font-semibold tracking-[-0.005em] text-text">
              {{ t('dock.title') }}
            </span>
            <span class="vt-tag">{{ total }}</span>
          </div>
          <button
            type="button"
            class="flex h-6 w-6 items-center justify-center rounded-vt-sm text-[10px] text-muted transition-colors duration-fast ease-standard hover:bg-text/[0.06] hover:text-text"
            :aria-label="t('dock.closeAria')"
            @click="collapse"
          >
            <svg viewBox="0 0 10 10" class="h-3 w-3" fill="none">
              <path d="M2 2l6 6M8 2L2 8" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
            </svg>
          </button>
        </header>

        <div v-if="dragState" class="shrink-0 border-b border-accent/30 bg-accent/[0.10] px-4 py-2 text-[11px] font-medium text-text/90">
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
            class="dock-item relative flex w-full items-center gap-3 border-b border-line/30 px-4 py-2.5 text-left transition-all duration-fast ease-standard last:border-b-0"
            :class="[
              hoverOverProjectId === project.id
                ? 'is-hover'
                : 'hover:bg-text/[0.045]',
              pulseProjectId === project.id ? 'is-pulse' : '',
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
                width: hoverOverProjectId === project.id ? '4px' : '2px',
                opacity: hoverOverProjectId === project.id ? 1 : 0.75,
              }"
              aria-hidden="true"
            />
            <div class="min-w-0 flex-1 pl-2">
              <div class="truncate text-[13px] font-medium text-text">{{ project.name }}</div>
              <div class="mt-0.5 truncate font-mono text-[11px] leading-4 text-muted/70">{{ project.path }}</div>
            </div>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<style scoped>
.dock-pill {
  box-shadow: 0 0 14px rgb(var(--vt-color-accent) / 0.18);
}
.dock-pill:hover {
  box-shadow: 0 0 22px rgb(var(--vt-color-accent) / 0.28);
}
.dock-pill.is-dragging {
  animation: dock-pill-pulse 1.4s ease-in-out infinite;
  background: rgb(var(--vt-color-accent) / 0.28);
  border-color: rgb(var(--vt-color-accent) / 0.65);
  box-shadow: 0 0 26px rgb(var(--vt-color-accent) / 0.40);
}
@keyframes dock-pill-pulse {
  0%, 100% { transform: scaleY(1); }
  50% { transform: scaleY(1.08); }
}
.dock-item.is-hover {
  background: rgb(var(--vt-color-accent) / 0.16);
  box-shadow: inset 0 0 0 1px rgb(var(--vt-color-accent) / 0.30);
}
.dock-item.is-pulse {
  animation: dock-item-pulse 0.8s ease-out;
}
@keyframes dock-item-pulse {
  0%   { background: rgb(var(--vt-color-success) / 0.30); }
  100% { background: transparent; }
}
</style>
