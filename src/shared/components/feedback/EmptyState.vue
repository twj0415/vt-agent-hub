<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const props = withDefaults(
  defineProps<{
    size?: 'sm' | 'md'
    fill?: boolean
    // 仅一行的旧用法：传 description 作为说明，无标题、无 CTA。
    description?: string
    // 升级用法：title + description 双行，配合 actionLabel 与 #action 插槽。
    title?: string
    actionLabel?: string
  }>(),
  {
    size: 'md',
    fill: true,
    description: 'common.emptyData',
  },
)

const emit = defineEmits<{
  action: []
}>()

const iconSize = computed(() => (props.size === 'sm' ? 36 : 48))

// 优先 title prop，否则不显示标题（保持旧调用方兼容）。
const titleText = computed(() => (props.title ? t(props.title) : ''))
const descriptionText = computed(() => t(props.description))
const actionText = computed(() => (props.actionLabel ? t(props.actionLabel) : ''))
</script>

<template>
  <!-- macOS 风空状态：简笔 SVG 图标 + 标题（可选） + 描述 + 可选 CTA。 -->
  <div
    class="flex flex-col items-center justify-center gap-3 text-center text-muted"
    :class="[
      size === 'sm' ? 'p-5' : 'p-10',
      fill ? 'h-full min-h-[280px] w-full flex-1 self-stretch' : '',
    ]"
  >
    <svg
      :width="iconSize"
      :height="iconSize"
      viewBox="0 0 48 48"
      fill="none"
      class="text-muted/40"
      aria-hidden="true"
    >
      <rect
        x="8"
        y="14"
        width="32"
        height="26"
        rx="3"
        stroke="currentColor"
        stroke-width="1.5"
      />
      <path
        d="M16 8h16l4 6H12l4-6z"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linejoin="round"
      />
      <line
        x1="16"
        y1="26"
        x2="32"
        y2="26"
        stroke="currentColor"
        stroke-width="1.5"
        stroke-linecap="round"
        stroke-dasharray="3 3"
      />
    </svg>

    <div v-if="titleText" class="text-[15px] font-semibold tracking-[-0.005em] text-text">
      {{ titleText }}
    </div>

    <div
      class="max-w-[360px] text-[13px] leading-relaxed text-muted/85"
      :class="size === 'sm' ? 'text-[12px]' : ''"
    >
      {{ descriptionText }}
    </div>

    <!-- 优先用 #action 插槽（页面定制按钮），无 slot 时若有 actionLabel 自动渲染默认按钮。 -->
    <div v-if="$slots.action || actionText" class="mt-1">
      <slot name="action">
        <a-button type="primary" @click="emit('action')">
          {{ actionText }}
        </a-button>
      </slot>
    </div>
  </div>
</template>
