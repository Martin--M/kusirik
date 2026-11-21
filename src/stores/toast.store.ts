import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useToastStore = defineStore('toast', () => {
  const show = ref(false)
  const message = ref('')
  const type = ref<'success' | 'error'>('success')
  let timeoutId: number | null = null

  function showToast(msg: string, t: 'success' | 'error' = 'success', durationMs?: number) {
    message.value = msg
    type.value = t
    show.value = true

    if (timeoutId) {
      clearTimeout(timeoutId)
    }

    const timeout = durationMs ?? (t === 'error' ? 6000 : 3000)
    timeoutId = window.setTimeout(() => {
      show.value = false
    }, timeout)
  }

  return {
    show,
    message,
    type,
    showToast
  }
})
