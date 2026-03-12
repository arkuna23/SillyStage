import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'

import en from './locales/en'
import zhCN from './locales/zh-CN'

export const localeStorageKey = 'sillystage.locale'
export const supportedLocales = ['zh-CN', 'en'] as const
const i18nextSupportedLocales = ['en', 'zh', 'zh-CN'] as const

export type AppLocale = (typeof supportedLocales)[number]

function normalizeLocale(value?: string | null): AppLocale | undefined {
  if (!value) {
    return undefined
  }

  const normalized = value.toLowerCase()

  if (normalized.startsWith('zh')) {
    return 'zh-CN'
  }

  if (normalized.startsWith('en')) {
    return 'en'
  }

  return undefined
}

function syncDocumentLanguage(locale: AppLocale) {
  if (typeof document === 'undefined') {
    return
  }

  document.documentElement.lang = locale
}

function resolveInitialLocale(): AppLocale {
  if (typeof window !== 'undefined') {
    const storedLocale = normalizeLocale(window.localStorage.getItem(localeStorageKey))
    if (storedLocale) {
      return storedLocale
    }

    const browserLocales = window.navigator.languages ?? [window.navigator.language]
    const matchedLocale = browserLocales
      .map((locale) => normalizeLocale(locale))
      .find((locale): locale is AppLocale => Boolean(locale))

    if (matchedLocale) {
      return matchedLocale
    }
  }

  return 'en'
}

const initialLocale = resolveInitialLocale()

void i18n.use(initReactI18next).init({
  fallbackLng: 'en',
  interpolation: {
    escapeValue: false,
  },
  load: 'languageOnly',
  lng: initialLocale,
  nonExplicitSupportedLngs: true,
  resources: {
    en: { translation: en },
    zh: { translation: zhCN },
    'zh-CN': { translation: zhCN },
  },
  supportedLngs: i18nextSupportedLocales,
})

syncDocumentLanguage(initialLocale)

i18n.on('languageChanged', (locale) => {
  const normalizedLocale = normalizeLocale(locale) ?? 'en'

  if (typeof window !== 'undefined') {
    window.localStorage.setItem(localeStorageKey, normalizedLocale)
  }

  syncDocumentLanguage(normalizedLocale)
})

export default i18n
