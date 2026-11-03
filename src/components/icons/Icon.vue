<script setup>
// Kleines, selbst gebautes Icon-Set (Outline-Stil) statt einer externen Icon-Bibliothek.
const props = defineProps({
  name: { type: String, required: true },
  size: { type: Number, default: 20 },
})

const paths = {
  pen: 'M16.862 4.487a2.1 2.1 0 0 1 2.969 2.97L8.25 19.037l-4.5 1.213 1.213-4.5L16.862 4.487Z M15 6l3 3',
  book: 'M4 5.5A2.5 2.5 0 0 1 6.5 3H20v15H6.5A2.5 2.5 0 0 0 4 20.5V5.5Z M4 20.5A2.5 2.5 0 0 1 6.5 18H20',
  download: 'M12 3v12m0 0 4-4m-4 4-4-4 M4 19h16',
  help: 'M9.09 9a3 3 0 1 1 5.82 1c0 2-3 2-3 4 M12 17h.01 M12 21a9 9 0 1 0 0-18 9 9 0 0 0 0 18Z',
  search: 'M21 21l-4.35-4.35 M10.5 18a7.5 7.5 0 1 0 0-15 7.5 7.5 0 0 0 0 15Z',
  refresh: 'M4 4v5h5 M20 20v-5h-5 M4.5 15a8 8 0 0 0 14.5 3.5 M19.5 9A8 8 0 0 0 5 5.5',
  eject: 'M12 4l7 9H5l7-9Z M5 19h14',
  trash: 'M4 7h16 M9 7V4h6v3 M6 7l1 13h10l1-13',
  close: 'M18 6 6 18 M6 6l12 12',
  check: 'M4 12l5 5L20 6',
  alert: 'M12 9v4 M12 17h.01 M10.3 3.9l-8 14A2 2 0 0 0 4 21h16a2 2 0 0 0 1.7-3.1l-8-14a2 2 0 0 0-3.4 0Z',
  folder: 'M3 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v8a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V7Z',
  'chevron-left': 'M15 18l-6-6 6-6',
}
</script>

<template>
  <svg
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="2"
    stroke-linecap="round"
    stroke-linejoin="round"
  >
    <path :d="paths[props.name] ?? ''" />
  </svg>
</template>
