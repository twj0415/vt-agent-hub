<script setup lang="ts">
import PageSection from '@/shared/components/shell/PageSection.vue'
import PageHeader from '@/shared/components/shell/PageHeader.vue'
import HealthBadge from '@/shared/components/feedback/HealthBadge.vue'
import VTDrawer from '@/shared/components/feedback/VTDrawer.vue'
import BindingMatrix from '@/shared/components/forms/BindingMatrix.vue'
import EmptyState from '@/shared/components/feedback/EmptyState.vue'
import DiffViewer from '@/shared/components/forms/DiffViewer.vue'
import { healthTone } from '@/shared/constants/status'
import { useHistoryCenter } from './composables/useHistoryCenter'

const {
  t,
  activeTab,
  historyStore,
  tabOptions,
  summaryRows,
  openHistoryTarget,
  resolveHistoryTarget,
  relatedAssetLabel,
} = useHistoryCenter()
</script>

<template>
  <div class="flex min-h-full flex-col gap-5">
    <PageHeader :title="t('pages.history.title')" :count="historyStore.filteredItems.length">
      <a-button
        v-if="activeTab === 'diagnostics'"
        type="button"
               @click="historyStore.exportDiagnostics()"
      >
        {{ t('pages.history.actions.exportDiagnostics') }}
      </a-button>
    </PageHeader>

    <section class="vt-card p-5">
      <div class="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <div
          v-for="row in summaryRows"
          :key="row.label"
          class="rounded-vt-md border border-line bg-surface-2/40 p-4"
        >
          <div class="text-[10px] font-semibold uppercase tracking-[0.14em] text-muted/80">{{ row.label }}</div>
          <div class="mt-3 text-2xl font-semibold text-text">{{ row.value }}</div>
        </div>
      </div>
    </section>

    <section class="vt-card p-3">
      <a-segmented
        v-model:value="activeTab"
        block
        size="large"
        :options="tabOptions.map((tab) => ({ label: tab.label, value: tab.key }))"
      />
      <div class="hidden">
        <a-button
          v-for="tab in tabOptions"
          :key="tab.key"
          @click="activeTab = tab.key"
        >
          {{ tab.label }}
        </a-button>
      </div>
    </section>

    <PageSection
      v-if="activeTab === 'records'"
      title-key="pages.history.timelineTitle"
      hide-header
    >
      <div class="mb-4 grid gap-3 md:grid-cols-4">
        <a-select :value="historyStore.activeProjectFilter" @update:value="historyStore.setProjectFilter($event === 'all' ? 'all' : Number($event))">
          <a-select-option value="all">{{ t('pages.history.filters.allProjects') }}</a-select-option>
          <a-select-option v-for="id in historyStore.filters.projectIds" :key="id" :value="String(id)">{{ t('pages.history.buttons.project', { id }) }}</a-select-option>
        </a-select>
        <a-select :value="historyStore.activeToolFilter" @update:value="historyStore.setToolFilter($event === 'all' ? 'all' : Number($event))">
          <a-select-option value="all">{{ t('pages.history.filters.allTools') }}</a-select-option>
          <a-select-option v-for="id in historyStore.filters.toolIds" :key="id" :value="String(id)">{{ t('pages.history.buttons.tool', { id }) }}</a-select-option>
        </a-select>
        <a-select :value="historyStore.activeKindFilter" @update:value="historyStore.setKindFilter(String($event))">
          <a-select-option value="all">{{ t('pages.history.filters.allKinds') }}</a-select-option>
          <a-select-option v-for="kind in historyStore.filters.kinds" :key="kind" :value="kind">{{ kind }}</a-select-option>
        </a-select>
        <a-select :value="historyStore.activeResultFilter" @update:value="historyStore.setResultFilter(String($event))">
          <a-select-option value="all">{{ t('pages.history.filters.allResults') }}</a-select-option>
          <a-select-option v-for="result in historyStore.filters.results" :key="result" :value="result">{{ result }}</a-select-option>
        </a-select>
      </div>

      <EmptyState
        v-if="!historyStore.filteredItems.length"
        size="sm"
        :title="'pages.history.emptyState.records.title'"
        :description="'pages.history.emptyState.records.description'"
      />

      <div v-else class="space-y-3">
        <div
          v-for="item in historyStore.filteredItems"
          :key="item.id"
          class="vt-card vt-card-hover p-4"
          role="button"
          tabindex="0"
          @click="historyStore.openHistoryDetail(item.id)"
        >
          <div class="flex items-center justify-between gap-3">
            <div class="text-sm font-semibold text-text">{{ item.title }}</div>
            <HealthBadge :level="item.level" />
          </div>
          <div class="mt-2 text-xs text-muted">{{ item.createdAt }} / {{ item.action }} / {{ item.result }}</div>
          <div class="mt-3 text-sm leading-6 text-muted">{{ item.detail }}</div>
          <div class="mt-3 flex flex-wrap gap-2">
            <a-button v-if="item.projectId" size="small" class="!rounded-full" @click.stop="openHistoryTarget(resolveHistoryTarget(item))">{{ t('pages.history.buttons.project', { id: item.projectId }) }}</a-button>
            <a-button v-if="item.toolId" size="small" class="!rounded-full" @click.stop="openHistoryTarget(resolveHistoryTarget(item))">{{ t('pages.history.buttons.tool', { id: item.toolId }) }}</a-button>
            <a-button v-if="item.relatedRuleId" size="small" class="!rounded-full" @click.stop="openHistoryTarget(resolveHistoryTarget(item))">{{ relatedAssetLabel(item) }}</a-button>
            <a-button size="small" class="!rounded-full" @click.stop="openHistoryTarget(resolveHistoryTarget(item))">{{ resolveHistoryTarget(item) }}</a-button>
          </div>
        </div>
      </div>
    </PageSection>

    <PageSection
      v-else-if="activeTab === 'backups'"
      title-key="pages.history.backupTitle"
      hide-header
    >
      <EmptyState
        v-if="!historyStore.backupEntries.length"
        size="sm"
        :title="'pages.history.emptyState.backups.title'"
        :description="'pages.history.emptyState.backups.description'"
      />
      <div v-else class="space-y-3">
        <div
          v-for="item in historyStore.backupEntries"
          :key="item.id"
          class="vt-card p-4"
        >
          <div class="flex items-center justify-between gap-3">
            <div>
              <div class="text-sm font-semibold text-text">{{ item.fileName }}</div>
              <div class="mt-2 text-xs text-muted">{{ item.createdAt }}</div>
            </div>
            <div class="flex flex-wrap gap-2">
              <a-button size="small" class="!rounded-full" @click="historyStore.openRestorePreview(item.id)">{{ t('pages.history.actions.previewRestore') }}</a-button>
              <a-button size="small" danger class="!rounded-full" @click="historyStore.removeBackup(item.id)">{{ t('common.delete') }}</a-button>
            </div>
          </div>
          <div class="mt-3 text-sm leading-6 text-muted">{{ item.scope }} -> {{ item.targetPath }}</div>
          <div class="mt-2 text-xs text-muted">{{ item.path }}</div>
        </div>
      </div>
    </PageSection>

    <PageSection
      v-else
      title-key="pages.history.diagnosticTitle"
      hide-header
    >
      <EmptyState
        v-if="!historyStore.libraryDiagnostics"
        size="sm"
        :title="'pages.history.emptyState.diagnostics.title'"
        :description="'pages.history.emptyState.diagnostics.description'"
      />
      <template v-else>
        <BindingMatrix
          :rows="[
            { label: t('pages.history.summary.projects'), value: String(historyStore.libraryDiagnostics.projectCount) },
            { label: t('pages.history.summary.rules'), value: String(historyStore.libraryDiagnostics.ruleCount) },
            { label: t('pages.history.summary.skills'), value: String(historyStore.libraryDiagnostics.skillCount) },
            { label: t('pages.history.summary.issues'), value: String(historyStore.libraryDiagnostics.issueCount), tone: historyStore.libraryDiagnostics.issueCount ? 'warning' : 'ready' },
            { label: t('pages.history.summary.health'), value: historyStore.libraryDiagnostics.healthState, tone: healthTone(historyStore.libraryDiagnostics.healthStateCode) },
          ]"
        />
        <BindingMatrix v-if="historyStore.diagnosticRows.length" class="mt-4" :rows="historyStore.diagnosticRows" />
      </template>
    </PageSection>

    <VTDrawer
      :open="historyStore.detailOpen"
      :title="t('pages.history.detail.title')"
      :width="560"
      @update:open="historyStore.setDetailOpen"
    >
      <div v-if="historyStore.activeItem" class="space-y-4">
        <BindingMatrix
          :rows="[
            { label: t('pages.history.detail.title'), value: historyStore.activeItem.title },
            { label: t('pages.history.detail.action'), value: historyStore.activeItem.action },
            { label: t('pages.history.detail.result'), value: historyStore.activeItem.result, tone: historyStore.activeItem.result === 'success' ? 'ready' : 'error' },
            { label: t('pages.history.detail.project'), value: historyStore.activeItem.projectId ? String(historyStore.activeItem.projectId) : '-' },
            { label: t('pages.history.detail.tool'), value: historyStore.activeItem.toolId ? String(historyStore.activeItem.toolId) : '-' },
            { label: t('pages.history.detail.rule'), value: historyStore.activeItem.relatedRuleId ? String(historyStore.activeItem.relatedRuleId) : '-' },
            { label: t('pages.history.detail.path'), value: historyStore.activeItem.relatedPath || '-' },
            { label: t('pages.history.detail.target'), value: historyStore.activeItem.navigationTarget || '-' },
          ]"
        />
        <div class="vt-card vt-card-strong p-4 text-[13px] leading-6 text-muted">
          {{ historyStore.activeItem.detail }}
        </div>
      </div>
    </VTDrawer>

    <VTDrawer
      :open="historyStore.previewOpen"
      :title="t('pages.history.detail.backupPreview')"
      :width="640"
      @update:open="historyStore.setPreviewOpen"
    >
      <div v-if="historyStore.restorePreview" class="space-y-4">
        <BindingMatrix
          :rows="[
            { label: t('pages.history.detail.backup'), value: historyStore.restorePreview.backupPath },
            { label: t('pages.history.detail.target'), value: historyStore.restorePreview.targetPath },
            { label: t('pages.history.detail.targetExists'), value: historyStore.restorePreview.targetExists ? t('common.yes') : t('common.no') },
          ]"
        />
        <div v-if="historyStore.restorePreview.warning" class="rounded-vt-md border border-warning/30 bg-warning/8 p-4 text-[13px] leading-6 text-muted">
          {{ historyStore.restorePreview.warning }}
        </div>
        <DiffViewer
          :before="historyStore.restorePreview.beforeContent"
          :after="historyStore.restorePreview.afterContent"
        />
      </div>
      <template #footer>
        <a-button class="!rounded-full" @click="historyStore.setPreviewOpen(false)">{{ t('common.close') }}</a-button>
        <a-button type="primary" class="!rounded-full" @click="historyStore.confirmRestore(true)">{{ t('common.restore') }}</a-button>
      </template>
    </VTDrawer>
  </div>
</template>
