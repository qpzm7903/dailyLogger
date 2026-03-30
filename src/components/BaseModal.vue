<template>
  <div
    class="fixed inset-0 flex items-center justify-center z-50"
    :class="backdropClass"
    @click.self="$emit('close')"
    @keydown.esc="$emit('close')"
  >
    <div
      ref="containerRef"
      role="dialog"
      aria-modal="true"
      :class="['bg-[var(--color-surface-1)] rounded-2xl border border-[var(--color-border)] shadow-2xl', contentClass]"
    >
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount } from 'vue'
import { useFocusTrap } from '../composables/useFocusTrap'

const props = withDefaults(defineProps<{
  /** Backdrop opacity: 'dark' = bg-black/80, 'medium' = bg-black/70, 'light' = bg-black/50 */
  backdrop?: 'dark' | 'medium' | 'light'
  /** Additional CSS classes for the content container */
  contentClass?: string
  /** Enable focus trap for accessibility (default: true) */
  focusTrap?: boolean
}>(), {
  backdrop: 'dark',
  contentClass: '',
  focusTrap: true,
})

defineEmits<{
  (e: 'close'): void
}>()

const backdropClass = computed(() => {
  const map = { dark: 'bg-black/80', medium: 'bg-black/70', light: 'bg-black/50' }
  return map[props.backdrop]
})

const containerRef = ref<HTMLElement | null>(null)
const { activate, deactivate } = useFocusTrap(containerRef)

onMounted(() => {
  if (props.focusTrap) {
    activate()
  }
})

onBeforeUnmount(() => {
  if (props.focusTrap) {
    deactivate()
  }
})
</script>
