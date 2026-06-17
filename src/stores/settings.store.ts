import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { StreamFormat } from '@/lib/url-builder'
import { pickLiveFormat } from '@/lib/url-builder'
import { getSetting, setSetting } from '@/lib/tauri-commands'

export type Theme = 'dark' | 'light'

export const useSettingsStore = defineStore('settings', () => {
  const theme = ref<Theme>('dark')
  const playerWindows = ref<string>('C:\\Program Files\\VideoLAN\\VLC\\vlc.exe')
  const playerAndroid = ref<string>('') // empty = system chooser
  const allowedFormats = ref<StreamFormat[]>(['ts'])
  const liveFormatOverride = ref<StreamFormat | null>(null)

  const liveFormat = computed(() =>
    pickLiveFormat(allowedFormats.value, liveFormatOverride.value)
  )

  async function load() {
    const [themeVal, playerWin, playerAnd, fmtsJson, overrideVal] = await Promise.all([
      getSetting('theme'),
      getSetting('player_windows'),
      getSetting('player_android'),
      getSetting('allowed_formats'),
      getSetting('live_format_override'),
    ])
    if (themeVal) theme.value = themeVal as Theme
    if (playerWin) playerWindows.value = playerWin
    if (playerAnd) playerAndroid.value = playerAnd
    if (fmtsJson) allowedFormats.value = JSON.parse(fmtsJson) as StreamFormat[]
    if (overrideVal) liveFormatOverride.value = overrideVal
  }

  async function setTheme(t: Theme) {
    theme.value = t
    await setSetting('theme', t)
    document.documentElement.setAttribute('data-theme', t)
  }

  async function setPlayerWindows(path: string) {
    playerWindows.value = path
    await setSetting('player_windows', path)
  }

  async function setLiveFormatOverride(fmt: StreamFormat | null) {
    liveFormatOverride.value = fmt
    await setSetting('live_format_override', fmt ?? '')
  }

  function applyTheme() {
    document.documentElement.setAttribute('data-theme', theme.value)
  }

  return {
    theme,
    playerWindows,
    playerAndroid,
    allowedFormats,
    liveFormatOverride,
    liveFormat,
    load,
    setTheme,
    setPlayerWindows,
    setLiveFormatOverride,
    applyTheme,
  }
})
