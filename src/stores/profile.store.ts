import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Profile } from '@/types/profile'
import { getProfile } from '@/lib/tauri-commands'


export const useProfileStore = defineStore('profile', () => {
  const profile = ref<Profile | null>(null)
  const filterProfileId = ref<number | null | undefined>(undefined)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const hasProfile = computed(() => profile.value !== null)

  async function loadProfile(id = 1) {
    isLoading.value = true
    error.value = null
    try {
      profile.value = await getProfile(id)
    } catch (e) {
      error.value = String(e)
    } finally {
      isLoading.value = false
    }
  }

  function setProfile(p: Profile) {
    profile.value = p
  }

  function clearProfile() {
    profile.value = null
  }

  return {
    profile,
    filterProfileId,
    isLoading,
    error,
    hasProfile,
    loadProfile,
    setProfile,
    clearProfile
  }
})
