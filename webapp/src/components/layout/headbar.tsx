import { faMasksTheater } from '@fortawesome/free-solid-svg-icons/faMasksTheater'
import { faTableCellsLarge } from '@fortawesome/free-solid-svg-icons/faTableCellsLarge'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import type { TFunction } from 'i18next'
import { useCallback, useEffect, useRef, useState } from 'react'
import { NavLink, useLocation, useNavigate } from 'react-router-dom'
import { useTranslation } from 'react-i18next'

import { appPaths } from '../../app/paths'
import {
  getStageAccessStatus,
  STAGE_API_AVAILABILITY_REFRESH_EVENT,
  type StageAccessStatus,
} from '../../features/stage/stage-access'
import { HeadbarMenu } from '../headbar-menu'
import { SegmentedSelector } from '../ui/segmented-selector'
import { useToast } from '../ui/toast-context'
import { cn } from '../../lib/cn'

type HeadbarStageAccessStatus = StageAccessStatus | 'checking'

function getStageUnavailableMessage(
  status: Exclude<HeadbarStageAccessStatus, 'checking' | 'ready'>,
  t: TFunction,
) {
  if (status === 'blockedPresets') {
    return t('stage.headbar.presetRequiredWarning')
  }

  return t('stage.headbar.apiRequiredWarning')
}

export function Headbar() {
  const { t } = useTranslation()
  const location = useLocation()
  const navigate = useNavigate()
  const { pushToast } = useToast()
  const [isAtTop, setIsAtTop] = useState(true)
  const [stageAccessStatus, setStageAccessStatus] = useState<HeadbarStageAccessStatus>('checking')
  const lastStageAccessStatusRef = useRef<HeadbarStageAccessStatus>('checking')
  const currentTopLevelPath = location.pathname.startsWith(appPaths.stageRoot)
    ? appPaths.stage
    : appPaths.workspace

  const loadStageAccessStatus = useCallback(async (signal?: AbortSignal) => {
    try {
      return await getStageAccessStatus(signal)
    } catch {
      if (signal?.aborted) {
        return null
      }

      return null
    }
  }, [])

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

  useEffect(() => {
    const controller = new AbortController()
    void (async () => {
      const nextStatus = await loadStageAccessStatus(controller.signal)

      if (controller.signal.aborted) {
        return
      }

      if (nextStatus === null) {
        return
      }

      setStageAccessStatus(nextStatus)

      if (
        lastStageAccessStatusRef.current !== nextStatus &&
        nextStatus !== 'ready'
      ) {
        pushToast({
          message: getStageUnavailableMessage(nextStatus, t),
          tone: 'warning',
        })
      }

      lastStageAccessStatusRef.current = nextStatus
    })()

    return () => {
      controller.abort()
    }
  }, [loadStageAccessStatus, location.pathname, pushToast, t])

  useEffect(() => {
    const handleRefresh = () => {
      const controller = new AbortController()
      void (async () => {
        const nextStatus = await loadStageAccessStatus(controller.signal)

        if (controller.signal.aborted) {
          return
        }

        if (nextStatus === null) {
          return
        }

        setStageAccessStatus(nextStatus)

        if (
          lastStageAccessStatusRef.current !== nextStatus &&
          nextStatus !== 'ready'
        ) {
          pushToast({
            message: getStageUnavailableMessage(nextStatus, t),
            tone: 'warning',
          })
        }

        lastStageAccessStatusRef.current = nextStatus
      })()
    }

    window.addEventListener(STAGE_API_AVAILABILITY_REFRESH_EVENT, handleRefresh)

    return () => {
      window.removeEventListener(STAGE_API_AVAILABILITY_REFRESH_EVENT, handleRefresh)
    }
  }, [loadStageAccessStatus, pushToast, t])

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
            'grid grid-cols-[auto_minmax(0,1fr)_auto] items-center gap-2.5 border transition-all duration-300 ease-out',
            isAtTop
              ? 'rounded-b-[1.6rem] border-[var(--color-border-subtle)] border-t-0 bg-[color-mix(in_srgb,var(--color-bg-panel)_92%,transparent)] px-4 py-2.5 shadow-[0_12px_34px_rgba(0,0,0,0.16)] backdrop-blur sm:px-5'
              : 'rounded-[1.6rem] border-[var(--color-border-subtle)] bg-[var(--color-bg-panel-strong)] px-4 py-2 shadow-[0_18px_48px_rgba(0,0,0,0.26)] backdrop-blur sm:px-5',
          )}
        >
          <NavLink
            className="flex min-w-0 items-center md:justify-self-start"
            to={appPaths.workspace}
          >
            <p className="font-display text-[1.15rem] leading-none text-[var(--color-text-primary)]">
              SillyStage
            </p>
          </NavLink>

          <SegmentedSelector
            ariaLabel={t('common.navigation')}
            className="justify-self-center"
            items={[
              {
                ariaLabel: t('workspace.headbar.label'),
                icon: <FontAwesomeIcon fixedWidth icon={faTableCellsLarge} />,
                label: <span>{t('workspace.headbar.label')}</span>,
                value: appPaths.workspace,
              },
              {
                ariaLabel: t('stage.headbar.label'),
                disabled:
                  stageAccessStatus !== 'checking' &&
                  stageAccessStatus !== 'ready',
                icon: <FontAwesomeIcon fixedWidth icon={faMasksTheater} />,
                label: <span>{t('stage.headbar.label')}</span>,
                value: appPaths.stage,
              },
            ]}
            onDisabledValueClick={(value) => {
              if (value !== appPaths.stage) {
                return
              }

              if (
                stageAccessStatus === 'checking' ||
                stageAccessStatus === 'ready'
              ) {
                return
              }

              pushToast({
                message: getStageUnavailableMessage(stageAccessStatus, t),
                tone: 'warning',
              })
            }}
            onValueChange={(value) => {
              if (value !== currentTopLevelPath) {
                navigate(value)
              }
            }}
            value={currentTopLevelPath}
          />

          <HeadbarMenu className="justify-self-end" />
        </div>
      </div>
    </header>
  )
}
