<script setup>
import Icon from './icons/Icon.vue'

defineProps({
  title: { type: String, required: true },
  penConnected: { type: Boolean, default: false },
})
defineEmits(['toggle-sidebar'])
</script>

<template>
  <header class="h-16 shrink-0 bg-white border-b border-slate-200 flex items-center gap-4 px-4 sm:px-6">
    <button
      class="lg:hidden text-slate-500 hover:text-slate-700"
      @click="$emit('toggle-sidebar')"
    >
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
        <path d="M4 6h16M4 12h16M4 18h16" />
      </svg>
    </button>

    <h1 class="text-base font-semibold text-slate-700 flex-1">{{ title }}</h1>

    <span
      class="inline-flex items-center gap-1.5 rounded-full px-3 py-1 text-xs font-medium"
      :class="penConnected ? 'bg-emerald-100 text-emerald-700' : 'bg-slate-100 text-slate-500'"
    >
      <span class="w-2 h-2 rounded-full" :class="penConnected ? 'bg-emerald-500' : 'bg-slate-400'" />
      {{ penConnected ? 'Stift verbunden' : 'Kein Stift erkannt' }}
    </span>
  </header>
</template>
