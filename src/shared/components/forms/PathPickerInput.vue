<script setup lang="ts">
import { pickFilePath, pickFolderPath } from '@/shared/api/tauri'

const props = withDefaults(defineProps<{
  value: string
  placeholder?: string
  buttonText: string
  mode?: 'folder' | 'file'
  fileKind?: 'markdown' | 'json' | 'all'
  disabled?: boolean
}>(), {
  mode: 'folder',
  fileKind: 'all',
  placeholder: '',
  disabled: false,
})

const emit = defineEmits<{
  'update:value': [value: string]
  error: [message: string]
}>()

async function choosePath() {
  const response = props.mode === 'folder'
    ? await pickFolderPath()
    : await pickFilePath(props.fileKind)

  if (response.success && response.data) {
    emit('update:value', response.data)
  } else if (!response.success) {
    emit('error', response.error?.message ?? '')
  }
}
</script>

<template>
  <a-input-group compact>
    <a-input
      :value="value"
      :placeholder="placeholder"
      :disabled="disabled"
      style="width: calc(100% - 104px)"
      @update:value="emit('update:value', String($event))"
    />
    <a-button style="width: 104px" :disabled="disabled" @click="choosePath">
      {{ buttonText }}
    </a-button>
  </a-input-group>
</template>
