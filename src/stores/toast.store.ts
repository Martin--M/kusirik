import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useToastStore = defineStore('toast', () => {
  const show = ref(false)
  const message = ref('')
  const type = ref<'success' | 'error'>('success')
  let timeoutId: number | null = null

  function showToast(msg: string, t: 'success' | 'error' = 'success') {
    message.value = msg
    type.value = t
    show.value = true

    if (timeoutId) {
      clearTimeout(timeoutId)
    }

    timeoutId = window.setTimeout(() => {
      show.value = false
    }, 3000)
  }

  return {
    show,
    message,
    type,
    showToast
  }
})
