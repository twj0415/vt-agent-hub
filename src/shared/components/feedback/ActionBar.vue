<template>
  <div class="flex flex-wrap items-center justify-between gap-3 rounded-[16px] border border-line/60 bg-panel/65 p-4 shadow-surface">
    <div class="min-w-0 flex-1">
      <slot name="meta" />
    </div>
    <div class="flex flex-wrap items-center gap-2">
      <slot name="actions" />
    </div>
  </div>
</template>
