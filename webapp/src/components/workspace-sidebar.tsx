import type { ReactNode } from 'react'
import { NavLink } from 'react-router-dom'

import { cn } from '../lib/cn'

export type WorkspaceSidebarItem = {
  icon?: ReactNode
  label: string
  to: string
}

type WorkspaceSidebarProps = {
  ariaLabel: string
  items: ReadonlyArray<WorkspaceSidebarItem>
}

export function WorkspaceSidebar({
  ariaLabel,
  items,
}: WorkspaceSidebarProps) {
  return (
    <aside className="h-full min-h-0">
      <nav
        aria-label={ariaLabel}
        className="scrollbar-none flex h-full min-h-0 gap-3 overflow-x-auto py-2 lg:flex-col lg:overflow-x-hidden lg:overflow-y-auto lg:pr-3"
      >
        {items.map((item) => (
          <NavLink
            className={({ isActive }) =>
              cn(
                'group min-w-[13rem] rounded-[1.35rem] border px-3.5 py-3.5 transition duration-200 ease-out lg:min-w-0',
                isActive
                  ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] text-[var(--color-text-primary)] shadow-[0_18px_46px_var(--color-accent-glow-soft)]'
                  : 'border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_82%,transparent)] text-[var(--color-text-secondary)] hover:border-[var(--color-accent-copper-soft)] hover:text-[var(--color-text-primary)]',
              )
            }
            key={item.to}
            to={item.to}
          >
            {({ isActive }) => (
              <div className="flex items-center justify-between gap-2.5">
                <div className="flex min-w-0 items-center gap-3">
                  {item.icon ? (
                    <span
                      aria-hidden="true"
                      className={cn(
                        'inline-flex size-4 shrink-0 items-center justify-center transition',
                        isActive
                          ? 'text-[var(--color-text-primary)]'
                          : 'text-[var(--color-text-muted)] group-hover:text-[var(--color-text-secondary)]',
                      )}
                    >
                      {item.icon}
                    </span>
                  ) : null}
                  <span className="truncate text-sm font-medium leading-5">
                    {item.label}
                  </span>
                </div>
                <span
                  aria-hidden="true"
                  className={cn(
                    'h-2.5 w-2.5 rounded-full transition',
                    isActive
                      ? 'bg-[var(--color-accent-gold)] shadow-[0_0_0_4px_var(--color-accent-gold-soft)]'
                      : 'bg-[var(--color-border-subtle)] group-hover:bg-[var(--color-accent-copper)]',
                  )}
                />
              </div>
            )}
          </NavLink>
        ))}
      </nav>
    </aside>
  )
}
