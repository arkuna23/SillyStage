import * as DialogPrimitive from '@radix-ui/react-dialog'
import {
  type ComponentPropsWithoutRef,
  type ElementRef,
  forwardRef,
} from 'react'

import { cn } from '../../lib/cn'

export const Dialog = DialogPrimitive.Root
export const DialogTrigger = DialogPrimitive.Trigger
export const DialogPortal = DialogPrimitive.Portal
export const DialogClose = DialogPrimitive.Close

export const DialogOverlay = forwardRef<
  ElementRef<typeof DialogPrimitive.Overlay>,
  ComponentPropsWithoutRef<typeof DialogPrimitive.Overlay>
>(function DialogOverlay({ className, ...props }, ref) {
  return (
    <DialogPrimitive.Overlay
      className={cn(
        'dialog-overlay fixed inset-0 z-50 bg-[rgba(11,8,16,0.62)] backdrop-blur-[3px]',
        className,
      )}
      ref={ref}
      {...props}
    />
  )
})

export const DialogContent = forwardRef<
  ElementRef<typeof DialogPrimitive.Content>,
  ComponentPropsWithoutRef<typeof DialogPrimitive.Content>
>(function DialogContent({ children, className, ...props }, ref) {
  return (
    <DialogPortal>
      <DialogOverlay />
      <DialogPrimitive.Content
        className={cn(
          'dialog-content fixed left-1/2 top-1/2 z-50 w-[min(92vw,56rem)] -translate-x-1/2 -translate-y-1/2 outline-none',
        )}
        ref={ref}
        {...props}
      >
        <div
          className={cn(
            'dialog-surface w-full rounded-[1.9rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel-strong)] shadow-[var(--shadow-dialog)] backdrop-blur-xl',
            className,
          )}
        >
          {children}
        </div>
      </DialogPrimitive.Content>
    </DialogPortal>
  )
})

export const DialogTitle = forwardRef<
  ElementRef<typeof DialogPrimitive.Title>,
  ComponentPropsWithoutRef<typeof DialogPrimitive.Title>
>(function DialogTitle({ className, ...props }, ref) {
  return (
    <DialogPrimitive.Title
      className={cn(
        'font-display text-3xl leading-tight text-[var(--color-text-primary)]',
        className,
      )}
      ref={ref}
      {...props}
    />
  )
})

export const DialogDescription = forwardRef<
  ElementRef<typeof DialogPrimitive.Description>,
  ComponentPropsWithoutRef<typeof DialogPrimitive.Description>
>(function DialogDescription({ className, ...props }, ref) {
  return (
    <DialogPrimitive.Description
      className={cn('text-sm leading-7 text-[var(--color-text-secondary)]', className)}
      ref={ref}
      {...props}
    />
  )
})

export function DialogHeader({
  className,
  ...props
}: ComponentPropsWithoutRef<'div'>) {
  return <div className={cn('space-y-3 p-6 md:p-7', className)} {...props} />
}

export function DialogBody({ className, ...props }: ComponentPropsWithoutRef<'div'>) {
  return <div className={cn('scrollbar-none px-6 pb-6 md:px-7 md:pb-7', className)} {...props} />
}

export function DialogFooter({
  className,
  ...props
}: ComponentPropsWithoutRef<'div'>) {
  return (
    <div
      className={cn(
        'flex flex-col-reverse gap-3 border-t border-[var(--color-border-subtle)] px-6 py-5 sm:flex-row sm:justify-between md:px-7',
        className,
      )}
      {...props}
    />
  )
}
