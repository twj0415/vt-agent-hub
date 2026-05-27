<script setup lang="ts">
defineProps<{
  title?: string
  disabled?: boolean
  danger?: boolean
}>()

const emit = defineEmits<{
  click: [event: MouseEvent]
}>()

// 自绘小圆按钮，macOS Finder hover 风：默认无背景、hover 加 accent 浅底；
// 直接用原生 button 避开 ant-button 默认样式的 !important 战争。
</script>

<template>
  <a-tooltip :title="title" :mouse-enter-delay="0.15">
    <button
      type="button"
      class="card-icon-btn"
      :class="{ 'is-danger': danger, 'is-disabled': disabled }"
      :disabled="disabled"
      @click="emit('click', $event)"
    >
      <slot />
    </button>
  </a-tooltip>
</template>

<style scoped>
.card-icon-btn {
  display: inline-flex;
  height: 28px;
  width: 28px;
  align-items: center;
  justify-content: center;
  padding: 0;
  border: 1px solid transparent;
  border-radius: 9px;
  background: transparent;
  color: rgb(var(--vt-color-muted));
  font-size: 13px;
  outline: none;
  cursor: pointer;
  transition: background-color var(--vt-duration-normal) var(--vt-ease-standard),
    box-shadow var(--vt-duration-normal) var(--vt-ease-standard),
    color var(--vt-duration-normal) var(--vt-ease-standard),
    border-color var(--vt-duration-normal) var(--vt-ease-standard);
}

.card-icon-btn:hover:not(:disabled) {
  border-color: rgb(var(--vt-color-line) / 0.45);
  background: rgb(var(--vt-color-text) / 0.055);
  color: rgb(var(--vt-color-text));
}

.card-icon-btn:focus-visible {
  border-color: rgb(var(--vt-color-accent) / 0.38);
  box-shadow: 0 0 0 3px rgb(var(--vt-color-accent) / 0.1);
}

.card-icon-btn.is-danger:hover:not(:disabled) {
  background: rgb(var(--vt-color-danger) / 0.1);
  color: rgb(var(--vt-color-danger));
}

.card-icon-btn.is-disabled,
.card-icon-btn:disabled {
  color: rgb(var(--vt-color-muted) / 0.55);
  opacity: 0.55;
  cursor: not-allowed;
}
</style>
