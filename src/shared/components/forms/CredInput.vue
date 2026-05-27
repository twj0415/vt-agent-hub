<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'

const props = defineProps<{
  label?: string
  labelKey?: string
  modelValue: string
  placeholder?: string
  placeholderKey?: string
}>()

defineEmits<{
  'update:modelValue': [value: string]
}>()

const { t } = useI18n()

const labelText = computed(() => {
  if (props.labelKey) return t(props.labelKey)
  return props.label ?? ''
})

const placeholderText = computed(() => {
  if (props.placeholderKey) return t(props.placeholderKey)
  return props.placeholder ?? ''
})
</script>

<template>
  <a-form layout="vertical">
    <a-form-item :label="labelText" class="!mb-0">
    <a-input-password
      :value="modelValue"
      :placeholder="placeholderText"
      @update:value="$emit('update:modelValue', String($event))"
    />
    </a-form-item>
  </a-form>
</template>
