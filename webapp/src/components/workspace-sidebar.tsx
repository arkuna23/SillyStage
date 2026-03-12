import { NavLink } from 'react-router-dom'

import { cn } from '../lib/cn'
import { Card } from './ui/card'

export type WorkspaceSidebarItem = {
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
    <aside className="panel-enter lg:sticky lg:top-[7.5rem] lg:self-start">
      <Card className="overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_90%,transparent)]">
        <nav
          aria-label={ariaLabel}
          className="flex gap-3 overflow-x-auto px-3 py-3 lg:flex-col"
        >
          {items.map((item) => (
            <NavLink
              className={({ isActive }) =>
                cn(
                  'group min-w-[16rem] rounded-[1.45rem] border px-4 py-4 transition duration-200 ease-out lg:min-w-0',
                  isActive
                    ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] text-[var(--color-text-primary)] shadow-[0_18px_46px_rgba(217,167,74,0.12)]'
                    : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] text-[var(--color-text-secondary)] hover:border-[var(--color-accent-copper-soft)] hover:text-[var(--color-text-primary)]',
                )
              }
              key={item.to}
              to={item.to}
            >
              {({ isActive }) => (
                <div className="flex items-center justify-between gap-3">
                  <span className="text-sm font-medium">
                    {item.label}
                  </span>
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
      </Card>
    </aside>
  )
}
