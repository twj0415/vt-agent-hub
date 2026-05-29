<script setup lang="ts">
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import AppTopbar from '@/shared/components/shell/AppTopbar.vue'
import AppSidebar from '@/shared/components/shell/AppSidebar.vue'
import VTProjectDock from '@/shared/components/shell/VTProjectDock.vue'

// Project Dock 只在拖拽源（Rules/Skills）出现，其他页面不打扰。
const route = useRoute()
const dockVisible = computed(() => route.name === 'rules' || route.name === 'skills')
</script>

<template>
  <div class="flex h-screen w-screen flex-col overflow-hidden bg-bg text-text">
    <AppTopbar />

    <div class="flex min-h-0 flex-1 bg-bg">
      <AppSidebar />

      <main class="min-h-0 flex-1 overflow-hidden border-l border-line/60 bg-bg">
        <div class="h-full overflow-y-auto px-5 py-4 md:px-6 md:py-5">
          <RouterView />
        </div>
      </main>
    </div>

    <VTProjectDock v-if="dockVisible" />
  </div>
</template>
