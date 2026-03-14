import { useTranslation } from 'react-i18next'

import { normalizeLocale } from '../i18n/config'
import { cn } from '../lib/cn'

const localeOptions = [
  { key: 'zh-CN', labelKey: 'common.locales.zh-CN' },
  { key: 'en', labelKey: 'common.locales.en' },
] as const

type LanguageToggleProps = {
  className?: string
  compact?: boolean
  showLabel?: boolean
}

export function LanguageToggle({
  className,
  compact = false,
  showLabel = true,
}: LanguageToggleProps = {}) {
  const { i18n, t } = useTranslation()
  const activeLocale = normalizeLocale(i18n.language ?? i18n.resolvedLanguage) ?? 'en'

  return (
    <div
      className={cn(
        'relative z-10 inline-flex items-center rounded-full border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel-strong)] backdrop-blur pointer-events-auto',
        compact ? 'px-1.5 py-1.5' : 'px-2 py-2',
        showLabel ? 'gap-3' : 'gap-2',
        className,
      )}
    >
      {showLabel ? (
        <span className="pl-2 text-[0.7rem] uppercase text-[var(--color-text-muted)]">
          {t('common.language')}
        </span>
      ) : null}

      <div
        aria-label={t('common.language')}
        className={cn(
          'inline-flex items-center gap-1 rounded-full bg-white/5',
          compact ? 'p-0.5' : 'p-1',
        )}
        role="group"
      >
        {localeOptions.map((option) => {
          const selected = activeLocale === option.key

          return (
            <button
              key={option.key}
              aria-pressed={selected}
              className={cn(
                'cursor-pointer rounded-full font-medium transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-focus-ring)]',
                compact ? 'px-2.5 py-1.5 text-[0.82rem]' : 'px-3 py-2 text-sm',
                selected
                  ? 'bg-[var(--color-accent-gold)] text-[color:var(--color-accent-ink)] shadow-[0_10px_30px_var(--color-accent-glow)]'
                  : 'text-[var(--color-text-secondary)] hover:bg-white/5 hover:text-[var(--color-text-primary)]',
              )}
              lang={option.key}
              onClick={() => {
                if (selected) {
                  return
                }

                void i18n.changeLanguage(option.key)
              }}
              type="button"
            >
              {t(option.labelKey)}
            </button>
          )
        })}
      </div>
    </div>
  )
}
