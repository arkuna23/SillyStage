import { createContext, useContext, useEffect, useMemo, useRef } from 'react'

export type ToastTone = 'error' | 'success' | 'warning'

export type ToastInput = {
  message: string
  tone?: ToastTone
}

export type ToastContextValue = {
  pushToast: (toast: ToastInput) => void
}

export const toastContext = createContext<ToastContextValue | null>(null)

export function useToast() {
  const context = useContext(toastContext)

  if (!context) {
    throw new Error('useToast must be used within ToastProvider')
  }

  return context
}

export function useToastNotice<T extends ToastInput | null>(notice: T) {
  const { pushToast } = useToast()
  const lastNoticeRef = useRef<T>(null)

  useEffect(() => {
    if (!notice || lastNoticeRef.current === notice) {
      return
    }

    pushToast(notice)
    lastNoticeRef.current = notice
  }, [notice, pushToast])
}

export function useToastMessage(message: string | null, tone: ToastTone = 'error') {
  const notice = useMemo(() => (message ? { message, tone } : null), [message, tone])

  useToastNotice(notice)
}
