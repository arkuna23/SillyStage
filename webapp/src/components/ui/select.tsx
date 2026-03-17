import * as SelectPrimitive from '@radix-ui/react-select'
import { useMemo, type ComponentPropsWithoutRef } from 'react'

import { cn } from '../../lib/cn'

type SelectOption = {
  label: string
  value: string
}

type SelectProps = Omit<
  ComponentPropsWithoutRef<typeof SelectPrimitive.Root>,
  'children' | 'onValueChange' | 'value'
> & {
  allowClear?: boolean
  clearLabel?: string
  items: SelectOption[]
  onValueChange?: (value: string) => void
  placeholder?: string
  textAlign?: 'start' | 'center'
  triggerId?: string
  triggerClassName?: string
  value?: string
}

const clearValuePrefix = '__select_clear__'

function createClearValue(items: ReadonlyArray<SelectOption>) {
  let index = 0
  let nextValue = clearValuePrefix

  while (items.some((item) => item.value === nextValue)) {
    index += 1
    nextValue = `${clearValuePrefix}_${index}`
  }

  return nextValue
}

function ChevronDownIcon() {
  return (
    <svg
      aria-hidden="true"
      className="h-4 w-4"
      fill="none"
      viewBox="0 0 16 16"
      xmlns="http://www.w3.org/2000/svg"
    >
      <path
        d="M4 6.5L8 10.5L12 6.5"
        stroke="currentColor"
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth="1.5"
      />
    </svg>
  )
}

export function Select({
  allowClear = false,
  clearLabel,
  items,
  onValueChange,
  placeholder,
  textAlign = 'start',
  triggerClassName,
  triggerId,
  value,
  ...props
}: SelectProps) {
  const clearValue = useMemo(() => createClearValue(items), [items])
  const rootValue = typeof value === 'string' && value.length > 0 ? value : undefined

  return (
    <SelectPrimitive.Root
      {...props}
      onValueChange={(nextValue) => {
        onValueChange?.(allowClear && nextValue === clearValue ? '' : nextValue)
      }}
      value={rootValue}
    >
      <SelectPrimitive.Trigger
        className={cn(
          'inline-flex h-12 w-full items-center justify-between gap-3 rounded-2xl border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-[1.125rem] text-sm text-[var(--color-text-primary)] outline-none transition focus:border-[var(--color-accent-copper)] focus:ring-2 focus:ring-[var(--color-focus-ring)] data-[placeholder]:text-[var(--color-text-muted)]',
          triggerClassName,
        )}
        id={triggerId}
      >
        <div
          className={cn(
            'min-w-0 flex-1 pr-2',
            textAlign === 'center' ? 'text-center' : 'text-left',
          )}
        >
          <SelectPrimitive.Value placeholder={placeholder} />
        </div>
        <SelectPrimitive.Icon className="text-[var(--color-text-secondary)]">
          <ChevronDownIcon />
        </SelectPrimitive.Icon>
      </SelectPrimitive.Trigger>

      <SelectPrimitive.Portal>
        <SelectPrimitive.Content
          className="select-content z-50 max-h-[var(--radix-select-content-available-height)] w-[var(--radix-select-trigger-width)] min-w-[var(--radix-select-trigger-width)] origin-[var(--radix-select-content-transform-origin)] overflow-hidden rounded-[1.4rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel-strong)] p-1.5 shadow-[var(--shadow-select)] backdrop-blur"
          position="popper"
          sideOffset={8}
        >
          <SelectPrimitive.Viewport className="scrollbar-none w-full space-y-1.5">
            {allowClear ? (
              <SelectPrimitive.Item
                className={cn(
                  'relative flex cursor-pointer select-none items-center rounded-xl px-4 py-3 text-sm text-[var(--color-text-secondary)] outline-none transition hover:bg-white/10 hover:text-[var(--color-text-primary)] focus:bg-white/10 focus:text-[var(--color-text-primary)]',
                  textAlign === 'center' ? 'justify-center text-center' : 'justify-start text-left',
                )}
                value={clearValue}
              >
                <SelectPrimitive.ItemText>{clearLabel ?? placeholder}</SelectPrimitive.ItemText>
              </SelectPrimitive.Item>
            ) : null}
            {items.map((item) => (
              <SelectPrimitive.Item
                key={item.value}
                className={cn(
                  'relative flex cursor-pointer select-none items-center rounded-xl px-4 py-3 text-sm text-[var(--color-text-primary)] outline-none transition hover:bg-white/10 focus:bg-white/10 data-[state=checked]:text-[var(--color-accent-gold-strong)]',
                  textAlign === 'center' ? 'justify-center text-center' : 'justify-start text-left',
                )}
                value={item.value}
              >
                <SelectPrimitive.ItemText>{item.label}</SelectPrimitive.ItemText>
              </SelectPrimitive.Item>
            ))}
          </SelectPrimitive.Viewport>
        </SelectPrimitive.Content>
      </SelectPrimitive.Portal>
    </SelectPrimitive.Root>
  )
}
