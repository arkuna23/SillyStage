import { useEffect, useState } from 'react'
import type { ReactNode } from 'react'
import { faHouse } from '@fortawesome/free-solid-svg-icons/faHouse'
import { faTableColumns } from '@fortawesome/free-solid-svg-icons/faTableColumns'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { NavLink, useLocation, useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'

import { appPaths } from '../../app/paths'
import { HeadbarMenu } from '../headbar-menu'
import { cn } from '../../lib/cn'
import { SegmentedSelector } from '../ui/segmented-selector'

const navigationItems = [
  { icon: <FontAwesomeIcon fixedWidth icon={faHouse} />, key: 'nav.home', to: appPaths.home },
  {
    icon: <FontAwesomeIcon fixedWidth icon={faTableColumns} />,
    key: 'nav.workspace',
    to: appPaths.workspace,
  },
] as const satisfies ReadonlyArray<{
  icon: ReactNode
  key: 'nav.home' | 'nav.workspace'
  to: typeof appPaths.home | typeof appPaths.workspace
}>

export function Headbar() {
  const { t } = useTranslation()
  const [isAtTop, setIsAtTop] = useState(true)
  const location = useLocation()
  const navigate = useNavigate()

  const activeNavigation = location.pathname.startsWith(appPaths.workspaceRoot)
    ? appPaths.workspace
    : appPaths.home

  useEffect(() => {
    const handleScroll = () => {
      setIsAtTop(window.scrollY <= 8)
    }

    handleScroll()
    window.addEventListener('scroll', handleScroll, { passive: true })

    return () => {
      window.removeEventListener('scroll', handleScroll)
    }
  }, [])

  return (
    <header
      className={cn(
        'fixed inset-x-0 z-50 transition-all duration-300 ease-out',
        isAtTop ? 'top-0' : 'top-3',
      )}
    >
      <div
        className={cn(
          'mx-auto transition-all duration-300 ease-out',
          'max-w-6xl px-4 sm:px-5 lg:px-6',
          isAtTop ? 'pt-0' : 'pt-1',
        )}
      >
        <div
          className={cn(
            'grid items-center gap-2.5 border transition-all duration-300 ease-out md:grid-cols-[minmax(0,1fr)_auto_minmax(0,1fr)]',
            isAtTop
              ? 'rounded-b-[1.6rem] border-[var(--color-border-subtle)] border-t-0 bg-[color-mix(in_srgb,var(--color-bg-panel)_92%,transparent)] px-4 py-2.5 shadow-[0_12px_34px_rgba(0,0,0,0.16)] backdrop-blur sm:px-5'
              : 'rounded-[1.6rem] border-[var(--color-border-subtle)] bg-[var(--color-bg-panel-strong)] px-4 py-2 shadow-[0_18px_48px_rgba(0,0,0,0.26)] backdrop-blur sm:px-5',
          )}
        >
          <NavLink
            className="flex min-w-0 items-center md:justify-self-start"
            to={appPaths.home}
          >
            <p className="font-display text-[1.15rem] leading-none text-[var(--color-text-primary)]">
              SillyStage
            </p>
          </NavLink>

          <div className="order-3 flex w-full justify-center md:order-none md:justify-self-center">
            <SegmentedSelector
              ariaLabel={t('common.navigation')}
              className="w-full justify-center md:w-auto md:shrink-0"
              items={navigationItems.map((item) => ({
                icon: item.icon,
                label: t(item.key),
                value: item.to,
              }))}
              layoutId="headbar-route-selector"
              onValueChange={(nextRoute) => {
                if (nextRoute === activeNavigation) {
                  return
                }

                navigate(nextRoute)
              }}
              value={activeNavigation}
            />
          </div>

          <HeadbarMenu className="md:justify-self-end" />
        </div>
      </div>
    </header>
  )
}
