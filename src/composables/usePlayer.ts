import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import { launchPlayer } from '@/lib/tauri-commands'
import { buildLiveUrl, buildMovieUrl, buildEpisodeUrl } from '@/lib/url-builder'
import { useProfileStore } from '@/stores/profile.store'
import { useSettingsStore } from '@/stores/settings.store'

export function usePlayer() {
  const profileStore = useProfileStore()
  const settingsStore = useSettingsStore()

  const canPlay = computed(() => profileStore.hasProfile)

  async function playLive(streamId: MaybeRefOrGetter<number>) {
    if (!profileStore.profile) return
    const url = buildLiveUrl(
      {
        serverUrl: profileStore.profile.server_url,
        username: profileStore.profile.username,
        password: '***', // Password should be handled properly in backend
      },
      toValue(streamId),
      settingsStore.liveFormat
    )
    // Send to backend which will inject the actual password
    await launchPlayer(url)
  }

  async function playMovie(streamId: MaybeRefOrGetter<number>, containerExtension: MaybeRefOrGetter<string>) {
    if (!profileStore.profile) return
    const url = buildMovieUrl(
      {
        serverUrl: profileStore.profile.server_url,
        username: profileStore.profile.username,
        password: '***',
      },
      toValue(streamId),
      toValue(containerExtension)
    )
    await launchPlayer(url)
  }

  async function playEpisode(streamId: MaybeRefOrGetter<number>, containerExtension: MaybeRefOrGetter<string>) {
    if (!profileStore.profile) return
    const url = buildEpisodeUrl(
      {
        serverUrl: profileStore.profile.server_url,
        username: profileStore.profile.username,
        password: '***',
      },
      toValue(streamId),
      toValue(containerExtension)
    )
    await launchPlayer(url)
  }

  return { canPlay, playLive, playMovie, playEpisode }
}
