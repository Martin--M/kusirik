import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Profile } from '@/types/profile'
import { getProfile } from '@/lib/tauri-commands'

/** v1: single profile, id always = 1 */
export const PROFILE_ID = 1

export const useProfileStore = defineStore('profile', () => {
  const profile = ref<Profile | null>(null)
  const isLoading = ref(false)
  const error = ref<string | null>(null)

  const hasProfile = computed(() => profile.value !== null)

  async function loadProfile() {
    isLoading.value = true
    error.value = null
    try {
      profile.value = await getProfile(PROFILE_ID)
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

  return { profile, isLoading, error, hasProfile, loadProfile, setProfile, clearProfile }
})
