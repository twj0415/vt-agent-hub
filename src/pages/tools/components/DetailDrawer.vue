<script setup lang="ts">
  import { computed, ref, watch } from 'vue';
  import { PlusOutlined } from '@ant-design/icons-vue';
  import { useI18n } from 'vue-i18n';
  import CodePreview from '@/shared/components/feedback/CodePreview.vue';
  import VTEntityDetailDrawer from '@/shared/components/feedback/VTEntityDetailDrawer.vue';
  import type { DetailField, DetailHeader, DetailTab } from '@/shared/components/feedback/VTEntityDetailDrawer.vue';
  import RuleBindList from '@/shared/components/rules/RuleBindList.vue';
  import SkillBindList from '../../skills/components/SkillBindList.vue';
  import { useConfirm } from '@/shared/composables/useConfirm';
  import { useToolsStore } from '@/shared/stores/tools';
  import { useToolRules } from '../composables/useToolRules';
  import { useToolSkillBindings } from '../../skills/composables/useToolSkillBindings';
  import { useToolCards } from '../composables/useToolCards';

  const { t } = useI18n();
  const toolsStore = useToolsStore();
  const { boundRules } = useToolRules();
  const { boundSkills } = useToolSkillBindings();
  const { selectedToolCard } = useToolCards();
  const { confirmAction } = useConfirm();
  const bottomTab = ref<'rules' | 'skills' | 'preview'>('rules');
  const open = computed({
    get: () => toolsStore.detailOpen,
    set: (value) => toolsStore.setDetailOpen(value),
  });
  const loading = computed(() => toolsStore.bindLoading || toolsStore.skillBindLoading);
  const disabled = computed(() => !toolsStore.activeToolEnabled);
  const previewLoading = computed(() => toolsStore.globalPreviewLoading);
  const previewContent = computed(() => toolsStore.globalPreview?.afterContent ?? '');

  // 切到预览 tab 时按需拉取一次。
  watch(
    () => [open.value, bottomTab.value, toolsStore.activeId] as const,
    async ([drawerOpen, tab]) => {
      if (!drawerOpen || tab !== 'preview' || previewLoading.value || toolsStore.globalPreview) return;
      await toolsStore.loadGlobalPreview();
    },
    { immediate: true }
  );

  watch(
    () => toolsStore.activeId,
    () => {
      bottomTab.value = 'rules';
    }
  );

  const headerFields = computed<DetailField[]>(() => {
    if (!selectedToolCard.value) return [];
    return [
      { label: t('common.path'), value: selectedToolCard.value.path, mono: true },
      { label: t('pages.tools.noteLabel'), value: selectedToolCard.value.statusNote },
    ];
  });

  const header = computed<DetailHeader>(() => ({
    name: selectedToolCard.value?.name ?? '',
    status: selectedToolCard.value
      ? { tone: selectedToolCard.value.statusTone, label: selectedToolCard.value.statusLabel }
      : undefined,
    fields: headerFields.value,
  }));

  const tabs = computed<DetailTab[]>(() => [
    { key: 'rules', label: t('nav.rules') },
    { key: 'skills', label: t('nav.skills') },
    { key: 'preview', label: t('common.preview') },
  ]);

  function confirmUnbind(rule: { id: number; name: string }) {
    if (disabled.value) return
    confirmAction({
      danger: true,
      okText: t('pages.tools.binding.unbindAndApply'),
      content: t('pages.tools.binding.unbindConfirmContent', { name: rule.name }),
      onOk: async () => {
        const nextRuleIds = boundRules.value.filter((item) => item.id !== rule.id).map((item) => item.id);
        await toolsStore.saveToolRuleIdsAndSync(toolsStore.activeId, nextRuleIds, {
          notify: true,
          refreshRules: true,
        });
      },
    });
  }

  function confirmUnbindSkill(skill: { id: number; name: string }) {
    if (disabled.value) return
    confirmAction({
      danger: true,
      okText: t('common.unbind'),
      content: t('pages.projects.binding.unbindConfirmContent', { name: skill.name }),
      onOk: async () => {
        const nextSkillIds = boundSkills.value.filter((item) => item.id !== skill.id).map((item) => item.id);
        await toolsStore.saveToolSkillIdsAndSync(toolsStore.activeId, nextSkillIds, {
          notify: true,
          refreshSkills: true,
        });
      },
    });
  }
</script>

<template>
  <VTEntityDetailDrawer
    v-if="selectedToolCard"
    v-model:open="open"
    v-model:active-tab="bottomTab"
    :title="t('pages.tools.drawerTitle')"
    :loading="loading"
    :header="header"
    :tabs="tabs"
  >
    <template #tab-rules>
      <div class="min-h-0 flex-1 overflow-auto">
        <RuleBindList
          :rules="boundRules"
          :empty-text="t('pages.tools.binding.empty')"
          :unbind-text="t('common.unbind')"
          :loading="loading"
          :disabled="disabled"
          @unbind="confirmUnbind($event)"
        />
      </div>
    </template>

    <template #tab-skills>
      <div class="min-h-0 flex-1 overflow-auto">
        <div class="mb-3 flex justify-end">
          <a-button size="small" type="primary" :disabled="loading || disabled" @click="toolsStore.openSkillBinding()">
            <template #icon><PlusOutlined /></template>
            {{ t('common.bind') }}
          </a-button>
        </div>
        <SkillBindList
          :skills="boundSkills"
          :empty-text="t('pages.tools.binding.empty')"
          :unbind-text="t('common.unbind')"
          :loading="loading"
          :disabled="disabled"
          @unbind="confirmUnbindSkill($event)"
        />
      </div>
    </template>

    <template #tab-preview>
      <div class="flex min-h-0 flex-1 flex-col overflow-hidden rounded-[12px]">
        <CodePreview v-if="previewContent || !previewLoading" :content="previewContent" :empty-text="t('pages.tools.drawer.previewIdle')" />
      </div>
    </template>
  </VTEntityDetailDrawer>
</template>
