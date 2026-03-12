export const themeStorageKey = 'sillystage.theme'

export const themePreferences = ['system', 'light', 'dark'] as const

export type ThemePreference = (typeof themePreferences)[number]
export type ResolvedTheme = Exclude<ThemePreference, 'system'>

export function isThemePreference(
  value: string | null | undefined,
): value is ThemePreference {
  return themePreferences.includes(value as ThemePreference)
}

function readStoredThemePreference(): ThemePreference | undefined {
  if (typeof window === 'undefined') {
    return undefined
  }

  const storedValue = window.localStorage.getItem(themeStorageKey)
  return isThemePreference(storedValue) ? storedValue : undefined
}

export function getSystemTheme(): ResolvedTheme {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    return 'dark'
  }

  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

export function resolveInitialThemePreference(): ThemePreference {
  return readStoredThemePreference() ?? 'system'
}

export function resolveTheme(
  preference: ThemePreference,
  systemTheme: ResolvedTheme = getSystemTheme(),
): ResolvedTheme {
  return preference === 'system' ? systemTheme : preference
}

export function syncDocumentTheme(theme: ResolvedTheme, preference: ThemePreference) {
  if (typeof document === 'undefined') {
    return
  }

  document.documentElement.dataset.theme = theme
  document.documentElement.dataset.themePreference = preference
  document.documentElement.style.colorScheme = theme
}
