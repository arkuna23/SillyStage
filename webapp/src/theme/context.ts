import { createContext } from 'react'

import type { ResolvedTheme, ThemePreference } from './config'

export type ThemeContextValue = {
  resolvedTheme: ResolvedTheme
  setThemePreference: (preference: ThemePreference) => void
  themePreference: ThemePreference
}

export const ThemeContext = createContext<ThemeContextValue | null>(null)
