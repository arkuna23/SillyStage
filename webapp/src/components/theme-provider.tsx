import { type PropsWithChildren, useEffect, useState } from 'react'

import { ThemeContext } from '../theme/context'
import {
  resolveInitialThemePreference,
  resolveTheme,
  syncDocumentTheme,
  themeStorageKey,
  type ThemePreference,
} from '../theme/config'
import { useSystemTheme } from '../theme/use-system-theme'

const initialThemePreference = resolveInitialThemePreference()

syncDocumentTheme(resolveTheme(initialThemePreference), initialThemePreference)

export function ThemeProvider({ children }: PropsWithChildren) {
  const [themePreference, setThemePreference] = useState<ThemePreference>(
    initialThemePreference,
  )
  const systemTheme = useSystemTheme()
  const resolvedTheme = resolveTheme(themePreference, systemTheme)

  useEffect(() => {
    syncDocumentTheme(resolvedTheme, themePreference)

    if (typeof window !== 'undefined') {
      window.localStorage.setItem(themeStorageKey, themePreference)
    }
  }, [resolvedTheme, themePreference])

  return (
    <ThemeContext.Provider
      value={{
        resolvedTheme,
        setThemePreference,
        themePreference,
      }}
    >
      {children}
    </ThemeContext.Provider>
  )
}
