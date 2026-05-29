<script setup lang="ts">
import { useI18n } from 'vue-i18n'

defineProps<{
  titleKey: string
  descKey?: string
  bodyClass?: string
  hideHeader?: boolean
}>()

const { t } = useI18n()
</script>

<template>
  <section class="rounded-vt-lg border border-line bg-panel p-5">
    <header v-if="!hideHeader" class="mb-5 flex items-start justify-between gap-4">
      <div>
        <h2 class="text-[16px] font-semibold tracking-[-0.018em] text-text">{{ t(titleKey) }}</h2>
        <p v-if="descKey" class="mt-2 max-w-3xl text-[13px] leading-6 text-muted">{{ t(descKey) }}</p>
      </div>
      <slot name="header-extra" />
    </header>
    <div :class="bodyClass">
      <slot />
    </div>
  </section>
</template>
