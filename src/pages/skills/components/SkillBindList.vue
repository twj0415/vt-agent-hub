<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import EmptyState from '@/shared/components/feedback/EmptyState.vue'
import StatusBadge from '@/shared/components/feedback/StatusBadge.vue'

const { t } = useI18n()

function versionLabel(skill: { versionNo: number | null; versionId: number }) {
  return skill.versionNo ? `v${skill.versionNo}` : `#${skill.versionId}`
}

defineProps<{
  skills: Array<{
    id: number
    name: string
    description: string
    categoryLabel: string
    versionNo: number | null
    versionId: number
    healthState: 'normal' | 'abnormal'
  }>
  emptyText: string
  unbindText: string
  loading?: boolean
  disabled?: boolean
}>()

const emit = defineEmits<{
  unbind: [skill: { id: number; name: string }]
}>()
</script>

<template>
  <div class="detail-relation-list">
    <article v-for="skill in skills" :key="`${skill.id}-${skill.versionId}`" class="detail-relation-card">
      <div class="detail-relation-main">
        <div class="detail-relation-leading">
          {{ versionLabel(skill) }}
        </div>
        <div class="detail-relation-content">
          <div class="detail-relation-title-row">
            <div class="detail-relation-title">{{ skill.name }}</div>
            <StatusBadge
              :tone="skill.healthState === 'normal' ? 'ready' : 'error'"
              :label="skill.healthState === 'normal' ? t('common.normal') : t('common.abnormal')"
            />
          </div>
          <div class="detail-relation-description">
            {{ skill.description }}
          </div>
        </div>
      </div>
      <div class="detail-relation-meta">
        {{ skill.categoryLabel }}
      </div>
      <div class="detail-relation-actions">
        <a-button
          danger
          type="text"
          size="small"
          :loading="loading"
          :disabled="disabled"
          @click="emit('unbind', skill)"
        >
          {{ unbindText }}
        </a-button>
      </div>
    </article>
    <EmptyState v-if="!skills.length" size="sm" :description="emptyText" />
  </div>
</template>
