<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import EmptyState from '@/shared/components/feedback/EmptyState.vue'
import StatusBadge from '@/shared/components/feedback/StatusBadge.vue'

function versionLabel(rule: { versionNo: number | null; versionId: number }) {
  return rule.versionNo ? `v${rule.versionNo}` : `#${rule.versionId}`
}

defineProps<{
  rules: Array<{
    id: number
    name: string
    description: string
    categoryLabel: string
    versionNo: number | null
    versionId: number
    hasUpdate: boolean
    latestVersionNo: number | null
  }>
  emptyText: string
  unbindText: string
  loading?: boolean
  disabled?: boolean
}>()

const emit = defineEmits<{
  unbind: [rule: { id: number; name: string }]
}>()

const { t } = useI18n()
</script>

<template>
  <div class="detail-relation-list">
    <article v-for="rule in rules" :key="`${rule.id}-${rule.versionId}`" class="detail-relation-card">
      <div class="detail-relation-main">
        <div class="detail-relation-leading">
          {{ versionLabel(rule) }}
        </div>
        <div class="detail-relation-content">
          <div class="detail-relation-title">{{ rule.name }}</div>
          <div class="detail-relation-description">
            {{ rule.description }}
          </div>
          <div v-if="rule.hasUpdate && rule.latestVersionNo" class="detail-relation-status">
            <StatusBadge
              tone="info"
              :label="t('ui.common.updateAvailable', { version: rule.latestVersionNo })"
            />
          </div>
        </div>
      </div>
      <div class="detail-relation-meta">
        {{ rule.categoryLabel }}
      </div>
      <div class="detail-relation-actions">
        <a-button
          danger
          type="text"
          size="small"
          :loading="loading"
          :disabled="disabled"
          @click="emit('unbind', rule)"
        >
          {{ unbindText }}
        </a-button>
      </div>
    </article>
    <EmptyState v-if="!rules.length" size="sm" :description="emptyText" />
  </div>
</template>
