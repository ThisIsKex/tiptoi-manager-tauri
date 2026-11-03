<script setup>
import Icon from './icons/Icon.vue'

defineProps({
  active: { type: String, required: true },
  open: { type: Boolean, default: false },
})
defineEmits(['navigate', 'close'])

const tabs = [
  { id: 'pen', label: 'Stift', icon: 'pen' },
  { id: 'catalog', label: 'Katalog', icon: 'book' },
  { id: 'downloads', label: 'Audiodateien', icon: 'download' },
  { id: 'help', label: 'Hilfe', icon: 'help' },
]
</script>

<template>
  <!-- Overlay für mobile Drawer -->
  <div
    v-if="open"
    class="fixed inset-0 bg-black/30 z-30 lg:hidden"
    @click="$emit('close')"
  />

  <aside
    class="fixed inset-y-0 left-0 z-40 w-64 bg-white border-r border-slate-200 flex flex-col
      transition-transform duration-200 lg:static lg:translate-x-0"
    :class="open ? 'translate-x-0' : '-translate-x-full'"
  >
    <div class="flex items-center gap-2 px-5 h-16 border-b border-slate-200 shrink-0">
      <span class="w-3.5 h-7 rounded-r-lg bg-gradient-to-b from-orange-400 to-amber-300" />
      <span class="font-semibold text-slate-700">tiptoi<sup>®</sup> Manager</span>
    </div>

    <nav class="flex-1 px-3 py-4 space-y-1">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        class="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm font-medium transition-colors"
        :class="
          active === tab.id
            ? 'bg-orange-50 text-orange-600'
            : 'text-slate-600 hover:bg-slate-50'
        "
        @click="$emit('navigate', tab.id)"
      >
        <Icon :name="tab.icon" :size="18" />
        {{ tab.label }}
      </button>
    </nav>

    <div class="px-5 py-4 border-t border-slate-200 text-xs text-slate-400">
      Inoffizielles Community-Tool
    </div>
  </aside>
</template>
