import { faXmark } from '@fortawesome/free-solid-svg-icons/faXmark'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { AnimatePresence, motion, useReducedMotion } from 'framer-motion'
import {
  type PropsWithChildren,
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
} from 'react'
import { createPortal } from 'react-dom'

import { cn } from '../../lib/cn'
import { toastContext, type ToastInput } from './toast-context'

type ToastRecord = ToastInput & {
  id: string
}

const TOAST_DISMISS_MS = {
  error: 6200,
  success: 4200,
  warning: 5200,
} as const

function ToastCard({
  onDismiss,
  toast,
}: {
  onDismiss: (id: string) => void
  toast: ToastRecord
}) {
  const prefersReducedMotion = useReducedMotion()

  useEffect(() => {
    const timeoutId = window.setTimeout(() => {
      onDismiss(toast.id)
    }, TOAST_DISMISS_MS[toast.tone ?? 'success'])

    return () => {
      window.clearTimeout(timeoutId)
    }
  }, [onDismiss, toast.id, toast.tone])

  return (
    <motion.div
      animate={{ opacity: 1, x: 0, y: 0, scale: 1 }}
      className={cn(
        'pointer-events-auto flex w-[min(24rem,calc(100vw-2rem))] items-center gap-3 rounded-[1.45rem] border px-4 py-3 shadow-[0_18px_42px_rgba(0,0,0,0.18)] backdrop-blur-xl',
        toast.tone === 'success'
          ? 'border-[var(--color-state-success-line)] bg-[color-mix(in_srgb,var(--color-state-success-soft)_90%,var(--color-bg-panel))] text-[var(--color-text-primary)]'
          : toast.tone === 'warning'
            ? 'border-[var(--color-state-warning-line)] bg-[color-mix(in_srgb,var(--color-state-warning-soft)_90%,var(--color-bg-panel))] text-[var(--color-text-primary)]'
            : 'border-[var(--color-state-error-line)] bg-[color-mix(in_srgb,var(--color-state-error-soft)_90%,var(--color-bg-panel))] text-[var(--color-text-primary)]',
      )}
      exit={
        prefersReducedMotion
          ? { opacity: 0 }
          : { opacity: 0, x: -18, y: 8, scale: 0.98 }
      }
      initial={
        prefersReducedMotion
          ? { opacity: 0 }
          : { opacity: 0, x: -22, y: 10, scale: 0.97 }
      }
      layout
      transition={
        prefersReducedMotion
          ? { duration: 0.14 }
          : { duration: 0.22, ease: [0.22, 1, 0.36, 1] }
      }
    >
      <div className="flex min-w-0 flex-1 items-center gap-3">
        <span
          aria-hidden="true"
          className={cn(
            'inline-flex h-2.5 w-2.5 shrink-0 self-center rounded-full',
            toast.tone === 'success'
              ? 'bg-[var(--color-state-success)]'
              : toast.tone === 'warning'
                ? 'bg-[var(--color-state-warning)]'
                : 'bg-[var(--color-state-error)]',
          )}
        />

        <p className="min-w-0 flex-1 text-sm leading-6 text-[var(--color-text-primary)]">
          {toast.message}
        </p>
      </div>

      <button
        aria-label="Dismiss notification"
        className="inline-flex h-8 w-8 shrink-0 self-center items-center justify-center rounded-full border border-transparent text-[var(--color-text-muted)] transition hover:border-[var(--color-border-subtle)] hover:bg-[var(--color-bg-elevated)] hover:text-[var(--color-text-primary)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-focus-ring)]"
        onClick={() => {
          onDismiss(toast.id)
        }}
        type="button"
      >
        <FontAwesomeIcon icon={faXmark} />
      </button>
    </motion.div>
  )
}

function ToastViewport({
  onDismiss,
  toasts,
}: {
  onDismiss: (id: string) => void
  toasts: ToastRecord[]
}) {
  if (typeof document === 'undefined') {
    return null
  }

  return createPortal(
    <div className="pointer-events-none fixed bottom-4 left-4 z-[90] flex max-h-[calc(100vh-2rem)] flex-col-reverse gap-3">
      <AnimatePresence initial={false}>
        {toasts.map((toast) => (
          <ToastCard key={toast.id} onDismiss={onDismiss} toast={toast} />
        ))}
      </AnimatePresence>
    </div>,
    document.body,
  )
}

export function ToastProvider({ children }: PropsWithChildren) {
  const [toasts, setToasts] = useState<ToastRecord[]>([])
  const nextToastIdRef = useRef(0)

  const pushToast = useCallback((toast: ToastInput) => {
    const tone = toast.tone ?? 'success'

    setToasts((currentToasts) => [
      ...currentToasts,
      {
        ...toast,
        id: `toast:${nextToastIdRef.current++}`,
        tone,
      },
    ])
  }, [])

  const dismissToast = useCallback((toastId: string) => {
    setToasts((currentToasts) => currentToasts.filter((toast) => toast.id !== toastId))
  }, [])

  const value = useMemo(
    () => ({
      pushToast,
    }),
    [pushToast],
  )

  return (
    <toastContext.Provider value={value}>
      {children}
      <ToastViewport onDismiss={dismissToast} toasts={toasts} />
    </toastContext.Provider>
  )
}
