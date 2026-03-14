import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { motion, useReducedMotion } from 'framer-motion'
import { Suspense, useCallback, useEffect, useMemo, useRef, useState } from 'react'
import type { ReactNode } from 'react'
import { useLocation, useOutlet } from 'react-router-dom'
import { useTranslation } from 'react-i18next'

import { appPaths } from '../../app/paths'
import { getWorkspaceNavigationItems } from '../../app/workspace-navigation'
import { WorkspacePanelShell } from './workspace-panel-shell'
import { WorkspaceRail, WorkspaceRailSkeleton } from '../workspace-rail'
import { Button } from '../ui/button'
import { Card } from '../ui/card'
import { WorkspaceSidebar } from '../workspace-sidebar'
import type { WorkspaceRailContent } from './workspace-context'

function shouldShowWorkspaceRail(pathname: string) {
  return (
    pathname.startsWith(appPaths.dashboard) ||
    pathname.startsWith(appPaths.characters) ||
    pathname.startsWith(appPaths.storyResources) ||
    pathname.startsWith(appPaths.stories) ||
    pathname.startsWith(appPaths.apis) ||
    pathname.startsWith(appPaths.schemas) ||
    pathname.startsWith(appPaths.playerProfiles)
  )
}

function DashboardContentFallback() {
  return (
    <WorkspacePanelShell className="h-full">
      <Card className="h-full overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_94%,transparent)] shadow-none">
        <div className="border-b border-[var(--color-border-subtle)] px-6 py-5">
          <div className="h-10 w-52 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
        </div>

        <div className="space-y-8 p-6">
          <section className="space-y-4">
            <div className="h-7 w-40 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
              {Array.from({ length: 4 }).map((_, index) => (
                <div
                  className="rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4"
                  key={index}
                >
                  <div className="h-3 w-20 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
                  <div className="mt-4 h-8 w-14 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
                </div>
              ))}
            </div>
          </section>

          <div className="border-t border-[var(--color-border-subtle)]" />

          <section className="grid gap-6 xl:grid-cols-[minmax(0,0.9fr)_minmax(0,1.1fr)]">
            {Array.from({ length: 2 }).map((_, index) => (
              <div
                className="rounded-[1.5rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-5 py-5"
                key={index}
              >
                <div className="h-6 w-32 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
                <div className="mt-4 space-y-3">
                  {Array.from({ length: index === 0 ? 2 : 6 }).map((__, rowIndex) => (
                    <div className="h-10 rounded-[1rem] bg-[var(--color-bg-panel)]" key={rowIndex} />
                  ))}
                </div>
              </div>
            ))}
          </section>
        </div>
      </Card>
    </WorkspacePanelShell>
  )
}

function CharacterContentFallback() {
  return (
    <WorkspacePanelShell className="h-full">
      <Card className="h-full overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_94%,transparent)] shadow-none">
        <div className="border-b border-[var(--color-border-subtle)] px-6 py-6">
          <div className="h-4 w-24 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
          <div className="mt-4 h-10 w-64 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
        </div>

        <div className="grid gap-4 p-6 md:grid-cols-2 2xl:grid-cols-3">
          {Array.from({ length: 6 }).map((_, index) => (
            <div
              className="overflow-hidden rounded-[1.65rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)]"
              key={index}
            >
              <div className="h-40 animate-pulse bg-[color-mix(in_srgb,var(--color-accent-gold-soft)_55%,var(--color-bg-panel))]" />
              <div className="space-y-3 p-5">
                <div className="h-3 w-20 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
                <div className="h-6 w-2/3 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
                <div className="h-3 w-full animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
                <div className="h-3 w-4/5 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
              </div>
            </div>
          ))}
        </div>
      </Card>
    </WorkspacePanelShell>
  )
}

function ResourceListContentFallback() {
  return (
    <WorkspacePanelShell className="h-full">
      <Card className="h-full overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_94%,transparent)] shadow-none">
        <div className="border-b border-[var(--color-border-subtle)] px-6 py-5">
          <div className="flex items-center justify-between gap-4">
            <div className="h-10 w-48 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="h-11 w-11 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
          </div>
        </div>

        <div className="space-y-6 p-6">
          <div className="h-16 animate-pulse rounded-[1.4rem] bg-[var(--color-bg-elevated)]" />
          <div className="divide-y divide-[var(--color-border-subtle)]">
            {Array.from({ length: 5 }).map((_, index) => (
              <div
                className="grid gap-4 py-4 lg:grid-cols-[minmax(0,1.05fr)_minmax(0,0.95fr)_auto] lg:items-center"
                key={index}
              >
                <div className="space-y-2.5">
                  <div className="h-5 w-36 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
                  <div className="h-3 w-full animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
                </div>
                <div className="space-y-2.5">
                  <div className="h-3 w-24 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
                  <div className="h-3 w-4/5 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
                </div>
                <div className="flex justify-end gap-2">
                  <div className="h-9 w-16 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
                  <div className="h-9 w-16 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
                </div>
              </div>
            ))}
          </div>
        </div>
      </Card>
    </WorkspacePanelShell>
  )
}

function WorkspaceContentFallback({ pathname }: { pathname: string }) {
  if (pathname.startsWith(appPaths.dashboard)) {
    return <DashboardContentFallback />
  }

  if (pathname.startsWith(appPaths.characters)) {
    return <CharacterContentFallback />
  }

  return <ResourceListContentFallback />
}

const panelEase = [0.16, 1, 0.3, 1] as const
type WorkspaceStagePhase = 'entering' | 'idle' | 'exiting'

export function WorkspaceLayout() {
  const { t } = useTranslation()
  const location = useLocation()
  const prefersReducedMotion = useReducedMotion()
  const [openRailPath, setOpenRailPath] = useState<string | null>(null)
  const [railContent, setRailContent] = useState<WorkspaceRailContent | null>(null)
  const workspaceNavigationItems = useMemo(
    () => getWorkspaceNavigationItems(t),
    [t],
  )
  const [activeWorkspacePath, setActiveWorkspacePath] = useState(
    location.pathname.startsWith(appPaths.workspaceRoot)
      ? location.pathname
      : appPaths.workspace,
  )

  useEffect(() => {
    if (location.pathname.startsWith(appPaths.workspaceRoot)) {
      // Keep the last workspace child route stable while the root shell exits,
      // otherwise the layout re-evaluates against "/" mid-transition and visibly reflows.
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setActiveWorkspacePath(location.pathname)
    }
  }, [location.pathname])

  const handleRailContentChange = useCallback(
    (content: WorkspaceRailContent | null) => {
      if (content === null && shouldShowWorkspaceRail(activeWorkspacePath)) {
        return
      }

      setRailContent(content)
    },
    [activeWorkspacePath],
  )
  const outletContext = useMemo(
    () => ({ setRailContent: handleRailContentChange }),
    [handleRailContentChange],
  )
  const outlet = useOutlet(outletContext)
  const [displayedWorkspacePath, setDisplayedWorkspacePath] = useState(activeWorkspacePath)
  const [displayedOutlet, setDisplayedOutlet] = useState<ReactNode>(outlet)
  const [displayedRailContent, setDisplayedRailContent] = useState<WorkspaceRailContent | null>(
    null,
  )
  const [stageCycle, setStageCycle] = useState(0)
  const [stagePhase, setStagePhase] = useState<WorkspaceStagePhase>(
    prefersReducedMotion ? 'idle' : 'entering',
  )
  const [pendingStage, setPendingStage] = useState<{
    outlet: ReactNode
    path: string
  } | null>(null)
  const isExitingStage = stagePhase === 'exiting'
  const exitCommitRef = useRef(false)

  useEffect(() => {
    if (activeWorkspacePath === displayedWorkspacePath || stagePhase === 'exiting') {
      return
    }

    // Freeze the currently displayed workspace panel until its exit animation finishes,
    // otherwise React Router swaps the outlet immediately and the new panel fades in late.
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setPendingStage({
      outlet,
      path: activeWorkspacePath,
    })
    setStagePhase('exiting')
  }, [activeWorkspacePath, displayedWorkspacePath, outlet, stagePhase])

  useEffect(() => {
    if (stagePhase === 'exiting') {
      return
    }

    // Keep the rail content in sync with the currently displayed page only after
    // the page switch has committed, so the old rail stays stable during exit.
    // eslint-disable-next-line react-hooks/set-state-in-effect
    setDisplayedRailContent(railContent)
  }, [railContent, stagePhase])

  useEffect(() => {
    if (!prefersReducedMotion || stagePhase !== 'entering') {
      return
    }

    const frame = window.requestAnimationFrame(() => {
      setStagePhase('idle')
    })

    return () => {
      window.cancelAnimationFrame(frame)
    }
  }, [prefersReducedMotion, stagePhase])

  useEffect(() => {
    if (stagePhase !== 'exiting') {
      exitCommitRef.current = false
    }
  }, [stagePhase])

  const isMobileRailOpen = openRailPath === displayedWorkspacePath
  const shouldShowRail = shouldShowWorkspaceRail(displayedWorkspacePath)
  const hasRailContent = displayedRailContent !== null
  const mainStageKey = `${displayedWorkspacePath}:${stageCycle}:main`
  const railStageKey = `${displayedWorkspacePath}:${stageCycle}:${hasRailContent ? 'content' : 'skeleton'}:rail`
  const enterTransition = prefersReducedMotion
    ? { duration: 0 }
    : { delay: 0.04, duration: 0.28, ease: panelEase }
  const railEnterTransition = prefersReducedMotion
    ? { duration: 0 }
    : { delay: 0.14, duration: 0.3, ease: panelEase }

  function handleStageAnimationComplete() {
    if (stagePhase !== 'exiting' || pendingStage === null || exitCommitRef.current) {
      return
    }

    exitCommitRef.current = true
    setDisplayedWorkspacePath(pendingStage.path)
    setDisplayedOutlet(pendingStage.outlet)
    setDisplayedRailContent(null)
    setRailContent(null)
    setOpenRailPath(null)
    setPendingStage(null)
    setStageCycle((current) => current + 1)
    setStagePhase(prefersReducedMotion ? 'idle' : 'entering')
  }

  function handleEnterAnimationComplete() {
    if (stagePhase !== 'entering') {
      return
    }

    setStagePhase('idle')
  }

  return (
    <section className="flex h-full min-h-0 w-full flex-1 overflow-visible py-6 sm:py-8">
      <div
        className={
          shouldShowRail
            ? 'grid h-full min-h-0 w-full gap-5 overflow-visible lg:grid-cols-[13rem_minmax(0,1fr)] xl:grid-cols-[14rem_minmax(0,1fr)_15rem]'
            : 'grid h-full min-h-0 w-full gap-5 overflow-visible lg:grid-cols-[13rem_minmax(0,1fr)] xl:grid-cols-[14rem_minmax(0,1fr)]'
        }
      >
        <WorkspaceSidebar
          ariaLabel={t('workspace.sidebar.title')}
          items={workspaceNavigationItems.map((item) => ({
            icon: <FontAwesomeIcon fixedWidth icon={item.icon} />,
            label: item.label,
            to: item.to,
          }))}
        />

        <div
          className={
            shouldShowRail
              ? 'grid h-full min-h-0 min-w-0 gap-5 overflow-visible xl:col-span-2 xl:grid-cols-[minmax(0,1fr)_15rem]'
              : 'h-full min-h-0 min-w-0 overflow-visible'
          }
        >
          <motion.div
            animate={
              prefersReducedMotion
                ? { opacity: 1, x: 0, y: 0 }
                : isExitingStage
                  ? { opacity: 0, x: -18, y: 14 }
                  : { opacity: 1, x: 0, y: 0 }
            }
            className="flex h-full min-h-0 min-w-0 flex-col space-y-4"
            initial={prefersReducedMotion ? { opacity: 1, x: 0, y: 0 } : { opacity: 0, x: 18, y: 14 }}
            key={mainStageKey}
            onAnimationComplete={() => {
              if (isExitingStage) {
                handleStageAnimationComplete()
              } else if (!shouldShowRail) {
                handleEnterAnimationComplete()
              }
            }}
            transition={isExitingStage ? { duration: 0.24, ease: panelEase } : enterTransition}
          >
            {hasRailContent ? (
              <div className="xl:hidden">
                <Button
                  aria-expanded={isMobileRailOpen}
                  onClick={() => {
                    setOpenRailPath((current) =>
                      current === displayedWorkspacePath ? null : displayedWorkspacePath,
                    )
                  }}
                  size="sm"
                  variant="secondary"
                >
                  {isMobileRailOpen
                    ? t('workspace.rail.actions.close')
                    : t('workspace.rail.actions.open')}
                </Button>

                {isMobileRailOpen && displayedRailContent ? (
                  <motion.div
                    animate={prefersReducedMotion ? { opacity: 1, y: 0 } : { opacity: 1, y: 0 }}
                    className="mt-4"
                    initial={prefersReducedMotion ? { opacity: 1, y: 0 } : { opacity: 0, y: -10 }}
                    transition={
                      prefersReducedMotion
                        ? { duration: 0 }
                        : { duration: 0.22, ease: [0.22, 1, 0.36, 1] }
                    }
                  >
                    <WorkspaceRail content={displayedRailContent} />
                  </motion.div>
                ) : null}
              </div>
            ) : null}

            <div className="min-h-0 flex-1">
              <Suspense fallback={<WorkspaceContentFallback pathname={displayedWorkspacePath} />}>
                {displayedOutlet}
              </Suspense>
            </div>
          </motion.div>

          {shouldShowRail ? (
            hasRailContent && displayedRailContent ? (
              <motion.div
                animate={
                  prefersReducedMotion
                    ? { opacity: 1, x: 0, y: 0 }
                    : isExitingStage
                      ? { opacity: 0, x: -18, y: 14 }
                      : { opacity: 1, x: 0, y: 0 }
                }
                className="hidden xl:block xl:h-full xl:min-h-0"
                initial={
                  prefersReducedMotion ? { opacity: 1, x: 0, y: 0 } : { opacity: 0, x: 24, y: 18 }
                }
                key={railStageKey}
                onAnimationComplete={() => {
                  if (isExitingStage) {
                    handleStageAnimationComplete()
                  } else {
                    handleEnterAnimationComplete()
                  }
                }}
                transition={isExitingStage ? { duration: 0.24, ease: panelEase } : railEnterTransition}
              >
                <WorkspaceRail className="xl:h-full xl:min-h-0" content={displayedRailContent} />
              </motion.div>
            ) : (
              <motion.div
                animate={
                  prefersReducedMotion
                    ? { opacity: 1, x: 0, y: 0 }
                    : isExitingStage
                      ? { opacity: 0, x: -18, y: 14 }
                      : { opacity: 1, x: 0, y: 0 }
                }
                className="hidden xl:block xl:h-full xl:min-h-0"
                initial={
                  prefersReducedMotion ? { opacity: 1, x: 0, y: 0 } : { opacity: 0, x: 24, y: 18 }
                }
                key={railStageKey}
                onAnimationComplete={() => {
                  if (isExitingStage) {
                    handleStageAnimationComplete()
                  } else {
                    handleEnterAnimationComplete()
                  }
                }}
                transition={isExitingStage ? { duration: 0.24, ease: panelEase } : railEnterTransition}
              >
                <WorkspaceRailSkeleton className="xl:h-full xl:min-h-0" />
              </motion.div>
            )
          ) : null}
        </div>
      </div>
    </section>
  )
}
