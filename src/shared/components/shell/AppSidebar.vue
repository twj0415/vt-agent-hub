<script setup lang="ts">
  import { computed } from 'vue';
  import { useRoute } from 'vue-router';
  import { useI18n } from 'vue-i18n';
  import { appRoutes } from '@/shared/config/routes';

  interface NavItem {
    key: string;
    labelKey: string;
    to: string;
  }

  interface NavGroup {
    titleKey: string;
    items: NavItem[];
  }

  const navGroups: NavGroup[] = [
    {
      titleKey: 'nav.workbench',
      items: [
        { key: 'projects', labelKey: 'nav.projects', to: appRoutes.projects },
        { key: 'tools', labelKey: 'nav.tools', to: appRoutes.tools },
      ],
    },
    {
      titleKey: 'nav.assets',
      items: [
        { key: 'rules', labelKey: 'nav.rules', to: appRoutes.rules },
        { key: 'skills', labelKey: 'nav.skills', to: appRoutes.skills },
        { key: 'presets', labelKey: 'nav.providers', to: appRoutes.presets },
      ],
    },
  ];

  const footerItems: NavItem[] = [
    { key: 'history', labelKey: 'nav.history', to: appRoutes.history }
  ];

  const route = useRoute();
  const { t } = useI18n();

  const currentKey = computed(() => String(route.name ?? 'projects'));
</script>

<template>
  <aside class="sidebar-shell flex h-full w-[208px] shrink-0 flex-col border-r border-line/60 bg-bg/90 backdrop-blur-xl backdrop-saturate-150">
    <div data-tauri-drag-region class="h-3 shrink-0" />

    <nav class="flex-1 overflow-y-auto px-2.5 pb-3">
      <div v-for="(group, gi) in navGroups" :key="gi" :class="gi > 0 ? 'mt-5' : 'mt-1'">
        <div class="mb-1.5 px-3 text-[10px] font-semibold uppercase leading-4 tracking-[0.10em] text-muted/55">
          {{ t(group.titleKey) }}
        </div>

        <RouterLink
          v-for="item in group.items"
          :key="item.key"
          :to="item.to"
          class="nav-item group relative mb-0.5 flex h-8 items-center rounded-vt-md pl-3 pr-3 text-[13px] tracking-[-0.008em] transition-all duration-fast ease-standard"
          :class="
            currentKey === item.key
              ? 'nav-item-active font-semibold text-text'
              : 'font-medium text-muted hover:bg-text/[0.04] hover:text-text'
          "
        >
          <span
            v-if="currentKey === item.key"
            class="absolute left-1 top-1/2 h-3.5 w-[2px] -translate-y-1/2 rounded-full bg-accent"
            aria-hidden="true"
          />
          {{ t(item.labelKey) }}
        </RouterLink>
      </div>
    </nav>

    <div class="border-t border-line/45 px-2.5 pb-3 pt-2">
      <RouterLink
        v-for="item in footerItems"
        :key="item.key"
        :to="item.to"
        class="nav-item group relative flex h-8 items-center rounded-vt-md pl-3 pr-3 text-[13px] tracking-[-0.008em] transition-all duration-fast ease-standard"
        :class="
          currentKey === item.key
            ? 'nav-item-active font-semibold text-text'
            : 'font-medium text-muted hover:bg-text/[0.04] hover:text-text'
        "
      >
        <span
          v-if="currentKey === item.key"
          class="absolute left-1 top-1/2 h-3.5 w-[2px] -translate-y-1/2 rounded-full bg-accent"
          aria-hidden="true"
        />
        {{ t(item.labelKey) }}
      </RouterLink>
    </div>
  </aside>
</template>

<style scoped>
.sidebar-shell {
  position: relative;
}

.nav-item-active {
  background: rgb(var(--vt-color-accent) / 0.10);
  box-shadow: inset 0 0 0 1px rgb(var(--vt-color-accent) / 0.18);
}

.nav-item-active::after {
  content: '';
  position: absolute;
  inset: 0;
  border-radius: inherit;
  pointer-events: none;
  background: linear-gradient(
    90deg,
    rgb(var(--vt-color-accent) / 0.06) 0%,
    transparent 60%
  );
}
</style>
