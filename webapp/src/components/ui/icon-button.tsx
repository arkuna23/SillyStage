import type { ReactNode } from 'react'

import { cn } from '../../lib/cn'
import { Button, type ButtonProps } from './button'

type IconButtonProps = Omit<ButtonProps, 'children'> & {
  icon: ReactNode
  label: string
}

const iconSizeClasses = {
  lg: 'w-12 px-0',
  md: 'w-11 px-0',
  sm: 'w-9 px-0',
} as const

export function IconButton({
  className,
  icon,
  label,
  size = 'md',
  ...props
}: IconButtonProps) {
  return (
    <Button
      aria-label={label}
      className={cn(iconSizeClasses[size], className)}
      size={size}
      title={label}
      {...props}
    >
      <span aria-hidden="true" className="inline-flex items-center justify-center">
        {icon}
      </span>
    </Button>
  )
}
