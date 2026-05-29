<script setup lang="ts">
  import { computed } from 'vue';
  import { useI18n } from 'vue-i18n';
  import SelectableList from '@/shared/components/feedback/SelectableList.vue';
  import VTModal from '@/shared/components/feedback/VTModal.vue';
  import { useToolsStore } from '@/shared/stores/tools';
  import { useToolSkillBindings } from '../composables/useToolSkillBindings';

  const { t } = useI18n();
  const toolsStore = useToolsStore();
  const { bindableSkills, categoryLabel, skillDescription } = useToolSkillBindings();
  const loading = computed(() => toolsStore.skillBindLoading);
  const disabled = computed(() => !toolsStore.activeToolEnabled);
</script>

<template>
  <VTModal
    :open="toolsStore.skillBindOpen"
    :width="760"
    :title="`${t('common.bind')} ${t('nav.skills')}`"
    :ok-text="t('common.confirm')"
    :cancel-text="t('common.close')"
    :body-style="{ maxHeight: '72vh', overflow: 'hidden', padding: '18px 20px' }"
    :loading="loading"
    :ok-button-props="{ disabled }"
    @ok="toolsStore.saveSkillBinding()"
    @close="toolsStore.setSkillBindOpen(false)"
  >
    <SelectableList :items="bindableSkills" :empty-text="t('common.emptyData')">
      <button
        v-for="skill in bindableSkills"
        :key="skill.id"
        type="button"
        class="flex w-full items-start gap-3 border-b border-line/80 px-4 py-3 text-left transition last:border-b-0 hover:bg-accent/5"
        :disabled="loading || disabled"
        @click="toolsStore.toggleSkillSelection(skill.id)"
      >
        <a-checkbox :checked="toolsStore.skillBindingDraft.selectedNewSkillIds.includes(skill.id)" :disabled="loading || disabled" class="mt-1" @click.stop @change="toolsStore.toggleSkillSelection(skill.id)" />
        <span class="flex min-w-0 flex-1 items-start justify-between gap-3">
          <span class="min-w-0">
            <span class="flex min-w-0 flex-wrap items-baseline gap-2">
              <span class="min-w-0 truncate text-sm font-semibold text-text">{{ skill.name }}</span>
              <span class="shrink-0 text-[11px] font-normal leading-4 text-muted/60">v{{ skill.versionNo }}</span>
            </span>
            <span class="mt-1 line-clamp-2 text-xs leading-5 text-muted">{{ skillDescription(skill) }}</span>
          </span>
          <span class="flex shrink-0 items-center gap-1.5">
            <span class="shrink-0 rounded-[4px] border border-accent/55 bg-accent/20 px-1.5 py-[1px] text-[10px] font-medium leading-5 text-accent shadow-sm">
              {{ categoryLabel(skill.categoryCode) }}
            </span>
          </span>
        </span>
      </button>
    </SelectableList>
  </VTModal>
</template>
