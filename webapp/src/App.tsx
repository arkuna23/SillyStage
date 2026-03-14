import { lazy, useEffect } from 'react'
import { BrowserRouter, Navigate, Route, Routes, useLocation } from 'react-router-dom'

import { appPaths } from './app/paths'
import { AppShell } from './components/layout/app-shell'
import { WorkspaceLayout } from './components/layout/workspace-layout'
import { ThemeProvider } from './components/theme-provider'

const HomePage = lazy(() =>
  import('./pages/home-page').then((module) => ({ default: module.HomePage })),
)

const DashboardPage = lazy(() =>
  import('./features/dashboard/dashboard-page').then((module) => ({
    default: module.DashboardPage,
  })),
)

const CharacterManagementPage = lazy(() =>
  import('./features/characters/character-management-page').then((module) => ({
    default: module.CharacterManagementPage,
  })),
)

const ApiManagementPage = lazy(() =>
  import('./features/apis/api-management-page').then((module) => ({
    default: module.ApiManagementPage,
  })),
)

const SchemaManagementPage = lazy(() =>
  import('./features/schemas/schema-management-page').then((module) => ({
    default: module.SchemaManagementPage,
  })),
)

const PlayerProfilesPage = lazy(() =>
  import('./features/player-profiles/player-profiles-page').then((module) => ({
    default: module.PlayerProfilesPage,
  })),
)

const StoryResourcesPage = lazy(() =>
  import('./features/story-resources/story-resources-page').then((module) => ({
    default: module.StoryResourcesPage,
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
            <Route element={<ApiManagementPage />} path="apis" />
            <Route element={<CharacterManagementPage />} path="characters" />
            <Route element={<DashboardPage />} path="dashboard" />
            <Route element={<StoryResourcesPage />} path="story-resources" />
            <Route element={<SchemaManagementPage />} path="schemas" />
            <Route element={<PlayerProfilesPage />} path="player-profiles" />
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
