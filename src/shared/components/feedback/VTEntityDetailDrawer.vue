<script setup lang="ts">
import { computed } from 'vue'
import DetailDrawerLayout from './DetailDrawerLayout.vue'
import DetailSection from './DetailSection.vue'
import DetailTabsSection from './DetailTabsSection.vue'
import StatusBadge from './StatusBadge.vue'
import VTDrawer from './VTDrawer.vue'
import { DETAIL_DRAWER_WIDTH } from '@/shared/constants/layout'
import type { BadgeTone } from '@/shared/types/ui'

// 通用实体详情抽屉：header（名称 + 状态 + 字段表） + tabs + 每个 tab 的内容插槽。
// projects / rules / tools 三个 DetailDrawer 模板骨架完全相同，抽出来减少重复。
export type DetailField = {
  label: string
  value: string
  // 长路径之类的字段用 mono 字体并允许换行。
  mono?: boolean
}

export type DetailHeader = {
  name: string
  status?: {
    tone: BadgeTone
    label: string
  }
  fields?: DetailField[]
}

export type DetailTab = {
  key: string
  label: string
}

const props = withDefaults(
  defineProps<{
    width?: number
    loading?: boolean
    title: string
    header: DetailHeader
    tabs: DetailTab[]
  }>(),
  {
    width: DETAIL_DRAWER_WIDTH,
    loading: false,
  },
)

const open = defineModel<boolean>('open', { required: true })
const activeTab = defineModel<string>('activeTab', { required: true })

const fieldLabelWidth = computed(() => {
  // 根据最长 label 估算左列宽度，保持冒号对齐又不过于宽。
  const maxLen = (props.header.fields ?? []).reduce((acc, f) => Math.max(acc, f.label.length), 0)
  return Math.min(Math.max(maxLen * 14, 40), 96)
})
</script>

<template>
  <VTDrawer v-model:open="open" :title="title" :width="width">
    <DetailDrawerLayout :loading="loading">
      <div class="flex h-full min-h-0 flex-col gap-3 overflow-hidden">
        <DetailSection class="shrink-0">
          <div class="flex min-w-0 flex-wrap items-center gap-3">
            <div class="truncate text-xl font-bold leading-[1.2] text-text">{{ header.name }}</div>
            <StatusBadge v-if="header.status" :tone="header.status.tone" :label="header.status.label" />
            <slot name="header-extra" />
          </div>

          <div v-if="header.fields && header.fields.length" class="mt-4 border-t border-line/70 pt-3">
            <div class="grid gap-y-2">
              <div
                v-for="(field, idx) in header.fields"
                :key="idx"
                class="flex min-w-0 items-baseline gap-3"
              >
                <span
                  class="shrink-0 text-sm font-semibold leading-6 text-muted"
                  :style="{ width: `${fieldLabelWidth}px` }"
                >
                  {{ field.label }}:
                </span>
                <div
                  class="min-w-0 flex-1 text-sm leading-6 text-text"
                  :class="field.mono ? 'break-all font-mono' : ''"
                >
                  {{ field.value }}
                </div>
              </div>
            </div>
          </div>
        </DetailSection>

        <DetailTabsSection v-if="tabs.length" v-model:active-key="activeTab" :tabs="tabs">
          <slot :name="`tab-${activeTab}`" />
        </DetailTabsSection>
      </div>
    </DetailDrawerLayout>
  </VTDrawer>
</template>
