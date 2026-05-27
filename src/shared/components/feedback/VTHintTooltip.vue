<script setup lang="ts">
import { computed } from 'vue'
import { QuestionCircleOutlined } from '@ant-design/icons-vue'
import { useI18n } from 'vue-i18n'

const props = withDefaults(
  defineProps<{
    // 直接文本（已是终态字符串），优先级低于 textKey。
    text?: string
    // i18n key，存在时优先使用并经过 t() 翻译。
    textKey?: string
    // 提示位置，沿用 ant-design-vue Tooltip 的位置枚举。
    placement?: 'top' | 'bottom' | 'left' | 'right'
    // 12px = 紧凑场景（表单字段标签后），14px = 默认。
    size?: 12 | 14
  }>(),
  {
    placement: 'top',
    size: 14,
  },
)

const { t } = useI18n()

const content = computed(() => (props.textKey ? t(props.textKey) : props.text ?? ''))
</script>

<template>
  <!-- macOS 风问号提示：极低视觉权重，hover 触发说明气泡。 -->
  <a-tooltip
    v-if="content"
    :title="content"
    :placement="placement"
    :mouse-enter-delay="0.15"
    :mouse-leave-delay="0.05"
  >
    <button
      type="button"
      class="inline-flex shrink-0 items-center justify-center text-muted/55 outline-none transition-colors duration-normal ease-standard hover:text-muted focus-visible:text-text"
      :style="{ fontSize: `${size}px` }"
      :aria-label="content"
      @click.stop
    >
      <QuestionCircleOutlined />
    </button>
  </a-tooltip>
</template>
