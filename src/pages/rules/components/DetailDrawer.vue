<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import CodePreview from '@/shared/components/feedback/CodePreview.vue'
import EmptyState from '@/shared/components/feedback/EmptyState.vue'
import StatusBadge from '@/shared/components/feedback/StatusBadge.vue'
import VTEntityDetailDrawer from '@/shared/components/feedback/VTEntityDetailDrawer.vue'
import type { DetailField, DetailHeader, DetailTab } from '@/shared/components/feedback/VTEntityDetailDrawer.vue'
import { useConfirm } from '@/shared/composables/useConfirm'
import { useRuleCards } from '../composables/useRuleCards'
import { useRuleDetail } from '../composables/useRuleDetail'
import { useRuleStore } from '@/shared/stores/rules'
import type { ToolId } from '@/shared/tool-registry'

const { t } = useI18n()
const { confirmAction } = useConfirm()
const ruleStore = useRuleStore()
const { categoryLabel } = useRuleCards()
const { activeRule, detailProjectRelations, detailTab, detailToolRelations, relationActionLoading, unbindProjectRelation, unbindToolRelation } = useRuleDetail()

// 抽屉开关：复用 store 单一来源。
const open = computed({
  get: () => ruleStore.detailOpen,
  set: (value) => ruleStore.setDetailOpen(value),
})

const headerFields = computed<DetailField[]>(() => {
  if (!activeRule.value) return []
  return [
    { label: t('ui.common.category'), value: categoryLabel(activeRule.value.categoryCode) },
    { label: t('ui.common.version'), value: `v${activeRule.value.versionNo}` },
    { label: t('ui.common.description'), value: activeRule.value.summary || t('common.empty') },
  ]
})

const header = computed<DetailHeader>(() => ({
  name: activeRule.value?.name ?? '',
  fields: headerFields.value,
}))

const tabs = computed<DetailTab[]>(() => [
  { key: 'body', label: t('pages.rules.form.body') },
  { key: 'projects', label: t('ui.common.affectedProjects') },
  { key: 'tools', label: t('ui.common.linkedTools') },
])

function confirmUnbindProject(item: { id: number; name: string }) {
  confirmAction({
    danger: true,
    okText: t('pages.projects.binding.unbindAndApply'),
    content: t('pages.projects.binding.unbindConfirmContent', { name: item.name }),
    onOk: () => unbindProjectRelation(item.id),
  })
}

function confirmUnbindTool(item: { id: ToolId; name: string }) {
  confirmAction({
    danger: true,
    okText: t('pages.tools.binding.unbindAndApply'),
    content: t('pages.tools.binding.unbindConfirmContent', { name: item.name }),
    onOk: () => unbindToolRelation(item.id),
  })
}
</script>

<template>
  <VTEntityDetailDrawer
    v-model:open="open"
    v-model:active-tab="detailTab"
    :title="t('pages.rules.drawerTitle')"
    :header="header"
    :tabs="tabs"
  >
    <template #tab-body>
      <div v-if="activeRule?.body.trim()" class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-[12px]">
        <CodePreview :content="activeRule.body" />
      </div>
      <div v-else class="min-h-0 flex-1 overflow-auto">
        <EmptyState size="sm" :description="'common.empty'" />
      </div>
    </template>

    <template #tab-projects>
      <div class="min-h-0 flex-1 overflow-auto">
        <div class="detail-relation-list">
          <article v-for="item in detailProjectRelations" :key="item.id" class="detail-relation-card">
            <div class="detail-relation-main is-plain">
              <div class="detail-relation-content">
                <div class="detail-relation-title-row">
                  <div class="detail-relation-title">{{ item.name }}</div>
                  <StatusBadge :tone="item.statusTone" :label="item.statusLabel" />
                </div>
                <div class="detail-relation-description">{{ item.description }}</div>
              </div>
            </div>
            <div class="detail-relation-actions">
              <a-button
                danger
                type="text"
                size="small"
                :loading="relationActionLoading"
                @click="confirmUnbindProject(item)"
              >
                {{ t('common.unbind') }}
              </a-button>
            </div>
          </article>
          <EmptyState v-if="!detailProjectRelations.length" size="sm" :description="'common.empty'" />
        </div>
      </div>
    </template>

    <template #tab-tools>
      <div class="min-h-0 flex-1 overflow-auto">
        <div class="detail-relation-list">
          <article v-for="item in detailToolRelations" :key="item.id" class="detail-relation-card">
            <div class="detail-relation-main is-plain">
              <div class="detail-relation-content">
                <div class="detail-relation-title-row">
                  <div class="detail-relation-title">{{ item.name }}</div>
                  <StatusBadge :tone="item.statusTone" :label="item.statusLabel" />
                </div>
                <div class="detail-relation-description">{{ item.description }}</div>
              </div>
            </div>
            <div class="detail-relation-actions">
              <a-button
                danger
                type="text"
                size="small"
                :loading="relationActionLoading"
                @click="confirmUnbindTool(item)"
              >
                {{ t('common.unbind') }}
              </a-button>
            </div>
          </article>
          <EmptyState v-if="!detailToolRelations.length" size="sm" :description="'common.empty'" />
        </div>
      </div>
    </template>
  </VTEntityDetailDrawer>
</template>
