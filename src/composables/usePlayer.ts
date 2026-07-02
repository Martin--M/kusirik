import { computed, toValue, type MaybeRefOrGetter } from 'vue'
import { launchPlayer, resolveStreamUrl, launchAndroidIntent } from '@/lib/tauri-commands'
import { buildLiveUrl, buildMovieUrl, buildEpisodeUrl, buildCatchupUrl } from '@/lib/url-builder'
import { useProfileStore } from '@/stores/profile.store'
import { useSettingsStore } from '@/stores/settings.store'
import { useToastStore } from '@/stores/toast.store'

export function usePlayer() {
  const profileStore = useProfileStore()
  const settingsStore = useSettingsStore()
  const toastStore = useToastStore()

  const canPlay = computed(() => profileStore.hasProfile)

  const isAndroid = computed(() => {
    return typeof navigator !== 'undefined' && /Android/i.test(navigator.userAgent)
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
    const url = buildLiveUrl(
      {
        serverUrl: profileStore.profile.server_url,
        username: profileStore.profile.username,
        password: '***',
      },
      toValue(streamId),
      settingsStore.liveFormat
    )
    await startPlayback(url)
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
    await startPlayback(url)
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
    await startPlayback(url)
  }

  async function playCatchup(streamId: MaybeRefOrGetter<number>, startDateTime: string, durationMinutes: number) {
    if (!profileStore.profile) return
    const url = buildCatchupUrl(
      {
        serverUrl: profileStore.profile.server_url,
        username: profileStore.profile.username,
        password: '***',
      },
      toValue(streamId),
      startDateTime,
      durationMinutes
    )
    await startPlayback(url)
  }

  return { canPlay, playLive, playMovie, playEpisode, playCatchup }
}
