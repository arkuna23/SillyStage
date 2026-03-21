import { motion, useReducedMotion } from 'framer-motion'
import { type ReactNode, useCallback, useEffect, useLayoutEffect, useRef, useState } from 'react'

import { cn } from '../../lib/cn'

type SegmentedSelectorItem = {
  ariaLabel?: string
  disabled?: boolean
  icon?: ReactNode
  label: ReactNode
  value: string
}

type SegmentedSelectorProps = {
  ariaLabel: string
  className?: string
  items: ReadonlyArray<SegmentedSelectorItem>
  layoutId?: string
  onDisabledValueClick?: (value: string) => void
  onValueChange?: (value: string) => void
  value: string
}

export function SegmentedSelector({
  ariaLabel,
  className,
  items,
  layoutId,
  onDisabledValueClick,
  onValueChange,
  value,
}: SegmentedSelectorProps) {
  const prefersReducedMotion = useReducedMotion()
  const containerRef = useRef<HTMLDivElement | null>(null)
  const buttonRefs = useRef(new Map<string, HTMLButtonElement | null>())
  const [activeFrame, setActiveFrame] = useState<{
    height: number
    left: number
    top: number
    width: number
  } | null>(null)

  const setButtonRef = useCallback((buttonValue: string, node: HTMLButtonElement | null) => {
    if (node) {
      buttonRefs.current.set(buttonValue, node)
      return
    }

    buttonRefs.current.delete(buttonValue)
  }, [])

  const updateActiveFrame = useCallback(() => {
    const containerNode = containerRef.current
    const activeButton = buttonRefs.current.get(value)

    if (!containerNode || !activeButton) {
      setActiveFrame(null)
      return
    }

    const nextFrame = {
      height: activeButton.offsetHeight,
      left: activeButton.offsetLeft,
      top: activeButton.offsetTop,
      width: activeButton.offsetWidth,
    }

    setActiveFrame((previousFrame) => {
      if (
        previousFrame &&
        previousFrame.left === nextFrame.left &&
        previousFrame.top === nextFrame.top &&
        previousFrame.width === nextFrame.width &&
        previousFrame.height === nextFrame.height
      ) {
        return previousFrame
      }

      return nextFrame
    })
  }, [value])

  useLayoutEffect(() => {
    const frame = globalThis.requestAnimationFrame(() => {
      updateActiveFrame()
    })

    return () => {
      globalThis.cancelAnimationFrame(frame)
    }
  }, [items, updateActiveFrame])

  useEffect(() => {
    const containerNode = containerRef.current
    const activeButton = buttonRefs.current.get(value)

    if (!containerNode || !activeButton || typeof ResizeObserver === 'undefined') {
      return undefined
    }

    const resizeObserver = new ResizeObserver(() => {
      updateActiveFrame()
    })

    resizeObserver.observe(containerNode)
    resizeObserver.observe(activeButton)

    return () => {
      resizeObserver.disconnect()
    }
  }, [updateActiveFrame, value])

  return (
    <div
      aria-label={ariaLabel}
      className={cn(
        'relative inline-flex items-stretch gap-1 rounded-[1.2rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel-strong)] p-1 shadow-[inset_0_1px_0_rgba(255,255,255,0.02)]',
        className,
      )}
      data-segmented-layout-id={layoutId}
      ref={containerRef}
      role="group"
    >
      {activeFrame ? (
        <motion.span
          animate={{
            height: activeFrame.height,
            left: activeFrame.left,
            top: activeFrame.top,
            width: activeFrame.width,
          }}
          className="pointer-events-none absolute rounded-[0.95rem] border border-[var(--color-accent-gold-line)] bg-[linear-gradient(135deg,color-mix(in_srgb,var(--color-accent-gold)_86%,var(--color-bg-curtain)),color-mix(in_srgb,var(--color-accent-gold-strong)_82%,var(--color-bg-curtain)))] shadow-[0_10px_24px_var(--color-accent-glow-soft)]"
          initial={false}
          transition={
            prefersReducedMotion
              ? { duration: 0 }
              : { damping: 34, mass: 0.34, stiffness: 420, type: 'spring' }
          }
        />
      ) : null}
      {items.map((item) => {
        const selected = item.value === value

        return (
          <button
            aria-label={item.ariaLabel}
            aria-current={selected ? 'page' : undefined}
            aria-disabled={item.disabled || undefined}
            className={cn(
              'relative inline-flex h-10 min-w-[2.9rem] items-center justify-center self-stretch rounded-[0.95rem] px-3 text-[0.82rem] font-medium leading-none transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-focus-ring)] xl:min-w-[6rem] xl:px-3.5',
              selected
                ? 'text-[color:var(--color-accent-ink)]'
                : 'text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]',
              item.disabled ? 'cursor-not-allowed opacity-40' : undefined,
            )}
            key={item.value}
            onClick={() => {
              if (item.disabled) {
                onDisabledValueClick?.(item.value)
                return
              }

              if (item.value === value) {
                return
              }

              onValueChange?.(item.value)
            }}
            type="button"
            ref={(node) => {
              setButtonRef(item.value, node)
            }}
          >
            <span className="relative z-10 inline-flex items-center gap-2 xl:gap-2.5">
              {item.icon ? (
                <span aria-hidden="true" className="inline-flex size-4 items-center justify-center">
                  {item.icon}
                </span>
              ) : null}
              <span>{item.label}</span>
            </span>
          </button>
        )
      })}
    </div>
  )
}
