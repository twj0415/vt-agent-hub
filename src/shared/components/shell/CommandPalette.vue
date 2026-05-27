<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useCommandPalette, type CommandGroup, type CommandResult } from '@/shared/composables/useCommandPalette'

const { t } = useI18n()
const { open, query, activeIndex, results, close, selectNext, selectPrev, execute } = useCommandPalette()

const inputRef = ref<HTMLInputElement | null>(null)
const listRef = ref<HTMLDivElement | null>(null)

// 浮层打开时聚焦输入；关闭时清空。
watch(open, async (next) => {
  if (next) {
    await nextTick()
    inputRef.value?.focus()
  }
})

// 高亮项变化时自动滚动到可见区域。
watch(activeIndex, async () => {
  await nextTick()
  const el = listRef.value?.querySelector<HTMLElement>('[data-active="true"]')
  if (el) el.scrollIntoView({ block: 'nearest' })
})

// 按分组渲染：先收集组顺序，再按组分发。
const groupedResults = computed(() => {
  const map = new Map<CommandGroup, CommandResult[]>()
  for (const r of results.value) {
    const list = map.get(r.group) ?? []
    list.push(r)
    map.set(r.group, list)
  }
  return Array.from(map.entries())
})

// 给当前 result 对应到全局索引，用于高亮判断。
function indexOf(result: CommandResult): number {
  return results.value.indexOf(result)
}

const groupTitleKey: Record<CommandGroup, string> = {
  pages: 'commandPalette.groups.pages',
  projects: 'commandPalette.groups.projects',
  rules: 'commandPalette.groups.rules',
  skills: 'commandPalette.groups.skills',
  providers: 'commandPalette.groups.providers',
  tools: 'commandPalette.groups.tools',
}

function onKeydown(event: KeyboardEvent) {
  if (event.key === 'ArrowDown') {
    event.preventDefault()
    selectNext()
  } else if (event.key === 'ArrowUp') {
    event.preventDefault()
    selectPrev()
  } else if (event.key === 'Enter') {
    event.preventDefault()
    void execute()
  } else if (event.key === 'Escape') {
    event.preventDefault()
    close()
  }
}
</script>

<template>
  <!-- macOS Spotlight 风格命令面板：teleport 到 body，居中浮层 + 半透 backdrop。 -->
  <Teleport to="body">
    <Transition
      enter-active-class="transition-all duration-normal ease-spring"
      leave-active-class="transition-all duration-fast ease-standard"
      enter-from-class="opacity-0"
      leave-to-class="opacity-0"
    >
      <div
        v-if="open"
        class="fixed inset-0 z-[1000] flex items-start justify-center px-4"
        style="backdrop-filter: blur(10px) saturate(120%); background: rgba(0, 0, 0, 0.14)"
        @click.self="close"
      >
        <Transition
          enter-active-class="transition-all duration-slow ease-spring"
          leave-active-class="transition-all duration-fast ease-standard"
          enter-from-class="opacity-0 translate-y-[-12px] scale-[0.97]"
          leave-to-class="opacity-0 translate-y-[-8px] scale-[0.98]"
        >
          <div
            v-if="open"
            class="mt-[17vh] flex max-h-[64vh] w-full max-w-[660px] flex-col overflow-hidden rounded-[18px] border border-line/55 bg-panel-strong/92 shadow-surface-lg"
            style="backdrop-filter: blur(20px) saturate(180%)"
          >
            <!-- 输入区：56px 高，无 border 只有底部细线分隔。 -->
            <div class="flex h-14 shrink-0 items-center gap-3 border-b border-line/40 px-5">
              <svg width="16" height="16" viewBox="0 0 16 16" fill="none" class="text-muted/60" aria-hidden="true">
                <circle cx="7" cy="7" r="5" stroke="currentColor" stroke-width="1.4" />
                <path d="M11 11l3 3" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" />
              </svg>
              <input
                ref="inputRef"
                v-model="query"
                type="text"
                :placeholder="t('commandPalette.placeholder')"
                class="h-full flex-1 bg-transparent text-[15px] font-medium text-text outline-none placeholder:text-muted/55"
                @keydown="onKeydown"
              />
              <kbd
                class="hidden shrink-0 rounded border border-line/60 bg-bg px-1.5 py-0.5 text-[10px] font-medium text-muted/70 md:inline-flex"
              >
                Esc
              </kbd>
            </div>

            <!-- 结果区：分组展示，可滚动。 -->
            <div ref="listRef" class="min-h-0 flex-1 overflow-y-auto py-2">
              <div v-if="!results.length" class="px-5 py-8 text-center text-[13px] text-muted/70">
                {{ t('commandPalette.emptyHint') }}
              </div>

              <div v-for="[group, items] in groupedResults" :key="group" class="mb-2 last:mb-0">
                <div class="px-5 py-1 text-[10px] font-semibold uppercase tracking-[0.12em] text-muted/55">
                  {{ t(groupTitleKey[group]) }}
                </div>
                <button
                  v-for="result in items"
                  :key="result.id"
                  type="button"
                  :data-active="indexOf(result) === activeIndex"
                  class="flex w-full items-center gap-3 px-5 py-2 text-left transition-colors duration-fast ease-standard"
                  :class="
                    indexOf(result) === activeIndex
                      ? 'bg-accent/[0.105] text-text shadow-[inset_3px_0_0_rgb(var(--vt-color-accent)/0.75)]'
                      : 'text-text hover:bg-text/[0.045]'
                  "
                  @mouseenter="activeIndex = indexOf(result)"
                  @click="execute(result)"
                >
                  <span class="min-w-0 flex-1">
                    <span class="block truncate text-[13px] font-medium">{{ result.label }}</span>
                    <span
                      v-if="result.hint"
                      class="block truncate font-mono text-[11px] leading-4 text-muted/70"
                    >
                      {{ result.hint }}
                    </span>
                  </span>
                </button>
              </div>
            </div>

            <!-- 底部提示条。 -->
            <div
              class="flex h-8 shrink-0 items-center justify-end gap-3 border-t border-line/35 bg-bg/55 px-4 text-[10px] text-muted/65"
            >
              <span class="flex items-center gap-1">
                <kbd class="rounded border border-line/60 bg-panel-strong px-1 text-[9px]">↑↓</kbd>
                {{ t('commandPalette.hint.navigate') }}
              </span>
              <span class="flex items-center gap-1">
                <kbd class="rounded border border-line/60 bg-panel-strong px-1 text-[9px]">↵</kbd>
                {{ t('commandPalette.hint.open') }}
              </span>
              <span class="flex items-center gap-1">
                <kbd class="rounded border border-line/60 bg-panel-strong px-1 text-[9px]">Esc</kbd>
                {{ t('commandPalette.hint.close') }}
              </span>
            </div>
          </div>
        </Transition>
      </div>
    </Transition>
  </Teleport>
</template>
