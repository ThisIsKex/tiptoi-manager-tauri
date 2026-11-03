<script setup>
import { useToast } from '../composables/useToast'

const { toasts, remove } = useToast()

const styles = {
  success: 'bg-emerald-600',
  error: 'bg-red-600',
  warn: 'bg-amber-500',
  info: 'bg-slate-700',
}
</script>

<template>
  <div class="fixed inset-x-0 top-4 z-50 flex flex-col items-center gap-2 px-4 pointer-events-none">
    <transition-group name="toast">
      <div
        v-for="t in toasts"
        :key="t.id"
        class="pointer-events-auto w-full max-w-sm rounded-lg px-4 py-3 text-sm text-white shadow-lg flex items-start gap-3"
        :class="styles[t.severity] ?? styles.info"
      >
        <span class="flex-1">{{ t.summary }}</span>
        <button class="opacity-70 hover:opacity-100" @click="remove(t.id)">✕</button>
      </div>
    </transition-group>
  </div>
</template>

<style scoped>
.toast-enter-active,
.toast-leave-active {
  transition: all 0.2s ease;
}
.toast-enter-from,
.toast-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}
</style>
