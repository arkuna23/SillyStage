import { useTranslation } from 'react-i18next'
import { Link } from 'react-router-dom'

import { appPaths } from '../app/paths'
import { Button } from '../components/ui/button'

export function HomePage() {
  const { t } = useTranslation()

  return (
    <section className="flex min-h-[calc(100vh-8rem)] w-full flex-1 items-center justify-center py-10 sm:py-16">
      <div className="panel-enter mx-auto flex w-full max-w-4xl flex-col items-center text-center">
        <div className="max-w-3xl space-y-5">
          <h1 className="font-display text-5xl leading-[0.92] text-[var(--color-text-primary)] sm:text-6xl lg:text-7xl">
            {t('home.landing.title')}
          </h1>
          <p className="mx-auto max-w-2xl text-base leading-8 text-[var(--color-text-secondary)] sm:text-lg">
            {t('home.landing.subtitle')}
          </p>
        </div>

        <div className="mt-9 flex justify-center">
          <Button asChild size="lg">
            <Link to={appPaths.workspace}>{t('home.landing.start')}</Link>
          </Button>
        </div>
      </div>
    </section>
  )
}
