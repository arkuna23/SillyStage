import { useEffect, useRef, useState } from 'react'
import type { ComponentPropsWithoutRef, ReactNode } from 'react'
import { useNavigate } from 'react-router-dom'

import { DIALOG_EXIT_DURATION_MS } from './dialog'
import { Button } from './button'

type DialogRouteButtonProps = Omit<
  ComponentPropsWithoutRef<typeof Button>,
  'asChild' | 'children' | 'onClick'
> & {
  children: ReactNode
  onRequestClose: () => void
  to: string
}

export function DialogRouteButton({
  children,
  disabled,
  onRequestClose,
  to,
  ...props
}: DialogRouteButtonProps) {
  const navigate = useNavigate()
  const navigateRef = useRef(navigate)
  const timeoutRef = useRef<number | null>(null)
  const shouldPreserveNavigationRef = useRef(false)
  const [isNavigating, setIsNavigating] = useState(false)

  useEffect(() => {
    navigateRef.current = navigate
  }, [navigate])

  useEffect(() => {
    return () => {
      if (timeoutRef.current !== null && !shouldPreserveNavigationRef.current) {
        window.clearTimeout(timeoutRef.current)
      }
    }
  }, [])

  return (
    <Button
      {...props}
      disabled={disabled || isNavigating}
      onClick={() => {
        if (disabled || isNavigating) {
          return
        }

        setIsNavigating(true)
        shouldPreserveNavigationRef.current = true
        onRequestClose()

        timeoutRef.current = window.setTimeout(() => {
          window.requestAnimationFrame(() => {
            navigateRef.current(to)
          })
        }, DIALOG_EXIT_DURATION_MS + 24)
      }}
    >
      {children}
    </Button>
  )
}
