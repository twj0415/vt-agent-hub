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
  <aside class="flex h-full w-[216px] shrink-0 flex-col border-r border-line/35 bg-bg/88 backdrop-blur-xl backdrop-saturate-150">
    <div data-tauri-drag-region class="h-3 shrink-0" />

    <nav class="flex-1 overflow-y-auto px-3 pb-3">
      <div v-for="(group, gi) in navGroups" :key="gi" :class="gi > 0 ? 'mt-5' : 'mt-1'">
        <div class="mb-1 px-2.5 text-[11px] font-medium leading-5 text-muted/55">
          {{ t(group.titleKey) }}
        </div>

        <RouterLink
          v-for="item in group.items"
          :key="item.key"
          :to="item.to"
          class="relative flex h-8 items-center rounded-[9px] px-3 text-[13px] tracking-[-0.008em] transition-colors duration-fast ease-standard"
          :class="
            currentKey === item.key
              ? 'bg-text/[0.075] font-semibold text-text shadow-[inset_0_0_0_0.5px_rgb(var(--vt-color-line)/0.55)] before:absolute before:left-1.5 before:top-2 before:h-4 before:w-[3px] before:rounded-full before:bg-accent'
              : 'font-medium text-muted hover:bg-text/[0.045] hover:text-text'
          "
        >
          {{ t(item.labelKey) }}
        </RouterLink>
      </div>
    </nav>

    <div class="border-t border-line/35 px-3 pb-3 pt-2">
      <RouterLink
        v-for="item in footerItems"
        :key="item.key"
        :to="item.to"
        class="relative flex h-8 items-center rounded-[9px] px-3 text-[13px] tracking-[-0.008em] transition-colors duration-fast ease-standard"
        :class="
          currentKey === item.key
            ? 'bg-text/[0.08] font-semibold text-text shadow-[inset_0_1px_0_rgb(255_255_255/0.18)] before:absolute before:left-1 before:top-1.5 before:h-4 before:w-[3px] before:rounded-full before:bg-accent'
            : 'font-medium text-muted hover:bg-text/[0.04] hover:text-text'
        "
      >
        {{ t(item.labelKey) }}
      </RouterLink>
    </div>
  </aside>
</template>
