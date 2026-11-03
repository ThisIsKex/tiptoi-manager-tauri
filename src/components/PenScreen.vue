<script setup>
import { computed } from 'vue'
import Icon from './icons/Icon.vue'

const props = defineProps({
  pen: { type: Object, default: null },
  loading: { type: Boolean, default: false },
  error: { type: String, default: '' },
})
const emit = defineEmits(['refresh', 'eject', 'delete'])

const usedPercent = computed(() => {
  if (!props.pen || !props.pen.total_space) return 0
  return Math.round((props.pen.used_space / props.pen.total_space) * 100)
})

function formatSize(bytes) {
  if (!bytes) return '0 MB'
  const mb = bytes / (1024 * 1024)
  return mb > 1024 ? `${(mb / 1024).toFixed(2)} GB` : `${mb.toFixed(0)} MB`
}
</script>

<template>
  <div class="p-4 sm:p-6 max-w-6xl mx-auto space-y-6">
    <div v-if="error" class="alert-error flex items-center gap-2">
      <Icon name="alert" :size="18" />
      {{ error }}
    </div>

    <div v-if="!pen" class="card p-10 flex flex-col items-center text-center gap-3">
      <div class="w-14 h-14 rounded-full bg-slate-100 flex items-center justify-center text-slate-400">
        <Icon name="pen" :size="26" />
      </div>
      <p class="text-slate-500 text-sm">Kein Stift erkannt. Schließe den tiptoi-Stift per USB an.</p>
      <button class="btn-primary" :disabled="loading" @click="emit('refresh')">
        <span v-if="loading" class="spinner !w-4 !h-4 !border-2 !border-white/40 !border-t-white" />
        <Icon v-else name="refresh" :size="16" />
        Erneut suchen
      </button>
    </div>

    <template v-else>
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
        <div class="card p-5">
          <p class="text-xs font-medium text-slate-400 mb-1">Seriennummer</p>
          <p class="text-lg font-semibold text-slate-700 truncate">{{ pen.serial_number }}</p>
        </div>
        <div class="card p-5">
          <p class="text-xs font-medium text-slate-400 mb-1">Firmware</p>
          <p class="text-lg font-semibold text-slate-700 truncate">{{ pen.firmware_descriptor }}</p>
        </div>
        <div class="card p-5">
          <p class="text-xs font-medium text-slate-400 mb-1">Generation</p>
          <p class="text-lg font-semibold text-slate-700 truncate">{{ pen.friendly_generation }}</p>
        </div>
      </div>

      <div class="card p-5">
        <div class="flex items-center justify-between mb-2">
          <p class="text-sm font-medium text-slate-600">Speicherplatz</p>
          <p class="text-sm text-slate-400">
            {{ formatSize(pen.free_space) }} frei von {{ formatSize(pen.total_space) }}
          </p>
        </div>
        <div class="progress-track">
          <div class="progress-bar" :style="{ width: usedPercent + '%' }" />
        </div>

        <div class="flex flex-wrap gap-3 mt-5">
          <button class="btn-outline" :disabled="loading" @click="emit('refresh')">
            <span v-if="loading" class="spinner !w-4 !h-4 !border-2" />
            <Icon v-else name="refresh" :size="16" />
            Aktualisieren
          </button>
          <button class="btn-outline" @click="emit('eject', pen.mount_path)">
            <Icon name="eject" :size="16" />
            Sicher auswerfen
          </button>
        </div>
      </div>

      <div class="card overflow-hidden">
        <div class="px-5 py-4 border-b border-slate-200">
          <h2 class="text-sm font-semibold text-slate-700">Installierte Bücher</h2>
        </div>

        <p v-if="pen.installed_products.length === 0" class="text-sm text-slate-400 px-5 py-6">
          Keine Audiodateien auf dem Stift gefunden.
        </p>

        <ul v-else class="divide-y divide-slate-100">
          <li
            v-for="product in pen.installed_products"
            :key="product.path"
            class="flex items-center gap-3 px-5 py-3"
          >
            <Icon name="book" :size="18" />
            <span class="flex-1 text-sm text-slate-700 truncate">{{ product.filename }}</span>
            <span class="text-xs text-slate-400 shrink-0">{{ formatSize(product.size_bytes) }}</span>
            <button
              class="text-slate-400 hover:text-red-600 p-1.5 rounded-md hover:bg-red-50 shrink-0"
              @click="emit('delete', product.path)"
            >
              <Icon name="trash" :size="16" />
            </button>
          </li>
        </ul>
      </div>
    </template>
  </div>
</template>
