import { ref, onMounted } from 'vue'
import { platform } from '@tauri-apps/plugin-os'

const currentPlatform = ref(null)
const isMobile = ref(false)
const isDesktop = ref(true)

/**
 * Composable for detecting the current platform.
 *
 * Uses Tauri's os plugin to detect the platform at runtime.
 * Provides isMobile and isDesktop refs for conditional rendering.
 *
 * @returns {{ platform: import('vue').Ref<string|null>, isMobile: import('vue').Ref<boolean>, isDesktop: import('vue').Ref<boolean> }}
 */
export function usePlatform() {
  onMounted(async () => {
    try {
      currentPlatform.value = await platform()
      // 'android' or 'ios' indicates mobile platform
      isMobile.value = currentPlatform.value === 'android' || currentPlatform.value === 'ios'
      isDesktop.value = !isMobile.value
    } catch (e) {
      // Fallback for web development or non-Tauri environment
      console.warn('Platform detection failed, assuming desktop:', e)
      currentPlatform.value = 'unknown'
      isMobile.value = false
      isDesktop.value = true
    }
  })

  return {
    platform: currentPlatform,
    isMobile,
    isDesktop
  }
}