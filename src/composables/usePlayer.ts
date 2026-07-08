import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import { useQueryClient } from '@tanstack/vue-query'
import { launchPlayer, resolveStreamUrl, launchAndroidIntent, recordPlaybackHistory, getProfile } from '@/lib/tauri-commands'
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

  const canPlay = computed(() => profileStore.hasProfile)

  const isAndroid = computed(() => {
    return checkIsAndroid()
  })

  async function startPlayback(url: string, profileId?: number) {
    try {
      if (isAndroid.value) {
        // Android requires resolving the final URL with the actual password and launching it via Intent
        const finalUrl = await resolveStreamUrl(url, profileId)
        await launchAndroidIntent(finalUrl)
      } else {
        // Desktop handles resolution and process spawning inside launch_player
        await launchPlayer(url, profileId)
      }
    } catch (e: any) {
      console.error('Playback launch failed:', e)
      toastStore.showToast(e?.message || e || 'Failed to launch external player.', 'error')
    }
  }

  async function playLive(streamId: MaybeRefOrGetter<number>, profileId?: number) {
    let profile = profileStore.profile
    if (profileId && profileId !== profile?.id) {
      try {
        profile = await getProfile(profileId)
      } catch (e) {
        console.error('Failed to get profile for live playback:', e)
      }
    }
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
    let profile = profileStore.profile
    if (profileId && profileId !== profile?.id) {
      try {
        profile = await getProfile(profileId)
      } catch (e) {
        console.error('Failed to get profile for movie playback:', e)
      }
    }
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
    let profile = profileStore.profile
    if (profileId && profileId !== profile?.id) {
      try {
        profile = await getProfile(profileId)
      } catch (e) {
        console.error('Failed to get profile for episode playback:', e)
      }
    }
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
    let profile = profileStore.profile
    if (profileId && profileId !== profile?.id) {
      try {
        profile = await getProfile(profileId)
      } catch (e) {
        console.error('Failed to get profile for catchup playback:', e)
      }
    }
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
