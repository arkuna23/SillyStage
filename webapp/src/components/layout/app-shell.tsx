import { AnimatePresence, motion, useReducedMotion } from 'framer-motion'
import { Suspense } from 'react'
import { useLocation, useOutlet } from 'react-router-dom'

import { Headbar } from './headbar'

function RouteLoadingFallback() {
  return (
    <div className="flex min-h-[calc(100vh-8rem)] w-full flex-1 items-center justify-center">
      <div
        aria-label="Loading route"
        className="inline-flex items-center gap-2 rounded-full border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-2 text-sm text-[var(--color-text-secondary)] shadow-[0_12px_30px_rgba(0,0,0,0.12)] backdrop-blur"
        role="status"
      >
        <span className="h-2 w-2 rounded-full bg-[var(--color-accent-gold)] animate-pulse" />
        <span className="h-2 w-2 rounded-full bg-[var(--color-accent-gold-soft)] animate-pulse [animation-delay:120ms]" />
        <span className="h-2 w-2 rounded-full bg-[var(--color-info-blue-soft)] animate-pulse [animation-delay:220ms]" />
      </div>
    </div>
  )
}

export function AppShell() {
  const location = useLocation()
  const outlet = useOutlet()
  const prefersReducedMotion = useReducedMotion()

  return (
    <div className="relative min-h-screen overflow-hidden bg-[var(--color-bg-stage)] text-[var(--color-text-primary)]">
      <div className="stage-grid pointer-events-none absolute inset-0 opacity-70" />
      <div className="spotlight pointer-events-none absolute -left-12 top-0 h-72 w-72 bg-[var(--color-accent-gold-soft)]" />
      <div className="spotlight pointer-events-none absolute right-0 top-1/4 h-80 w-80 bg-[var(--color-info-blue-soft)] [animation-duration:20s]" />

      <Headbar />

      <div className="relative mx-auto flex min-h-screen max-w-[88rem] flex-col px-5 pb-8 pt-24 sm:px-8 sm:pt-28 lg:px-10 lg:pt-32">
        <AnimatePresence initial={false} mode="wait">
          <motion.div
            animate={prefersReducedMotion ? { opacity: 1, y: 0 } : { opacity: 1, y: 0 }}
            className="flex flex-1"
            exit={prefersReducedMotion ? { opacity: 1, y: 0 } : { opacity: 0, y: -10 }}
            initial={prefersReducedMotion ? { opacity: 1, y: 0 } : { opacity: 0, y: 16 }}
            key={location.pathname}
            transition={
              prefersReducedMotion
                ? { duration: 0 }
                : { duration: 0.28, ease: [0.16, 1, 0.3, 1] }
            }
          >
            <Suspense fallback={<RouteLoadingFallback />}>{outlet}</Suspense>
          </motion.div>
        </AnimatePresence>
      </div>
    </div>
  )
}
