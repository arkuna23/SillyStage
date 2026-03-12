import { resources } from './resources'

export const supportedLocales = ['en', 'zh-CN'] as const
export type AppLocale = (typeof supportedLocales)[number]

export const fallbackLocale: AppLocale = 'en'
export const languageStorageKey = 'sillystage.locale'

export const localeLabelKeys: Record<AppLocale, string> = {
  en: 'common.locales.en',
  'zh-CN': 'common.locales.zh-CN',
}

export function normalizeLocale(locale: string | null | undefined): AppLocale | undefined {
  if (!locale) {
    return undefined
  }

  const normalized = locale.toLowerCase()

  if (normalized.startsWith('zh')) {
    return 'zh-CN'
  }

  if (normalized.startsWith('en')) {
    return 'en'
  }

  return undefined
}

export function isAppLocale(locale: string): locale is AppLocale {
  return locale in resources
}

function readStoredLocale(): AppLocale | undefined {
  if (typeof window === 'undefined') {
    return undefined
  }

  return normalizeLocale(window.localStorage.getItem(languageStorageKey))
}

function readNavigatorLocale(): AppLocale | undefined {
  if (typeof navigator === 'undefined') {
    return undefined
  }

  for (const candidate of navigator.languages) {
    const locale = normalizeLocale(candidate)
    if (locale) {
      return locale
    }
  }

  return normalizeLocale(navigator.language)
}

export function resolveInitialLocale(): AppLocale {
  return readStoredLocale() ?? readNavigatorLocale() ?? fallbackLocale
}

export function persistLocale(locale: AppLocale): void {
  if (typeof window === 'undefined') {
    return
  }

  window.localStorage.setItem(languageStorageKey, locale)
}

export function syncDocumentLocale(locale: AppLocale): void {
  if (typeof document === 'undefined') {
    return
  }

  document.documentElement.lang = locale
  document.documentElement.dataset.locale = locale
}
