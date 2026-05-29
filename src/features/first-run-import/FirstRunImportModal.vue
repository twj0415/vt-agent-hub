<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { EyeInvisibleOutlined, EyeOutlined, ReloadOutlined } from '@ant-design/icons-vue'
import EmptyState from '@/shared/components/feedback/EmptyState.vue'
import { toolRegistry } from '@/shared/tool-registry'
import { useFirstRunImportStore } from '@/shared/stores/first-run-import'
import FirstRunPreferenceSwitch from './FirstRunPreferenceSwitch.vue'
import type { FirstRunImportCandidate, FirstRunImportRoot } from '@/shared/api/client'

type AssetKind = 'rule' | 'skill' | 'provider_preset'
type CategoryKey = 'tools' | AssetKind | 'unsupported'

type CategoryDefinition = {
  kind: CategoryKey
  label: string
}

type CategoryRow = CategoryDefinition & {
  total: number
  selected: number
  selectable: boolean
}

const store = useFirstRunImportStore()
const { t } = useI18n()
const activeCategory = ref<CategoryKey>('tools')
const revealedCredentials = ref<Record<string, boolean>>({})

const roots = computed(() => store.preview?.roots ?? [])
const candidates = computed(() => store.preview?.candidates ?? [])
const selectedSet = computed(() => new Set(store.selectedIds))
const activeCategoryItems = computed(() => itemsForCategory(activeCategory.value))
const activeCategorySelectableItems = computed(() => activeCategoryItems.value.filter((candidate) => candidate.selectable))
const activeCategorySelectedCount = computed(() => activeCategorySelectableItems.value.filter((candidate) => selectedSet.value.has(candidate.id)).length)
const activeCategoryFullySelected = computed(() => activeCategorySelectableItems.value.length > 0 && activeCategorySelectedCount.value === activeCategorySelectableItems.value.length)
const canApply = computed(() => store.selectedCount > 0 && !store.applying)
const hasSelectableInCategory = computed(() => activeCategorySelectableItems.value.length > 0)

const categoryDefinitions = computed<CategoryDefinition[]>(() => [
  { kind: 'tools', label: t('firstRunImport.categories.tools') },
  { kind: 'rule', label: t('firstRunImport.categories.rule') },
  { kind: 'skill', label: t('firstRunImport.categories.skill') },
  { kind: 'provider_preset', label: t('firstRunImport.categories.provider') },
  { kind: 'unsupported', label: t('firstRunImport.categories.unsupported') },
])
const categories = computed<CategoryRow[]>(() => categoryDefinitions.value.map((definition) => {
  const items = itemsForCategory(definition.kind)
  return {
    ...definition,
    total: definition.kind === 'tools' ? roots.value.length : items.length,
    selected: definition.kind === 'tools' ? 0 : items.filter((candidate) => selectedSet.value.has(candidate.id)).length,
    selectable: definition.kind !== 'tools' && definition.kind !== 'unsupported' && items.some((candidate) => candidate.selectable),
  }
}))
const activeCategoryRow = computed(() => categories.value.find((item) => item.kind === activeCategory.value) ?? categories.value[0])

watch(
  () => store.open,
  (open) => {
    if (open) {
      activeCategory.value = 'tools'
      revealedCredentials.value = {}
    }
  },
  { immediate: true },
)

function toolMeta(tool: string) {
  return toolRegistry.find((item) => item.key === tool) ?? null
}

function toolName(tool: string) {
  if (tool === 'claude') return 'Claude'
  if (tool === 'codex') return 'Codex'
  return tool
}

function toolIcon(tool: string) {
  return toolMeta(tool)?.iconSrc ?? ''
}

function toolFallback(tool: string) {
  return toolMeta(tool)?.iconText ?? tool.slice(0, 1).toUpperCase()
}

function rootStatus(root: FirstRunImportRoot) {
  if (!root.exists) return t('firstRunImport.status.missingRoot')
  if (root.candidateCount === 0) return t('firstRunImport.status.emptyRoot')
  return t('firstRunImport.status.detectedRoot', { count: root.candidateCount })
}

function rootSummaryChips(root: FirstRunImportRoot) {
  const scoped = candidates.value.filter((candidate) => candidate.sourceTool === root.tool)
  return [
    { label: t('firstRunImport.categories.rule'), count: scoped.filter((candidate) => candidate.assetType === 'rule').length },
    { label: t('firstRunImport.categories.skill'), count: scoped.filter((candidate) => candidate.assetType === 'skill').length },
    { label: t('firstRunImport.categories.provider'), count: scoped.filter((candidate) => candidate.assetType === 'provider_preset').length },
    { label: t('firstRunImport.categories.unsupportedShort'), count: scoped.filter((candidate) => !candidate.selectable || candidate.status === 'unsupported').length },
  ].filter((item) => item.count > 0)
}

function itemsForCategory(kind: CategoryKey) {
  if (kind === 'tools') return []
  if (kind === 'unsupported') {
    return candidates.value.filter((candidate) => !candidate.selectable || candidate.status === 'unsupported')
  }
  return candidates.value.filter((candidate) => candidate.assetType === kind)
}

function credentialToken(candidate: FirstRunImportCandidate) {
  const value = candidate.metadata.credentialToken
  return typeof value === 'string' && value.trim() ? value.trim() : ''
}

function displayValue(value: string | null | undefined) {
  const text = value?.trim()
  if (!text) return '-'
  if (text === 'firstRunImport.descriptions.initialImport') return t(text)
  return text
}

function credentialDisplay(candidate: FirstRunImportCandidate) {
  const token = credentialToken(candidate)
  if (!token) return '-'
  if (revealedCredentials.value[candidate.id]) return token
  if (token.length <= 10) return '*'.repeat(token.length)
  return `${token.slice(0, 4)}${'*'.repeat(Math.min(12, Math.max(6, token.length - 8)))}${token.slice(-4)}`
}

function toggleCredential(candidateId: string) {
  revealedCredentials.value = {
    ...revealedCredentials.value,
    [candidateId]: !revealedCredentials.value[candidateId],
  }
}

function shouldShowStatus(candidate: FirstRunImportCandidate) {
  return candidate.status !== 'ready'
}

function statusLabel(candidate: FirstRunImportCandidate) {
  if (candidate.status === 'unsupported') return t('firstRunImport.status.unsupported')
  if (candidate.status === 'conflict') return t('firstRunImport.status.conflict')
  if (candidate.status === 'error') return t('firstRunImport.status.error')
  if (candidate.status === 'warning') return t('firstRunImport.status.warning')
  return t('firstRunImport.status.ready')
}

function isCategorySelectable(kind: CategoryKey) {
  const row = categories.value.find((item) => item.kind === kind)
  return !!row?.selectable
}

function setActiveCategory(kind: CategoryKey) {
  activeCategory.value = kind
}

function toggleActiveCategorySelection() {
  if (!hasSelectableInCategory.value) return
  if (activeCategoryFullySelected.value) {
    for (const candidate of activeCategorySelectableItems.value) {
      if (selectedSet.value.has(candidate.id)) store.toggleCandidate(candidate.id)
    }
    return
  }

  for (const candidate of activeCategorySelectableItems.value) {
    if (!selectedSet.value.has(candidate.id)) store.toggleCandidate(candidate.id)
  }
}

function isSelected(id: string) {
  return selectedSet.value.has(id)
}

async function applySelected() {
  await store.applySelected()
}
</script>

<template>
  <div v-if="store.open" class="first-run-guide">
    <div class="guide-shell">
      <header class="guide-topbar">
        <div class="topbar-copy">
          <div class="brand-wordmark">VT Hub</div>
          <h1>{{ t('firstRunImport.title') }}</h1>
          <p v-html="t('firstRunImport.subtitle')" />
        </div>

        <div class="topbar-actions">
          <FirstRunPreferenceSwitch />
          <a-button class="rescan-button" size="small" :loading="store.loading" @click="store.loadPreview(false)">
            <template #icon><ReloadOutlined /></template>
            {{ t('firstRunImport.actions.rescanShort') }}
          </a-button>
        </div>
      </header>

      <main class="guide-main">
        <section class="guide-panel">
          <template v-if="store.error && !store.preview">
            <div class="error-state">
              <div class="error-mark">!</div>
              <div class="error-copy">
                <strong>{{ t('firstRunImport.errorTitle') }}</strong>
                <p>{{ store.error }}</p>
              </div>
            </div>
          </template>

          <template v-else-if="!store.preview">
            <div class="loading-state">
              <a-spin />
              <span>{{ t('firstRunImport.loading') }}</span>
            </div>
          </template>

          <template v-else>
            <div class="catalog-layout">
              <aside class="category-rail">
                <button
                  v-for="category in categories"
                  :key="category.kind"
                  type="button"
                  class="category-button"
                  :class="{ active: activeCategory === category.kind }"
                  @click="setActiveCategory(category.kind)"
                >
                  <span class="category-copy">
                    <strong>{{ category.label }}</strong>
                  </span>
                  <em v-if="category.kind === 'tools'">{{ category.total }}</em>
                  <em v-else-if="category.selectable">{{ category.selected }}/{{ category.total }}</em>
                  <em v-else>{{ category.total }}</em>
                </button>
              </aside>

              <section class="catalog-content">
                <div class="catalog-head">
                  <a-checkbox
                    v-if="activeCategory !== 'tools' && activeCategory !== 'unsupported' && isCategorySelectable(activeCategory)"
                    :checked="activeCategoryFullySelected"
                    :indeterminate="activeCategorySelectedCount > 0 && !activeCategoryFullySelected"
                    @change="toggleActiveCategorySelection"
                  >
                    {{ t('firstRunImport.actions.selectAll') }}
                  </a-checkbox>
                  <span v-else />

                  <div class="head-meta">
                    <span v-if="activeCategory !== 'tools' && activeCategoryRow.selectable" class="head-count">{{ activeCategorySelectedCount }}/{{ activeCategoryRow.total }}</span>
                    <span v-else class="head-count">{{ activeCategoryRow.total }}</span>
                  </div>
                </div>

                <div class="catalog-body">
                  <div v-if="activeCategory === 'tools'" class="source-grid">
                    <article
                      v-for="root in roots"
                      :key="root.tool"
                      class="tool-card"
                      :class="{ missing: !root.exists, empty: root.exists && root.candidateCount === 0 }"
                    >
                      <div class="tool-card-header">
                        <div class="tool-brand">
                          <div class="tool-avatar">
                            <img v-if="toolIcon(root.tool)" :src="toolIcon(root.tool)" :alt="toolName(root.tool)" />
                            <span v-else>{{ toolFallback(root.tool) }}</span>
                          </div>
                          <div class="tool-name">
                            <strong>{{ toolName(root.tool) }}</strong>
                            <small>{{ t('firstRunImport.labels.globalSource') }}</small>
                          </div>
                        </div>
                        <span class="status-chip" :class="{ missing: !root.exists, empty: root.exists && root.candidateCount === 0 }">{{ rootStatus(root) }}</span>
                      </div>

                      <dl class="meta-list">
                        <div class="meta-row">
                          <dt>{{ t('firstRunImport.labels.directory') }}</dt>
                          <dd>{{ root.path }}</dd>
                        </div>
                        <div class="meta-row meta-row-chips">
                          <dt>{{ t('firstRunImport.labels.content') }}</dt>
                          <dd>
                            <span v-if="!root.exists" class="meta-hint">{{ t('firstRunImport.hints.missingRoot') }}</span>
                            <div v-else class="chip-row">
                              <span v-for="chip in rootSummaryChips(root)" :key="`${root.tool}-${chip.label}`" class="meta-chip">
                                {{ chip.label }} {{ chip.count }}
                              </span>
                              <span v-if="rootSummaryChips(root).length === 0" class="meta-hint">{{ t('firstRunImport.hints.emptyRoot') }}</span>
                            </div>
                          </dd>
                        </div>
                      </dl>
                    </article>
                  </div>

                  <div v-else-if="activeCategoryItems.length" class="asset-grid">
                    <article
                      v-for="candidate in activeCategoryItems"
                      :key="candidate.id"
                      class="asset-card"
                      :class="{ selected: isSelected(candidate.id), disabled: !candidate.selectable }"
                      @click="candidate.selectable && store.toggleCandidate(candidate.id)"
                    >
                      <div class="asset-card-top">
                        <span class="asset-tool-logo" :aria-label="t('firstRunImport.labels.sourceWithTool', { tool: toolName(candidate.sourceTool) })">
                          <img v-if="toolIcon(candidate.sourceTool)" :src="toolIcon(candidate.sourceTool)" alt="" />
                          <span v-else>{{ toolFallback(candidate.sourceTool) }}</span>
                        </span>

                        <span class="check" :class="{ checked: isSelected(candidate.id), disabled: !candidate.selectable }">
                          <span v-if="isSelected(candidate.id)">✓</span>
                        </span>

                        <div class="asset-brand">
                          <strong>{{ candidate.name }}</strong>
                        </div>

                        <span v-if="shouldShowStatus(candidate)" class="status-chip" :class="candidate.status">{{ statusLabel(candidate) }}</span>
                      </div>

                      <dl class="asset-meta">
                        <div class="detail-row">
                          <dt>{{ t('firstRunImport.labels.path') }}</dt>
                          <dd>{{ displayValue(candidate.relativePath) }}</dd>
                        </div>
                        <div v-if="candidate.assetType === 'provider_preset'" class="detail-row credential-row">
                          <dt>{{ t('firstRunImport.labels.credential') }}</dt>
                          <dd>
                            <code class="detail-code credential-code">{{ credentialDisplay(candidate) }}</code>
                            <button v-if="credentialToken(candidate)" type="button" class="credential-eye" @click.stop="toggleCredential(candidate.id)">
                              <EyeInvisibleOutlined v-if="revealedCredentials[candidate.id]" />
                              <EyeOutlined v-else />
                            </button>
                          </dd>
                        </div>
                        <div class="detail-row full">
                          <dt>{{ t('firstRunImport.labels.description') }}</dt>
                          <dd>{{ displayValue(candidate.summary) }}</dd>
                        </div>
                      </dl>
                    </article>
                  </div>

                  <div v-else class="empty-wrap">
                    <EmptyState description="firstRunImport.empty" />
                  </div>
                </div>
              </section>
            </div>
          </template>
        </section>
      </main>

      <footer class="guide-actions">
        <div />

        <div class="action-group">
          <a-button @click="store.dismiss()">{{ t('firstRunImport.actions.skipDirect') }}</a-button>
          <a-button type="primary" :loading="store.applying" :disabled="!canApply" @click="applySelected">{{ t('firstRunImport.actions.importSelected') }}</a-button>
        </div>
      </footer>
    </div>
  </div>
</template>

<style scoped>
.first-run-guide {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: grid;
  background:
    radial-gradient(circle at 12% 0%, rgb(var(--vt-color-accent) / 0.13), transparent 32%),
    radial-gradient(circle at 90% 10%, rgb(var(--vt-color-line-strong, var(--vt-color-line)) / 0.22), transparent 26%),
    linear-gradient(180deg, rgb(var(--vt-color-bg) / 0.98), rgb(var(--vt-color-bg)));
  color: rgb(var(--vt-color-text));
}

.guide-shell {
  display: grid;
  width: 100%;
  height: 100%;
  grid-template-rows: auto minmax(0, 1fr) auto;
  overflow: hidden;
  background: rgb(var(--vt-color-panel-strong, var(--vt-color-panel)) / 0.9);
  box-shadow: inset 0 0 0 1px rgb(var(--vt-color-line-strong, var(--vt-color-line)) / 0.42);
  backdrop-filter: blur(22px);
}

.guide-topbar {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 20px;
  border-bottom: 1px solid rgb(var(--vt-color-line-strong, var(--vt-color-line)) / 0.46);
  background: rgb(var(--vt-color-panel-strong, var(--vt-color-panel)) / 0.72);
  padding: 20px 28px 16px;
}

.topbar-copy {
  display: grid;
  gap: 6px;
  min-width: 0;
}

.brand-wordmark {
  color: rgb(var(--vt-color-muted));
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

.topbar-copy h1 {
  margin: 0;
  color: rgb(var(--vt-color-text));
  font-size: 28px;
  font-weight: 780;
  letter-spacing: -0.045em;
}

.topbar-copy p {
  margin: 0;
  color: rgb(var(--vt-color-muted));
  font-size: 13px;
  line-height: 1.6;
}

.topbar-copy :deep(code) {
  border-radius: 7px;
  background: rgb(var(--vt-color-text) / 0.06);
  padding: 2px 6px;
  color: rgb(var(--vt-color-text));
  font-size: 12px;
}

.topbar-actions {
  display: flex;
  flex: 0 0 auto;
  flex-wrap: wrap;
  align-items: center;
  justify-content: flex-end;
  gap: 10px;
}

.rescan-button {
  border-color: rgb(var(--vt-color-accent) / 0.26) !important;
  background: rgb(var(--vt-color-accent) / 0.08) !important;
  color: rgb(var(--vt-color-accent)) !important;
  font-weight: 700;
}

.rescan-button:hover {
  border-color: rgb(var(--vt-color-accent) / 0.4) !important;
  background: rgb(var(--vt-color-accent) / 0.12) !important;
}

.selection-chip,
.meta-chip {
  border: 1px solid rgb(var(--vt-color-line) / 0.58);
  border-radius: 999px;
  background: rgb(var(--vt-color-text) / 0.045);
  padding: 5px 10px;
  color: rgb(var(--vt-color-text));
  font-size: 12px;
  font-weight: 650;
}

.guide-main {
  display: grid;
  min-height: 0;
  padding: 18px 24px 0;
}

.guide-panel {
  display: grid;
  min-height: 0;
  overflow: hidden;
  border: 1px solid rgb(var(--vt-color-line-strong, var(--vt-color-line)) / 0.58);
  border-radius: 24px;
  background: rgb(var(--vt-color-bg) / 0.68);
  box-shadow:
    inset 0 1px 0 rgb(255 255 255 / 0.22),
    0 0 0 1px rgb(var(--vt-color-text) / 0.025);
  padding: 16px;
}

.error-state,
.loading-state,
.empty-wrap {
  min-height: 360px;
}

.error-state,
.loading-state {
  display: grid;
  place-items: center;
  align-content: center;
  gap: 14px;
}

.error-state {
  text-align: center;
}

.error-mark {
  display: grid;
  width: 56px;
  height: 56px;
  place-items: center;
  border-radius: 18px;
  background: rgb(var(--vt-color-danger) / 0.12);
  color: rgb(var(--vt-color-danger));
  font-size: 24px;
  font-weight: 800;
}

.error-copy {
  display: grid;
  gap: 6px;
  max-width: 720px;
}

.error-copy strong {
  color: rgb(var(--vt-color-text));
  font-size: 16px;
  font-weight: 760;
}

.error-copy p,
.meta-hint {
  color: rgb(var(--vt-color-muted));
  font-size: 13px;
  line-height: 1.6;
}

.catalog-layout {
  display: grid;
  height: 100%;
  min-height: 0;
  grid-template-columns: 220px minmax(0, 1fr);
  gap: 18px;
}

.category-rail {
  display: grid;
  min-height: 0;
  align-content: start;
  gap: 10px;
  overflow-y: auto;
  border-right: 1px solid rgb(var(--vt-color-line-strong, var(--vt-color-line)) / 0.5);
  padding-right: 16px;
}

.category-button {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: 12px;
  width: 100%;
  border: 1px solid transparent;
  border-radius: 18px;
  background: transparent;
  padding: 10px 14px;
  text-align: left;
  cursor: pointer;
}

.category-button:hover {
  background: rgb(var(--vt-color-text) / 0.035);
}

.category-button.active {
  border-color: rgb(var(--vt-color-accent) / 0.38);
  background: rgb(var(--vt-color-accent) / 0.085);
  box-shadow:
    inset 0 1px 0 rgb(255 255 255 / 0.16),
    0 0 0 1px rgb(var(--vt-color-accent) / 0.08);
}

.category-copy {
  display: grid;
  gap: 2px;
  min-width: 0;
}

.category-copy strong {
  color: rgb(var(--vt-color-text));
  font-size: 13px;
  font-weight: 730;
}

.category-button em {
  color: rgb(var(--vt-color-muted));
  font-size: 12px;
  font-style: normal;
  font-weight: 700;
}

.catalog-content {
  display: grid;
  min-width: 0;
  min-height: 0;
  grid-template-rows: auto minmax(0, 1fr);
  overflow: hidden;
}

.catalog-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 16px;
  min-height: 34px;
  border-bottom: 1px solid rgb(var(--vt-color-line-strong, var(--vt-color-line)) / 0.52);
  padding: 0 2px 12px;
}

.head-meta {
  display: flex;
  align-items: center;
  justify-content: flex-end;
}

.head-count {
  color: rgb(var(--vt-color-muted));
  font-size: 12px;
  font-weight: 700;
}

.catalog-body {
  min-height: 0;
  overflow-y: auto;
  padding-top: 14px;
  padding-right: 4px;
}

.source-grid,
.asset-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 12px;
  align-content: start;
}

.tool-card,
.asset-card {
  border: 1px solid rgb(var(--vt-color-line-strong, var(--vt-color-line)) / 0.5);
  border-radius: 22px;
  background: rgb(var(--vt-color-panel-strong, var(--vt-color-panel)) / 0.8);
  box-shadow:
    inset 0 1px 0 rgb(255 255 255 / 0.18),
    0 1px 2px rgb(0 0 0 / 0.035);
}

.tool-card {
  display: grid;
  overflow: hidden;
  gap: 12px;
  padding: 14px;
}

.tool-card.missing,
.tool-card.empty {
  background: rgb(var(--vt-color-text) / 0.03);
}

.tool-card-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: 12px;
}

.tool-brand {
  display: flex;
  align-items: center;
  gap: 12px;
  min-width: 0;
}

.tool-avatar {
  display: grid;
  width: 46px;
  height: 46px;
  flex: 0 0 auto;
  place-items: center;
  overflow: hidden;
  border-radius: 16px;
  background: rgb(var(--vt-color-text) / 0.05);
}

.tool-avatar img {
  width: 28px;
  height: 28px;
  object-fit: contain;
}

.tool-avatar span {
  color: rgb(var(--vt-color-text));
  font-size: 14px;
  font-weight: 760;
}

.tool-name {
  display: grid;
  gap: 2px;
  min-width: 0;
}

.tool-name strong {
  color: rgb(var(--vt-color-text));
  font-size: 15px;
  font-weight: 760;
}

.tool-name small {
  color: rgb(var(--vt-color-muted));
  font-size: 12px;
}

.status-chip {
  justify-self: end;
  border: 1px solid transparent;
  border-radius: 999px;
  background: rgb(var(--vt-color-text) / 0.06);
  padding: 5px 10px;
  color: rgb(var(--vt-color-muted));
  font-size: 11px;
  font-weight: 650;
  white-space: nowrap;
}

.status-chip.conflict {
  background: rgb(var(--vt-color-warning) / 0.12);
  color: rgb(var(--vt-color-warning));
}

.status-chip.error {
  background: rgb(var(--vt-color-danger) / 0.12);
  color: rgb(var(--vt-color-danger));
}

.status-chip.warning {
  background: rgb(var(--vt-color-warning) / 0.12);
  color: rgb(var(--vt-color-warning));
}

.status-chip.unsupported {
  background: rgb(var(--vt-color-muted) / 0.12);
  color: rgb(var(--vt-color-muted));
}

.status-chip.missing,
.status-chip.empty {
  background: rgb(var(--vt-color-text) / 0.035);
  color: rgb(var(--vt-color-muted));
}

.meta-list,
.asset-meta {
  display: grid;
  gap: 10px;
}

.meta-row,
.detail-row {
  display: grid;
  grid-template-columns: 38px minmax(0, 1fr);
  gap: 8px;
  align-items: start;
}

.meta-row dt,
.detail-row dt {
  color: rgb(var(--vt-color-muted));
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.04em;
}

.meta-row dd,
.detail-row dd {
  min-width: 0;
  overflow: hidden;
  color: rgb(var(--vt-color-text));
  font-size: 12px;
  line-height: 1.6;
  text-overflow: ellipsis;
  word-break: break-word;
}

.meta-row-chips dd {
  display: grid;
  gap: 8px;
}

.chip-row {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.asset-card {
  display: grid;
  overflow: hidden;
  gap: 10px;
  padding: 13px;
  cursor: pointer;
}

.asset-card:hover {
  background: rgb(var(--vt-color-text) / 0.035);
}

.asset-card.selected {
  border-color: rgb(var(--vt-color-accent) / 0.56);
  background: rgb(var(--vt-color-accent) / 0.09);
  box-shadow:
    inset 0 1px 0 rgb(255 255 255 / 0.2),
    0 0 0 1px rgb(var(--vt-color-accent) / 0.12);
}

.asset-card.disabled {
  cursor: default;
  opacity: 0.72;
}

.asset-card-top {
  display: grid;
  grid-template-columns: auto auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 10px;
}

.check {
  display: grid;
  width: 20px;
  height: 20px;
  place-items: center;
  border: 1px solid rgb(var(--vt-color-line));
  border-radius: 7px;
  color: white;
  font-size: 12px;
  font-weight: 700;
}

.check.checked {
  border-color: rgb(var(--vt-color-accent));
  background: rgb(var(--vt-color-accent));
}

.check.disabled {
  background: rgb(var(--vt-color-text) / 0.05);
  color: rgb(var(--vt-color-muted));
  cursor: not-allowed;
}

.asset-tool-logo {
  display: grid;
  width: 22px;
  height: 22px;
  place-items: center;
  overflow: hidden;
  border-radius: 8px;
  background: rgb(var(--vt-color-text) / 0.045);
  color: rgb(var(--vt-color-muted));
  font-size: 10px;
  font-weight: 800;
}

.asset-tool-logo img {
  width: 16px;
  height: 16px;
  object-fit: contain;
}

.asset-brand {
  display: inline-flex;
  min-width: 0;
  align-items: center;
}

.asset-brand strong {
  overflow: hidden;
  color: rgb(var(--vt-color-text));
  font-size: 14px;
  font-weight: 760;
  line-height: 1.4;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.detail-row.full dd {
  display: -webkit-box;
  min-height: 38px;
  overflow: hidden;
  line-height: 1.55;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
}

.credential-row dd {
  display: flex;
  align-items: center;
  gap: 8px;
}

.detail-code {
  min-width: 0;
  overflow: hidden;
  border-radius: 10px;
  background: rgb(var(--vt-color-text) / 0.05);
  padding: 5px 8px;
  color: rgb(var(--vt-color-text));
  font-size: 11px;
  line-height: 1.4;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.credential-code {
  flex: 1;
}

.credential-eye {
  display: grid;
  width: 24px;
  height: 24px;
  flex: 0 0 auto;
  place-items: center;
  border: 0;
  border-radius: 8px;
  background: rgb(var(--vt-color-text) / 0.055);
  color: rgb(var(--vt-color-muted));
  cursor: pointer;
}

.credential-eye:hover {
  background: rgb(var(--vt-color-accent) / 0.1);
  color: rgb(var(--vt-color-accent));
}

.empty-wrap {
  display: grid;
  place-items: center;
  align-content: center;
}

.guide-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  border-top: 1px solid rgb(var(--vt-color-line-strong, var(--vt-color-line)) / 0.46);
  padding: 14px 28px 18px;
  background: rgb(var(--vt-color-panel-strong, var(--vt-color-panel)) / 0.64);
  backdrop-filter: blur(12px);
}

.selection-summary {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
  min-width: 0;
}

.selection-label {
  color: rgb(var(--vt-color-muted));
  font-size: 12px;
  font-weight: 700;
  letter-spacing: 0.04em;
  text-transform: uppercase;
}

.selection-summary strong {
  color: rgb(var(--vt-color-text));
  font-size: 18px;
  font-weight: 780;
}

.action-group {
  display: flex;
  align-items: center;
  gap: 10px;
}

@media (max-width: 1080px) {
  .catalog-layout {
    grid-template-columns: 1fr;
  }

  .category-rail {
    grid-template-columns: repeat(2, minmax(0, 1fr));
    overflow: visible;
    border-right: 0;
    border-bottom: 1px solid rgb(var(--vt-color-line-strong, var(--vt-color-line)) / 0.5);
    padding-right: 0;
    padding-bottom: 16px;
  }
}

@media (max-width: 860px) {
  .guide-topbar,
  .guide-actions {
    align-items: flex-start;
    flex-direction: column;
  }

  .topbar-actions,
  .action-group {
    width: 100%;
    flex-wrap: wrap;
    justify-content: flex-start;
  }

  .guide-actions {
    padding-inline: 20px;
  }

  .source-grid,
  .asset-grid,
  .category-rail {
    grid-template-columns: 1fr;
  }

  .guide-main {
    padding-inline: 14px;
  }
}
</style>
