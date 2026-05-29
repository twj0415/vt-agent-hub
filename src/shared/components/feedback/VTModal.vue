<script setup lang="ts">
  defineProps<{
    open: boolean;
    title: string;
    okText: string;
    cancelText: string;
    loading?: boolean;
    width?: number;
    bodyStyle?: Record<string, string>;
    okButtonProps?: Record<string, unknown>;
    wrapClassName?: string;
    hideFooter?: boolean;
  }>();

  const emit = defineEmits<{
    ok: [];
    close: [];
  }>();
</script>

<template>
  <!-- macOS 风弹窗：圆角 12 + 轻 shadow + 慷慨 padding + 顶部细线分隔标题区。 -->
  <a-modal
    class="app-modal"
    :wrap-class-name="['app-modal-wrap', wrapClassName].filter(Boolean).join(' ')"
    :open="open"
    :title="title"
    :ok-text="okText"
    :cancel-text="cancelText"
    :ok-button-props="okButtonProps"
    v-bind="hideFooter ? { footer: null } : {}"
    :confirm-loading="loading"
    :width="width"
    :body-style="bodyStyle ?? { maxHeight: '72vh', overflowY: 'auto', padding: '22px 24px' }"
    @ok="emit('ok')"
    @cancel="emit('close')"
  >
    <a-spin :spinning="loading">
      <slot />
    </a-spin>

    <template v-if="!hideFooter" #footer>
      <div class="vt-modal-footer-actions">
        <a-button @click="emit('close')">{{ cancelText }}</a-button>
        <a-button type="primary" :loading="loading" v-bind="okButtonProps ?? {}" @click="emit('ok')">
          {{ okText }}
        </a-button>
      </div>
    </template>
  </a-modal>
</template>

<style>
/* 全局：macOS Sheet 风半透 backdrop。 */
.app-modal-wrap .ant-modal-mask {
  backdrop-filter: blur(12px) saturate(140%);
  background: rgba(0, 0, 0, 0.32);
}

.app-modal .ant-modal-content {
  border-radius: var(--vt-radius-lg);
  border: 1px solid rgb(var(--vt-color-line));
}

.app-modal .ant-modal-header {
  border-bottom: 1px solid rgb(var(--vt-color-line) / 0.55);
  padding-bottom: 14px;
  margin-bottom: 0;
}

.app-modal .ant-modal-title {
  font-size: 14px;
  font-weight: 600;
  letter-spacing: -0.005em;
}

.app-modal .ant-modal-footer {
  border-top: 1px solid rgb(var(--vt-color-line) / 0.55);
  padding-top: 14px;
  margin-top: 0;
  background: rgb(var(--vt-color-panel) / 0.6);
  backdrop-filter: blur(10px);
}

.vt-modal-footer-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
</style>
