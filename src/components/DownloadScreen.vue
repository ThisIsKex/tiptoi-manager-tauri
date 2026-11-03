<script setup>
import { ref, onMounted, onUnmounted } from 'vue'
import { listen } from '@tauri-apps/api/event'
import Icon from './icons/Icon.vue'

// Map von URL -> Fortschritt, damit mehrere parallele Downloads angezeigt werden können.
const downloads = ref({})
let unlisten = null

onMounted(async () => {
  unlisten = await listen('download-progress', (event) => {
    const { url, downloaded, total } = event.payload
    downloads.value[url] = {
      downloaded,
      total,
      percent: total > 0 ? Math.round((downloaded / total) * 100) : 0,
    }
  })
})

onUnmounted(() => {
  if (unlisten) unlisten()
})

function shortName(url) {
  return decodeURIComponent(url.split('/').pop() ?? url)
}
</script>

<template>
  <div class="p-4 sm:p-6 max-w-3xl mx-auto">
    <div v-if="Object.keys(downloads).length === 0" class="card p-10 flex flex-col items-center text-center gap-3">
      <div class="w-14 h-14 rounded-full bg-slate-100 flex items-center justify-center text-slate-400">
        <Icon name="download" :size="26" />
      </div>
      <p class="text-slate-500 text-sm">
        Aktuell läuft kein Download. Starte einen Download im Katalog.
      </p>
    </div>

    <div v-else class="card divide-y divide-slate-100">
      <div v-for="(d, url) in downloads" :key="url" class="px-5 py-4">
        <div class="flex justify-between items-center text-sm text-slate-700 mb-2">
          <span class="truncate pr-2 font-medium">{{ shortName(url) }}</span>
          <span class="text-slate-400 shrink-0">{{ d.percent }}%</span>
        </div>
        <div class="progress-track">
          <div class="progress-bar" :style="{ width: d.percent + '%' }" />
        </div>
      </div>
    </div>
  </div>
</template>
