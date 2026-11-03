<script setup>
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import Icon from './icons/Icon.vue'

const emit = defineEmits(['download'])

// Zweistufige Navigation wie auf der Ravensburger-Seite selbst: erst
// Kategorien (ein einzelner, günstiger Request), Produkte einer Kategorie
// werden erst bei Auswahl nachgeladen (und dann pro Kategorie gecacht).
const categories = ref([])
const categoriesLoading = ref(true)
const categoriesError = ref('')

const selectedCategory = ref(null)
const products = ref([])
const productsLoading = ref(false)
const productsError = ref('')

const query = ref('')

async function loadCategories(forceRefresh = false) {
  categoriesLoading.value = true
  categoriesError.value = ''
  try {
    categories.value = await invoke('get_categories', { forceRefresh })
  } catch (e) {
    categoriesError.value = `Kategorien konnten nicht geladen werden: ${e}`
  } finally {
    categoriesLoading.value = false
  }
}

async function openCategory(category, forceRefresh = false) {
  selectedCategory.value = category
  query.value = ''
  productsLoading.value = true
  productsError.value = ''
  try {
    const data = await invoke('get_category_catalog', {
      categoryId: category.id,
      forceRefresh,
    })
    products.value = data.products ?? []
  } catch (e) {
    productsError.value = `Katalog konnte nicht geladen werden: ${e}`
  } finally {
    productsLoading.value = false
  }
}

function backToCategories() {
  selectedCategory.value = null
  products.value = []
}

onMounted(() => loadCategories(false))

const filtered = computed(() => {
  const q = query.value.toLowerCase()
  return products.value.filter((p) => (p.name ?? '').toLowerCase().includes(q))
})
</script>

<template>
  <div class="p-4 sm:p-6 max-w-6xl mx-auto">
    <!-- Kategorie-Übersicht -->
    <template v-if="!selectedCategory">
      <div class="flex items-center justify-between mb-6">
        <h2 class="text-lg font-semibold text-slate-700">Kategorien</h2>
        <button class="btn-outline shrink-0" :disabled="categoriesLoading" @click="loadCategories(true)">
          <span v-if="categoriesLoading" class="spinner !w-4 !h-4 !border-2" />
          <Icon v-else name="refresh" :size="16" />
          Aktualisieren
        </button>
      </div>

      <div v-if="categoriesError" class="alert-error mb-4 flex items-center gap-2">
        <Icon name="alert" :size="18" />
        {{ categoriesError }}
      </div>

      <div v-if="categoriesLoading" class="flex justify-center py-16">
        <span class="spinner" />
      </div>

      <div v-else class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-4">
        <button
          v-for="c in categories"
          :key="c.id"
          class="card p-5 flex flex-col items-center gap-3 text-center hover:border-orange-300 transition-colors"
          @click="openCategory(c)"
        >
          <div class="w-12 h-12 rounded-full bg-orange-50 flex items-center justify-center text-orange-500">
            <Icon name="folder" :size="22" />
          </div>
          <span class="text-sm font-medium text-slate-700 line-clamp-2">{{ c.name }}</span>
        </button>
      </div>
    </template>

    <!-- Produkte einer Kategorie -->
    <template v-else>
      <div class="flex flex-col sm:flex-row gap-3 mb-6 sm:items-center">
        <button class="btn-outline shrink-0 !w-fit" @click="backToCategories">
          <Icon name="chevron-left" :size="16" />
          Kategorien
        </button>
        <h2 class="text-base font-semibold text-slate-700 flex-1 min-w-0 truncate">
          {{ selectedCategory.name }}
        </h2>
      </div>

      <div class="flex flex-col sm:flex-row gap-3 mb-6">
        <div class="relative sm:max-w-sm w-full">
          <Icon name="search" :size="16" class="absolute left-3 top-1/2 -translate-y-1/2 text-slate-400" />
          <input v-model="query" type="text" placeholder="Buch suchen…" class="input !pl-9" />
        </div>
        <button
          class="btn-outline shrink-0"
          :disabled="productsLoading"
          @click="openCategory(selectedCategory, true)"
        >
          <span v-if="productsLoading" class="spinner !w-4 !h-4 !border-2" />
          <Icon v-else name="refresh" :size="16" />
          Aktualisieren
        </button>
      </div>

      <div v-if="productsError" class="alert-error mb-4 flex items-center gap-2">
        <Icon name="alert" :size="18" />
        {{ productsError }}
      </div>

      <div v-if="productsLoading" class="flex justify-center py-16">
        <span class="spinner" />
      </div>

      <div v-else class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
        <div
          v-for="p in filtered"
          :key="p.id ?? p.name"
          class="card p-4 flex flex-col"
        >
          <div class="aspect-square rounded-lg bg-slate-50 flex items-center justify-center mb-3 overflow-hidden">
            <img
              v-if="p.thumbnailUrl"
              :src="p.thumbnailUrl"
              loading="lazy"
              class="w-full h-full object-contain"
            />
            <Icon v-else name="book" :size="32" class="text-slate-300" />
          </div>
          <h3 class="text-sm font-medium text-slate-700 mb-3 flex-1 line-clamp-2">
            {{ p.name }}
          </h3>
          <button
            class="btn-primary w-full !text-xs !py-1.5"
            :disabled="!p.gmeUrl"
            @click="emit('download', p)"
          >
            <Icon name="download" :size="14" />
            Laden
          </button>
        </div>

        <p v-if="filtered.length === 0" class="col-span-full text-center text-slate-400 py-10">
          Keine Treffer.
        </p>
      </div>
    </template>
  </div>
</template>
