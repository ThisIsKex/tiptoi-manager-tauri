<script setup>
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useToast } from './composables/useToast'
import ToastContainer from './components/ToastContainer.vue'
import AppSidebar from './components/AppSidebar.vue'
import AppTopbar from './components/AppTopbar.vue'
import PenScreen from './components/PenScreen.vue'
import CatalogScreen from './components/CatalogScreen.vue'
import DownloadScreen from './components/DownloadScreen.vue'
import HelpScreen from './components/HelpScreen.vue'

const toast = useToast()
const active = ref('pen')
const sidebarOpen = ref(false)
const pen = ref(null)
const penLoading = ref(false)
const penError = ref('')
const penConnected = computed(() => pen.value !== null)

const titles = {
  pen: 'Stift',
  catalog: 'Katalog',
  downloads: 'Audiodateien',
  help: 'Hilfe',
}
const pageTitle = computed(() => titles[active.value])

function navigate(tab) {
  active.value = tab
  sidebarOpen.value = false
}

async function refreshPen() {
  penLoading.value = true
  penError.value = ''
  try {
    pen.value = await invoke('detect_pen', { explicitPath: null })
  } catch (e) {
    penError.value = `Fehler bei der Stift-Erkennung: ${e}`
  } finally {
    penLoading.value = false
  }
}

async function ejectPen(mountPath) {
  const ok = await invoke('eject_pen', { mountPath })
  toast.add({
    severity: ok ? 'success' : 'error',
    summary: ok ? 'Stift sicher ausgeworfen' : 'Auswerfen fehlgeschlagen',
    life: 3000,
  })
  if (ok) pen.value = null
}

async function downloadProduct(product) {
  if (!pen.value) {
    toast.add({ severity: 'warn', summary: 'Kein Stift verbunden', life: 3000 })
    return
  }
  const url = product.gmeUrl
  if (!url) {
    toast.add({ severity: 'error', summary: 'Keine Download-URL im Katalog gefunden', life: 3000 })
    return
  }
  const filename = decodeURIComponent(url.split('/').pop())
  const destPath = `${pen.value.mount_path}/${filename}`

  navigate('downloads')
  try {
    await invoke('download_book', { url, destPath })
    toast.add({ severity: 'success', summary: `${filename} geladen`, life: 3000 })
    await refreshPen()
  } catch (e) {
    toast.add({ severity: 'error', summary: `Download fehlgeschlagen: ${e}`, life: 4000 })
  }
}

onMounted(refreshPen)
</script>

<template>
  <div class="h-screen flex bg-slate-50">
    <ToastContainer />
    <AppSidebar :active="active" :open="sidebarOpen" @navigate="navigate" @close="sidebarOpen = false" />

    <div class="flex-1 flex flex-col min-w-0">
      <AppTopbar
        :title="pageTitle"
        :pen-connected="penConnected"
        @toggle-sidebar="sidebarOpen = !sidebarOpen"
      />

      <main class="flex-1 overflow-y-auto">
        <PenScreen
          v-if="active === 'pen'"
          :pen="pen"
          :loading="penLoading"
          :error="penError"
          @refresh="refreshPen"
          @eject="ejectPen"
        />
        <CatalogScreen v-else-if="active === 'catalog'" @download="downloadProduct" />
        <DownloadScreen v-else-if="active === 'downloads'" />
        <HelpScreen v-else-if="active === 'help'" />
      </main>
    </div>
  </div>
</template>
