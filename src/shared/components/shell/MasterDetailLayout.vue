<template>
  <div class="grid gap-5 xl:grid-cols-[340px_minmax(0,1fr)] 2xl:grid-cols-[390px_minmax(0,1fr)]">
    <div class="space-y-4">
      <slot name="master" />
    </div>
    <div class="space-y-4">
      <slot name="detail" />
    </div>
  </div>
</template>
