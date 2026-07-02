import { ref } from 'vue'
import en from '@/locales/en.json'
import fr from '@/locales/fr.json'

type LocaleType = 'en' | 'fr'

const messages: Record<LocaleType, any> = {
  en,
  fr
}

// Global shared state for application-wide reactivity
const currentLocale = ref<LocaleType>('en')

/**
 * Resolves a nested object path using a key like "settings.profile.title"
 */
function resolvePath(obj: any, path: string): any {
  return path.split('.').reduce((acc, part) => acc && acc[part], obj)
}

export function useI18n() {
  /**
   * Translate a key with optional dynamic placeholder replacements
   */
  const t = (key: string, replacements?: Record<string, string | number | null | undefined>): string => {
    // Try to get translation from current locale, fallback to English
    let translation = resolvePath(messages[currentLocale.value], key)
    if (translation === undefined) {
      translation = resolvePath(messages['en'], key)
    }

    if (translation === undefined || typeof translation !== 'string') {
      return key
    }

    // Handle token replacements (e.g. {error} -> message)
    if (replacements) {
      let result = translation
      for (const [token, value] of Object.entries(replacements)) {
        const replacementValue = value !== undefined && value !== null ? String(value) : ''
        result = result.replace(new RegExp(`{${token}}`, 'g'), replacementValue)
      }
      return result
    }

    return translation
  }

  /**
   * Localized date/time formatter utilizing native browser Intl APIs
   */
  const formatTime = (isoString: string | null): string => {
    if (!isoString) {
      return t('settings.stats.never')
    }
    try {
      const date = new Date(isoString)
      // Check for invalid date
      if (isNaN(date.getTime())) {
        return isoString
      }

      // Configure beautiful localized presentation using native Intl
      const options: Intl.DateTimeFormatOptions = {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
        hour12: false
      }

      // We map locales properly
      const localeTag = currentLocale.value === 'fr' ? 'fr-FR' : 'en-US'
      return new Intl.DateTimeFormat(localeTag, options).format(date)
    } catch (e) {
      return isoString
    }
  }

  /**
   * Set active locale dynamically
   */
  const setLocale = (locale: string) => {
    if (locale === 'fr' || locale === 'en') {
      currentLocale.value = locale
    }
  }

  return {
    locale: currentLocale,
    t,
    formatTime,
    setLocale
  }
}
