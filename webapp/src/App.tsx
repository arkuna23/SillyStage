import { lazy, useEffect } from 'react'
import { BrowserRouter, Navigate, Route, Routes, useLocation } from 'react-router-dom'

import { appPaths } from './app/paths'
import { AppShell } from './components/layout/app-shell'
import { WorkspaceLayout } from './components/layout/workspace-layout'
import { ThemeProvider } from './components/theme-provider'

const HomePage = lazy(() =>
  import('./pages/home-page').then((module) => ({ default: module.HomePage })),
)

const CharacterManagementPage = lazy(() =>
  import('./features/characters/character-management-page').then((module) => ({
    default: module.CharacterManagementPage,
  })),
)

function ScrollToTopOnNavigation() {
  const location = useLocation()

  useEffect(() => {
    window.scrollTo({ left: 0, top: 0 })
  }, [location.pathname])

  return null
}

function AppRoutes() {
  return (
    <>
      <ScrollToTopOnNavigation />

      <Routes>
        <Route element={<AppShell />}>
          <Route element={<HomePage />} path={appPaths.home} />
          <Route
            element={<Navigate replace to={appPaths.workspace} />}
            path={appPaths.workspaceRoot}
          />
          <Route element={<WorkspaceLayout />} path={appPaths.workspaceRoot}>
            <Route element={<CharacterManagementPage />} path="characters" />
          </Route>
          <Route element={<Navigate replace to={appPaths.home} />} path="*" />
        </Route>
      </Routes>
    </>
  )
}

function App() {
  return (
    <ThemeProvider>
      <BrowserRouter>
        <AppRoutes />
      </BrowserRouter>
    </ThemeProvider>
  )
}

export default App
