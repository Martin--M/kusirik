import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import { useQueryClient } from '@tanstack/vue-query'
import { launchPlayer, resolveStreamUrl, launchAndroidIntent, recordPlaybackHistory } from '@/lib/tauri-commands'
import { buildLiveUrl, buildMovieUrl, buildEpisodeUrl, buildCatchupUrl } from '@/lib/url-builder'
import { useProfileStore } from '@/stores/profile.store'
import { useSettingsStore } from '@/stores/settings.store'
import { useToastStore } from '@/stores/toast.store'
import { checkIsAndroid } from '@/lib/device'

export function usePlayer() {
  const profileStore = useProfileStore()
  const settingsStore = useSettingsStore()
  const toastStore = useToastStore()
  const queryClient = useQueryClient()

  const canPlay = computed(() => profileStore.profiles.length > 0)

  const isAndroid = computed(() => {
    return checkIsAndroid()
  })

  function getProfileById(profileId?: number) {
    if (profileId) {
      return profileStore.profiles.find(p => p.id === profileId)
    }
    return profileStore.profiles[0]
  }

  async function startPlayback(url: string, profileId?: number) {
    try {
      if (isAndroid.value) {
        const finalUrl = await resolveStreamUrl(url, profileId)
        await launchAndroidIntent(finalUrl)
      } else {
        await launchPlayer(url, profileId)
      }
    } catch (e: any) {
      console.error('Playback launch failed:', e)
      toastStore.showToast(e?.message || e || 'Failed to launch external player.', 'error')
    }
  }

  async function playLive(streamId: MaybeRefOrGetter<number>, profileId?: number) {
    const profile = getProfileById(profileId)
    if (!profile) return
    const id = toValue(streamId)
    const url = buildLiveUrl(
      {
        serverUrl: profile.server_url,
        username: profile.username,
        password: '***',
      },
      id,
      settingsStore.liveFormat
    )
    await startPlayback(url, profile.id)
    if (settingsStore.historyEnabled) {
      try {
        await recordPlaybackHistory(profile.id, 'live', id)
        queryClient.invalidateQueries({ queryKey: ['playback_history'] })
      } catch (e) {
        console.error('Failed to record live history:', e)
      }
    }
  }

  async function playMovie(
    streamId: MaybeRefOrGetter<number>,
    containerExtension: MaybeRefOrGetter<string>,
    profileId?: number
  ) {
    const profile = getProfileById(profileId)
    if (!profile) return
    const id = toValue(streamId)
    const url = buildMovieUrl(
      {
        serverUrl: profile.server_url,
        username: profile.username,
        password: '***',
      },
      id,
      toValue(containerExtension)
    )
    await startPlayback(url, profile.id)
    if (settingsStore.historyEnabled) {
      try {
        await recordPlaybackHistory(profile.id, 'vod', id)
        queryClient.invalidateQueries({ queryKey: ['playback_history'] })
      } catch (e) {
        console.error('Failed to record movie history:', e)
      }
    }
  }

  async function playEpisode(
    streamId: MaybeRefOrGetter<number>,
    containerExtension: MaybeRefOrGetter<string>,
    seriesId?: number,
    profileId?: number
  ) {
    const profile = getProfileById(profileId)
    if (!profile) return
    const id = toValue(streamId)
    const url = buildEpisodeUrl(
      {
        serverUrl: profile.server_url,
        username: profile.username,
        password: '***',
      },
      id,
      toValue(containerExtension)
    )
    await startPlayback(url, profile.id)
    if (settingsStore.historyEnabled && seriesId) {
      try {
        await recordPlaybackHistory(profile.id, 'series', seriesId)
        queryClient.invalidateQueries({ queryKey: ['playback_history'] })
      } catch (e) {
        console.error('Failed to record series history:', e)
      }
    }
  }

  async function playCatchup(
    streamId: MaybeRefOrGetter<number>,
    startDateTime: string,
    durationMinutes: number,
    profileId?: number
  ) {
    const profile = getProfileById(profileId)
    if (!profile) return
    const id = toValue(streamId)
    const url = buildCatchupUrl(
      {
        serverUrl: profile.server_url,
        username: profile.username,
        password: '***',
      },
      id,
      startDateTime,
      durationMinutes
    )
    await startPlayback(url, profile.id)
    if (settingsStore.historyEnabled) {
      try {
        await recordPlaybackHistory(profile.id, 'live', id)
        queryClient.invalidateQueries({ queryKey: ['playback_history'] })
      } catch (e) {
        console.error('Failed to record catchup history:', e)
      }
    }
  }

  return { canPlay, playLive, playMovie, playEpisode, playCatchup }
}
