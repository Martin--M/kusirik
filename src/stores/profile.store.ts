import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Profile } from '@/types/profile'
import { getProfiles } from '@/lib/tauri-commands'

export const useProfileStore = defineStore('profile', () => {
  const profiles = ref<Profile[]>([])
  const filterProfileId = ref<number | null | undefined>(undefined)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const hasProfile = computed(() => profiles.value.length > 0)
  const profile = computed(() => profiles.value[0] || null)

  async function loadProfiles() {
    isLoading.value = true
    error.value = null
    try {
      profiles.value = await getProfiles()
    } catch (e) {
      error.value = String(e)
    } finally {
      isLoading.value = false
    }
  }

  function clearProfiles() {
    profiles.value = []
  }

  function clearProfile() {
    clearProfiles()
  }

  function setProfile(p: Profile) {
    const idx = profiles.value.findIndex(item => item.id === p.id)
    if (idx !== -1) {
      profiles.value[idx] = p
    } else {
      profiles.value.push(p)
    }
  }

  return {
    profiles,
    profile,
    filterProfileId,
    isLoading,
    error,
    hasProfile,
    loadProfiles,
    clearProfiles,
    clearProfile,
    setProfile
  }
})
