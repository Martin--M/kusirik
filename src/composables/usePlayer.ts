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

  const canPlay = computed(() => profileStore.hasProfile)

  const isAndroid = computed(() => {
    return checkIsAndroid()
  })

  async function startPlayback(url: string) {
    try {
      if (isAndroid.value) {
        // Android requires resolving the final URL with the actual password and launching it via Intent
        const finalUrl = await resolveStreamUrl(url)
        await launchAndroidIntent(finalUrl)
      } else {
        // Desktop handles resolution and process spawning inside launch_player
        await launchPlayer(url)
      }
    } catch (e: any) {
      console.error('Playback launch failed:', e)
      toastStore.showToast(e?.message || e || 'Failed to launch external player.', 'error')
    }
  }

  async function playLive(streamId: MaybeRefOrGetter<number>) {
    if (!profileStore.profile) return
    const id = toValue(streamId)
    const url = buildLiveUrl(
      {
        serverUrl: profileStore.profile.server_url,
        username: profileStore.profile.username,
        password: '***',
      },
      id,
      settingsStore.liveFormat
    )
    await startPlayback(url)
    if (settingsStore.historyEnabled) {
      try {
        await recordPlaybackHistory(profileStore.profile.id, 'live', id)
        queryClient.invalidateQueries({ queryKey: ['playback_history'] })
      } catch (e) {
        console.error('Failed to record live history:', e)
      }
    }
  }

  async function playMovie(streamId: MaybeRefOrGetter<number>, containerExtension: MaybeRefOrGetter<string>) {
    if (!profileStore.profile) return
    const id = toValue(streamId)
    const url = buildMovieUrl(
      {
        serverUrl: profileStore.profile.server_url,
        username: profileStore.profile.username,
        password: '***',
      },
      id,
      toValue(containerExtension)
    )
    await startPlayback(url)
    if (settingsStore.historyEnabled) {
      try {
        await recordPlaybackHistory(profileStore.profile.id, 'vod', id)
        queryClient.invalidateQueries({ queryKey: ['playback_history'] })
      } catch (e) {
        console.error('Failed to record movie history:', e)
      }
    }
  }

  async function playEpisode(
    streamId: MaybeRefOrGetter<number>,
    containerExtension: MaybeRefOrGetter<string>,
    seriesId?: number
  ) {
    if (!profileStore.profile) return
    const id = toValue(streamId)
    const url = buildEpisodeUrl(
      {
        serverUrl: profileStore.profile.server_url,
        username: profileStore.profile.username,
        password: '***',
      },
      id,
      toValue(containerExtension)
    )
    await startPlayback(url)
    if (settingsStore.historyEnabled && seriesId) {
      try {
        await recordPlaybackHistory(profileStore.profile.id, 'series', seriesId)
        queryClient.invalidateQueries({ queryKey: ['playback_history'] })
      } catch (e) {
        console.error('Failed to record series history:', e)
      }
    }
  }

  async function playCatchup(streamId: MaybeRefOrGetter<number>, startDateTime: string, durationMinutes: number) {
    if (!profileStore.profile) return
    const id = toValue(streamId)
    const url = buildCatchupUrl(
      {
        serverUrl: profileStore.profile.server_url,
        username: profileStore.profile.username,
        password: '***',
      },
      id,
      startDateTime,
      durationMinutes
    )
    await startPlayback(url)
    if (settingsStore.historyEnabled) {
      try {
        await recordPlaybackHistory(profileStore.profile.id, 'live', id)
        queryClient.invalidateQueries({ queryKey: ['playback_history'] })
      } catch (e) {
        console.error('Failed to record catchup history:', e)
      }
    }
  }

  return { canPlay, playLive, playMovie, playEpisode, playCatchup }
}
