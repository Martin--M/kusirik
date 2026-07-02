import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { StreamFormat } from '@/lib/url-builder'
import { pickLiveFormat } from '@/lib/url-builder'
import { getSetting, setSetting } from '@/lib/tauri-commands'
import { useI18n } from '@/composables/useI18n'

export type Theme = 'dark' | 'light'

export const useSettingsStore = defineStore('settings', () => {
  const theme = ref<Theme>('dark')
  const playerWindows = ref<string>('')
  const playerAndroid = ref<string>('') // empty = system chooser
  const allowedFormats = ref<StreamFormat[]>(['ts'])
  const liveFormatOverride = ref<StreamFormat | null>(null)
  const sidebarCollapsed = ref(false)
  const language = ref<string>('en')

  const liveFormat = computed(() =>
    pickLiveFormat(allowedFormats.value, liveFormatOverride.value)
  )

  async function load() {
    // Load local storage preferences
    const storedSidebar = localStorage.getItem('sidebar_collapsed')
    if (storedSidebar !== null) {
      sidebarCollapsed.value = storedSidebar === 'true'
    }
    try {
      const [themeVal, playerWin, playerAnd, fmtsJson, overrideVal, langVal] = await Promise.all([
        getSetting('theme'),
        getSetting('player_windows'),
        getSetting('player_android'),
        getSetting('allowed_formats'),
        getSetting('live_format_override'),
        getSetting('language'),
      ])
      if (themeVal) theme.value = themeVal as Theme
      if (playerWin) playerWindows.value = playerWin
      if (playerAnd) playerAndroid.value = playerAnd
      if (fmtsJson) allowedFormats.value = JSON.parse(fmtsJson) as StreamFormat[]
      if (overrideVal) liveFormatOverride.value = overrideVal
      
      // Load language preference, fallback to system locale detection
      const activeLang = langVal || (navigator.language.startsWith('fr') ? 'fr' : 'en')
      language.value = activeLang
      const { setLocale } = useI18n()
      setLocale(activeLang)
    } catch (e) {
      console.warn('Failed to load settings from Tauri backend (ignoring in browser dev mode):', e)
    }
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

  async function setPlayerAndroid(pkg: string) {
    playerAndroid.value = pkg
    await setSetting('player_android', pkg)
  }

  async function setAllowedFormats(fmts: StreamFormat[]) {
    allowedFormats.value = fmts
    await setSetting('allowed_formats', JSON.stringify(fmts))
  }

  async function setLanguage(lang: string) {
    language.value = lang
    await setSetting('language', lang)
    const { setLocale } = useI18n()
    setLocale(lang)
  }

  function applyTheme() {
    document.documentElement.setAttribute('data-theme', theme.value)
  }

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value
    localStorage.setItem('sidebar_collapsed', String(sidebarCollapsed.value))
  }

  return {
    theme,
    playerWindows,
    playerAndroid,
    allowedFormats,
    liveFormatOverride,
    liveFormat,
    sidebarCollapsed,
    language,
    load,
    setTheme,
    setPlayerWindows,
    setPlayerAndroid,
    setAllowedFormats,
    setLiveFormatOverride,
    setLanguage,
    applyTheme,
    toggleSidebar,
  }
})

