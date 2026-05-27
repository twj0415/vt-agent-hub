<script setup lang="ts">
import { MoreOutlined } from '@ant-design/icons-vue'
import type { CardMoreMenuItem } from '@/shared/types/ui'
import CardIconButton from './CardIconButton.vue'

const props = defineProps<{
  items: CardMoreMenuItem[]
  title?: string
  disabled?: boolean
}>()

const emit = defineEmits<{
  select: [key: string]
}>()

function handleSelect(event: { key: string | number }) {
  if (props.disabled) return
  emit('select', String(event.key))
}
</script>

<template>
  <a-dropdown :trigger="disabled ? [] : ['hover']" placement="bottomRight" :disabled="disabled">
    <CardIconButton :title="title" :disabled="disabled">
      <MoreOutlined />
    </CardIconButton>
    <template #overlay>
      <a-menu class="card-more-menu" @click="handleSelect">
        <a-menu-item
          v-for="item in items"
          :key="item.key"
          :disabled="item.disabled"
          :danger="item.danger"
          class="card-more-menu__entry"
        >
          <span class="card-more-menu__label">{{ item.label }}</span>
        </a-menu-item>
      </a-menu>
    </template>
  </a-dropdown>
</template>

<style scoped>
.card-more-menu__label {
  display: inline-flex;
  align-items: center;
  white-space: nowrap;
}
</style>
