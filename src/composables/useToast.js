import { ref } from 'vue'

// Minimaler, eigener Toast-Store (Ersatz für primevue/usetoast).
const toasts = ref([])
let nextId = 0

function add({ severity = 'info', summary = '', life = 3000 }) {
  const id = nextId++
  toasts.value.push({ id, severity, summary })
  if (life > 0) {
    setTimeout(() => remove(id), life)
  }
}

function remove(id) {
  toasts.value = toasts.value.filter((t) => t.id !== id)
}

export function useToast() {
  return { toasts, add, remove }
}
