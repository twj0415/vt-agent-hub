<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
import DetailSection from './DetailSection.vue'

const props = defineProps<{
  tabs: Array<{
    key: string
    label: string
  }>
  activeKey: string
}>()

const emit = defineEmits<{
  'update:activeKey': [value: string]
}>()

const navRef = ref<HTMLElement | null>(null)
const indicatorLeft = ref(0)
const indicatorWidth = ref(0)
let resizeObserver: ResizeObserver | null = null
let frame = 0

function updateIndicator() {
  if (frame) cancelAnimationFrame(frame)
  frame = requestAnimationFrame(() => {
    const nav = navRef.value
    if (!nav) return

    const activeButton = Array.from(nav.querySelectorAll<HTMLElement>('.detail-tab-button')).find((button) => button.dataset.tabKey === props.activeKey)
    if (!activeButton) return

    indicatorLeft.value = activeButton.offsetLeft
    indicatorWidth.value = activeButton.offsetWidth
    activeButton.scrollIntoView({ block: 'nearest', inline: 'nearest', behavior: 'smooth' })
  })
}

watch(
  () => [props.activeKey, props.tabs.length],
  async () => {
    await nextTick()
    updateIndicator()
  },
)

onMounted(() => {
  updateIndicator()
  if (!navRef.value || typeof ResizeObserver === 'undefined') return
  resizeObserver = new ResizeObserver(updateIndicator)
  resizeObserver.observe(navRef.value)
})

onBeforeUnmount(() => {
  if (frame) cancelAnimationFrame(frame)
  resizeObserver?.disconnect()
})
</script>

<template>
  <DetailSection fill :padded="false">
    <template #header>
      <nav ref="navRef" class="detail-tab-nav relative inline-flex max-w-full items-center gap-0.5 overflow-x-auto rounded-[9px] border border-line/50 bg-bg/50 p-[3px]" aria-label="Detail tabs">
        <span
          class="absolute inset-y-[3px] left-0 z-0 rounded-md border border-line/40 bg-panel-strong shadow-surface transition-[transform,width] duration-200 ease-out motion-reduce:transition-none"
          :style="{ width: `${indicatorWidth}px`, transform: `translateX(${indicatorLeft}px)` }"
          aria-hidden="true"
        />
        <button
          v-for="tab in tabs"
          :key="tab.key"
          type="button"
          class="detail-tab-button relative z-10 h-7 min-w-[54px] whitespace-nowrap rounded-lg border border-transparent bg-transparent px-3 text-[13px] font-semibold leading-[26px] text-muted transition hover:text-text"
          :data-tab-key="tab.key"
          :class="{ 'text-text': activeKey === tab.key }"
          :aria-pressed="activeKey === tab.key"
          @click="emit('update:activeKey', tab.key)"
        >
          {{ tab.label }}
        </button>
      </nav>
    </template>

    <div class="flex min-h-0 flex-1 flex-col overflow-hidden p-2">
      <Transition
        mode="out-in"
        enter-active-class="transition duration-150 ease-out motion-reduce:transition-none"
        enter-from-class="translate-y-[3px] opacity-0"
        enter-to-class="translate-y-0 opacity-100"
        leave-active-class="transition duration-150 ease-in motion-reduce:transition-none"
        leave-from-class="translate-y-0 opacity-100"
        leave-to-class="-translate-y-0.5 opacity-0"
      >
        <div :key="activeKey" class="flex min-h-0 flex-1 flex-col overflow-hidden">
          <slot />
        </div>
      </Transition>
    </div>
  </DetailSection>
</template>

<style scoped>
.detail-tab-nav {
  scrollbar-width: none;
}
.detail-tab-nav::-webkit-scrollbar {
  display: none;
}
</style>
