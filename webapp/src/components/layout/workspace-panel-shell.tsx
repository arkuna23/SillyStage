import type { HTMLAttributes } from 'react'

import { cn } from '../../lib/cn'

export function WorkspacePanelShell({
  className,
  ...props
}: HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn(
        'relative min-h-0 overflow-visible rounded-[1.75rem] shadow-[var(--shadow-surface)]',
        className,
      )}
      {...props}
    />
  )
}
