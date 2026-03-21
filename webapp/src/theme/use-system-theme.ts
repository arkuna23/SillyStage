import { useSyncExternalStore } from 'react'

import { getSystemTheme, type ResolvedTheme } from './config'

function subscribeToSystemTheme(onStoreChange: () => void) {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    return () => {}
  }

  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')

  mediaQuery.addEventListener('change', onStoreChange)

  return () => {
    mediaQuery.removeEventListener('change', onStoreChange)
  }
}

function getServerSnapshot(): ResolvedTheme {
  return 'dark'
}

export function useSystemTheme() {
  return useSyncExternalStore(subscribeToSystemTheme, getSystemTheme, getServerSnapshot)
}
