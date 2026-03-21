import * as DropdownMenu from '@radix-ui/react-dropdown-menu'
import { type ComponentPropsWithoutRef, type ElementRef, forwardRef } from 'react'

import { cn } from '../../lib/cn'

export const PopupMenu = DropdownMenu.Root
export const PopupMenuPortal = DropdownMenu.Portal
export const PopupMenuTrigger = DropdownMenu.Trigger
export const PopupMenuRadioGroup = DropdownMenu.RadioGroup

export const PopupMenuItem = forwardRef<
  ElementRef<typeof DropdownMenu.Item>,
  ComponentPropsWithoutRef<typeof DropdownMenu.Item>
>(function PopupMenuItem({ className, ...props }, ref) {
  return (
    <DropdownMenu.Item
      className={cn(
        'relative flex cursor-pointer select-none items-center gap-3 rounded-[1rem] px-2.5 py-2.5 text-sm text-[var(--color-text-secondary)] outline-none transition focus:bg-white/8 focus:text-[var(--color-text-primary)] data-[disabled]:pointer-events-none data-[disabled]:opacity-40',
        className,
      )}
      ref={ref}
      {...props}
    />
  )
})

export const PopupMenuContent = forwardRef<
  ElementRef<typeof DropdownMenu.Content>,
  ComponentPropsWithoutRef<typeof DropdownMenu.Content>
>(function PopupMenuContent({ className, sideOffset = 10, ...props }, ref) {
  return (
    <PopupMenuPortal>
      <DropdownMenu.Content
        className={cn(
          'popup-menu-content z-50 min-w-56 origin-[var(--radix-dropdown-menu-content-transform-origin)] rounded-[1.4rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel-strong)] p-2 text-[var(--color-text-primary)] shadow-[var(--shadow-floating)] outline-none backdrop-blur-xl',
          className,
        )}
        ref={ref}
        sideOffset={sideOffset}
        {...props}
      />
    </PopupMenuPortal>
  )
})

export const PopupMenuLabel = forwardRef<
  ElementRef<typeof DropdownMenu.Label>,
  ComponentPropsWithoutRef<typeof DropdownMenu.Label>
>(function PopupMenuLabel({ className, ...props }, ref) {
  return (
    <DropdownMenu.Label
      className={cn(
        'px-2.5 py-2 text-[0.68rem] font-medium uppercase text-[var(--color-text-muted)]',
        className,
      )}
      ref={ref}
      {...props}
    />
  )
})

export const PopupMenuSeparator = forwardRef<
  ElementRef<typeof DropdownMenu.Separator>,
  ComponentPropsWithoutRef<typeof DropdownMenu.Separator>
>(function PopupMenuSeparator({ className, ...props }, ref) {
  return (
    <DropdownMenu.Separator
      className={cn('my-2 h-px bg-[var(--color-border-subtle)]', className)}
      ref={ref}
      {...props}
    />
  )
})

export const PopupMenuRadioItem = forwardRef<
  ElementRef<typeof DropdownMenu.RadioItem>,
  ComponentPropsWithoutRef<typeof DropdownMenu.RadioItem>
>(function PopupMenuRadioItem({ children, className, ...props }, ref) {
  return (
    <DropdownMenu.RadioItem
      className={cn(
        'relative flex cursor-pointer select-none items-center gap-3 rounded-[1rem] px-2.5 py-2.5 text-sm text-[var(--color-text-secondary)] outline-none transition focus:bg-white/8 focus:text-[var(--color-text-primary)] data-[state=checked]:bg-[var(--color-accent-gold-soft)] data-[state=checked]:text-[var(--color-text-primary)]',
        className,
      )}
      ref={ref}
      {...props}
    >
      <span className="flex h-4 w-4 items-center justify-center rounded-full border border-[var(--color-border-subtle)] bg-white/5">
        <DropdownMenu.ItemIndicator className="flex h-2 w-2 items-center justify-center leading-none">
          <span className="h-2 w-2 rounded-full bg-[var(--color-accent-gold)]" />
        </DropdownMenu.ItemIndicator>
      </span>
      <span className="flex-1">{children}</span>
    </DropdownMenu.RadioItem>
  )
})
