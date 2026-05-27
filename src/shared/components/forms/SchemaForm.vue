<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { FormField } from '@/shared/types/ui'

const props = defineProps<{
  fields: FormField[]
  colProps?: Record<string, number>
}>()

const emit = defineEmits<{
  updateField: [payload: { key: string; value: string }]
}>()

const { t } = useI18n()

const groups = computed(() => {
  const order: string[] = []
  const map = new Map<string, FormField[]>()

  for (const field of props.fields) {
    const key = field.groupKey ?? field.group ?? 'default'
    if (!map.has(key)) {
      map.set(key, [])
      order.push(key)
    }
    map.get(key)?.push(field)
  }

  return order.map((key) => ({
    key,
    label: props.fields.find((field) => (field.groupKey ?? field.group ?? 'default') === key)?.groupKey
      ? t(props.fields.find((field) => (field.groupKey ?? field.group ?? 'default') === key)?.groupKey ?? '')
      : props.fields.find((field) => (field.groupKey ?? field.group ?? 'default') === key)?.group,
    fields: map.get(key) ?? [],
  }))
})

function shouldUseFullWidth(field: FormField, groupSize: number) {
  const key = field.key.toLowerCase()
  return groupSize === 1 || field.type === 'textarea' || key.includes('url') || key.includes('path')
}

function resolveColProps(field: FormField, groupSize: number) {
  if (props.colProps) return props.colProps
  return shouldUseFullWidth(field, groupSize) ? { xs: 24 } : { xs: 24, md: 12 }
}
</script>

<template>
  <div class="grid gap-4">
    <section
      v-for="group in groups"
      :key="group.key"
      class="grid gap-3"
    >
      <header v-if="group.key !== 'default' && group.label" class="border-b border-line pb-2">
        <h3 class="text-sm font-semibold text-text">{{ group.label }}</h3>
      </header>

      <a-row :gutter="[16, 8]">
        <a-col
          v-for="field in group.fields"
          :key="field.key"
          v-bind="resolveColProps(field, group.fields.length)"
        >
          <a-form-item
            :name="field.key"
            :label="field.labelKey ? t(field.labelKey) : field.label"
            class="!mb-2"
          >
            <a-textarea
              v-if="field.type === 'textarea'"
              :value="field.value"
              :placeholder="field.placeholderKey ? t(field.placeholderKey) : field.placeholder"
              :rows="field.rows ?? 4"
              :disabled="field.disabled"
              @update:value="emit('updateField', { key: field.key, value: String($event) })"
            />
            <a-input-password
              v-else-if="field.type === 'password'"
              :value="field.value"
              :placeholder="field.placeholderKey ? t(field.placeholderKey) : field.placeholder"
              :disabled="field.disabled"
              @update:value="emit('updateField', { key: field.key, value: String($event) })"
            />
            <a-select
              v-else-if="field.type === 'select'"
              :value="field.value"
              :placeholder="field.placeholderKey ? t(field.placeholderKey) : field.placeholder"
              :disabled="field.disabled"
              :options="field.options?.map((option) => ({
                label: option.labelKey ? t(option.labelKey) : option.label,
                value: option.value,
              }))"
              @update:value="emit('updateField', { key: field.key, value: String($event) })"
            />
            <a-input
              v-else
              :value="field.value"
              :placeholder="field.placeholderKey ? t(field.placeholderKey) : field.placeholder"
              :disabled="field.disabled"
              @update:value="emit('updateField', { key: field.key, value: String($event) })"
            />
            <template v-if="field.help || field.helpKey" #extra>
              <span class="text-xs leading-5 text-muted">{{ field.helpKey ? t(field.helpKey) : field.help }}</span>
            </template>
          </a-form-item>
        </a-col>
      </a-row>
    </section>
  </div>
</template>
