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
  border-radius: var(--vt-radius-md);
  background: transparent;
  color: rgb(var(--vt-color-muted));
  font-size: 13px;
  outline: none;
  cursor: pointer;
  transition: background-color var(--vt-duration-fast) var(--vt-ease-standard),
    box-shadow var(--vt-duration-fast) var(--vt-ease-standard),
    color var(--vt-duration-fast) var(--vt-ease-standard),
    border-color var(--vt-duration-fast) var(--vt-ease-standard),
    transform var(--vt-duration-fast) var(--vt-ease-standard);
}

.card-icon-btn:hover:not(:disabled) {
  border-color: rgb(var(--vt-color-line-strong) / 0.55);
  background: rgb(var(--vt-color-accent) / 0.08);
  color: rgb(var(--vt-color-accent));
  box-shadow: 0 0 0 1px rgb(var(--vt-color-accent) / 0.15),
    0 2px 8px -2px rgb(var(--vt-color-accent) / 0.20);
}

.card-icon-btn:active:not(:disabled) {
  transform: scale(0.96);
}

.card-icon-btn:focus-visible {
  border-color: rgb(var(--vt-color-accent) / 0.45);
  box-shadow: 0 0 0 3px rgb(var(--vt-color-accent) / 0.12);
}

.card-icon-btn.is-danger:hover:not(:disabled) {
  background: rgb(var(--vt-color-danger) / 0.10);
  color: rgb(var(--vt-color-danger));
  box-shadow: 0 0 0 1px rgb(var(--vt-color-danger) / 0.20),
    0 2px 8px -2px rgb(var(--vt-color-danger) / 0.20);
}

.card-icon-btn.is-disabled,
.card-icon-btn:disabled {
  color: rgb(var(--vt-color-muted) / 0.55);
  opacity: 0.55;
  cursor: not-allowed;
}
</style>
