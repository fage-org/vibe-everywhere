<script setup lang="ts">
const props = defineProps<{
  tone?: "default" | "success" | "warning" | "danger" | "muted";
}>();

const toneClass = {
  default: "bg-primary/12 text-primary border-primary/20",
  success: "bg-emerald-500/12 text-emerald-700 dark:text-emerald-300 border-emerald-500/20",
  warning: "bg-amber-500/12 text-amber-700 dark:text-amber-300 border-amber-500/20",
  danger: "bg-rose-500/12 text-rose-700 dark:text-rose-300 border-rose-500/20",
  muted: "bg-secondary text-muted-foreground border-border"
}[props.tone ?? "default"];
</script>

<template>
  <span
    class="inline-flex items-center rounded-full border px-2.5 py-1 text-[11px] font-medium uppercase tracking-[0.16em]"
    :class="toneClass"
  >
    <slot />
  </span>
</template>
