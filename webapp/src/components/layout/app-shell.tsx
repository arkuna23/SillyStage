import { AnimatePresence, motion, useReducedMotion } from 'framer-motion'
import { Suspense } from 'react'
import { useLocation, useOutlet } from 'react-router-dom'

import { appPaths } from '../../app/paths'
import { Headbar } from './headbar'

function RouteLoadingFallback() {
  return (
    <div className="flex h-full w-full flex-1 items-center justify-center">
      <div
        aria-label="Loading route"
        className="inline-flex items-center gap-2 rounded-full border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-2 text-sm text-[var(--color-text-secondary)] shadow-[0_12px_30px_rgba(0,0,0,0.12)] backdrop-blur"
        role="status"
      >
        <span className="h-2 w-2 rounded-full bg-[var(--color-accent-gold)] animate-pulse" />
        <span className="h-2 w-2 rounded-full bg-[var(--color-accent-gold-soft)] animate-pulse [animation-delay:120ms]" />
        <span className="h-2 w-2 rounded-full bg-[var(--color-accent-copper-soft)] animate-pulse [animation-delay:220ms]" />
      </div>
    </div>
  )
}

export function AppShell() {
  const location = useLocation()
  const outlet = useOutlet()
  const prefersReducedMotion = useReducedMotion()
  const isWorkspaceRoute = location.pathname.startsWith(appPaths.workspaceRoot)
  const routeTransitionKey = location.pathname.startsWith(appPaths.workspaceRoot)
    ? appPaths.workspaceRoot
    : location.pathname

  return (
    <div
      className={
        isWorkspaceRoute
          ? 'relative h-screen overflow-visible bg-[var(--color-bg-stage)] text-[var(--color-text-primary)]'
          : 'relative h-screen overflow-hidden bg-[var(--color-bg-stage)] text-[var(--color-text-primary)]'
      }
    >
      <div className="stage-grid pointer-events-none absolute inset-0 opacity-70" />
      <div className="spotlight pointer-events-none absolute -left-12 top-0 h-72 w-72 bg-[var(--color-accent-gold-soft)]" />
      <div className="spotlight pointer-events-none absolute right-0 top-1/4 h-80 w-80 bg-[var(--color-accent-copper-soft)] [animation-duration:20s]" />

      <Headbar />

      <div
        className={
          isWorkspaceRoute
            ? 'relative mx-auto flex h-full max-w-[88rem] flex-col overflow-visible px-4 pb-8 pt-24 sm:px-5 sm:pt-28 lg:px-6 lg:pt-32'
            : 'relative mx-auto flex h-full max-w-[88rem] flex-col overflow-hidden px-4 pb-8 pt-24 sm:px-5 sm:pt-28 lg:px-6 lg:pt-32'
        }
      >
        <AnimatePresence initial={false} mode="wait">
          <motion.div
            animate={{ opacity: 1 }}
            className={
              isWorkspaceRoute
                ? 'flex h-full min-h-0 flex-1 overflow-visible'
                : 'flex h-full min-h-0 flex-1 overflow-hidden'
            }
            exit={
              prefersReducedMotion ? { opacity: 1 } : { opacity: 0 }
            }
            initial={
              prefersReducedMotion ? { opacity: 1 } : { opacity: 0 }
            }
            key={routeTransitionKey}
            transition={
              prefersReducedMotion
                ? { duration: 0 }
                : { duration: 0.24, ease: [0.22, 1, 0.36, 1] }
            }
          >
            <div
              className={
                isWorkspaceRoute
                  ? 'flex h-full min-h-0 w-full flex-1 overflow-visible'
                  : 'flex h-full min-h-0 w-full flex-1 overflow-hidden'
              }
            >
              <Suspense fallback={<RouteLoadingFallback />}>{outlet}</Suspense>
            </div>
          </motion.div>
        </AnimatePresence>
      </div>
    </div>
  )
}
