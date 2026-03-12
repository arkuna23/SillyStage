import { useTranslation } from 'react-i18next'

import { normalizeLocale } from '../i18n/config'
import { cn } from '../lib/cn'
import { themePreferences, type ThemePreference } from '../theme/config'
import { useTheme } from '../theme/use-theme'
import {
  PopupMenu,
  PopupMenuContent,
  PopupMenuLabel,
  PopupMenuRadioGroup,
  PopupMenuRadioItem,
  PopupMenuSeparator,
  PopupMenuTrigger,
} from './ui/popup-menu'

const localeOptions = [
  { key: 'zh-CN', labelKey: 'common.locales.zh-CN' },
  { key: 'en', labelKey: 'common.locales.en' },
] as const

const themeOptions = [
  { key: 'system', labelKey: 'common.themes.system' },
  { key: 'light', labelKey: 'common.themes.light' },
  { key: 'dark', labelKey: 'common.themes.dark' },
] as const satisfies ReadonlyArray<{
  key: ThemePreference
  labelKey: 'common.themes.system' | 'common.themes.light' | 'common.themes.dark'
}>

function HeadbarMenuIcon() {
  return (
    <svg
      aria-hidden="true"
      className="h-4 w-4"
      fill="none"
      viewBox="0 0 24 24"
      xmlns="http://www.w3.org/2000/svg"
    >
      <path
        d="M5 6.5H19M5 12H19M5 17.5H19"
        stroke="currentColor"
        strokeLinecap="round"
        strokeWidth="1.7"
      />
      <circle cx="9" cy="6.5" fill="currentColor" r="1.7" />
      <circle cx="15" cy="12" fill="currentColor" r="1.7" />
      <circle cx="11" cy="17.5" fill="currentColor" r="1.7" />
    </svg>
  )
}

export function HeadbarMenu({ className }: { className?: string }) {
  const { i18n, t } = useTranslation()
  const { setThemePreference, themePreference } = useTheme()
  const activeLocale = normalizeLocale(i18n.language ?? i18n.resolvedLanguage) ?? 'en'

  return (
    <PopupMenu>
      <PopupMenuTrigger asChild>
        <button
          aria-label={t('common.menu')}
          className={cn(
            'inline-flex h-10 w-10 items-center justify-center rounded-full border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel-strong)] text-[var(--color-text-secondary)] transition hover:text-[var(--color-text-primary)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-amber-200/70 data-[state=open]:bg-[var(--color-bg-elevated)] data-[state=open]:text-[var(--color-text-primary)]',
            className,
          )}
          type="button"
        >
          <HeadbarMenuIcon />
        </button>
      </PopupMenuTrigger>

      <PopupMenuContent align="end" className="w-64" sideOffset={14}>
        <PopupMenuLabel>{t('common.language')}</PopupMenuLabel>
        <PopupMenuRadioGroup
          onValueChange={(nextLocale) => {
            if (nextLocale === activeLocale) {
              return
            }

            void i18n.changeLanguage(nextLocale)
          }}
          value={activeLocale}
        >
          {localeOptions.map((option) => (
            <PopupMenuRadioItem key={option.key} value={option.key}>
              {t(option.labelKey)}
            </PopupMenuRadioItem>
          ))}
        </PopupMenuRadioGroup>

        <PopupMenuSeparator />

        <PopupMenuLabel>{t('common.theme')}</PopupMenuLabel>
        <PopupMenuRadioGroup
          onValueChange={(nextTheme) => {
            if (!themePreferences.includes(nextTheme as ThemePreference)) {
              return
            }

            setThemePreference(nextTheme as ThemePreference)
          }}
          value={themePreference}
        >
          {themeOptions.map((option) => (
            <PopupMenuRadioItem key={option.key} value={option.key}>
              {t(option.labelKey)}
            </PopupMenuRadioItem>
          ))}
        </PopupMenuRadioGroup>
      </PopupMenuContent>
    </PopupMenu>
  )
}
