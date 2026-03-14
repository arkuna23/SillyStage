import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'

import { Badge } from '../../components/ui/badge'
import { Button } from '../../components/ui/button'
import {
  Dialog,
  DialogBody,
  DialogClose,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../../components/ui/dialog'
import type { CharacterSummary } from './types'

type CharacterDetailsDialogProps = {
  coverUrl?: string
  deleting?: boolean
  onDelete: () => void
  onEdit: () => void
  exporting: boolean
  onExport: () => void
  onOpenChange: (open: boolean) => void
  open: boolean
  summary: CharacterSummary | null
}

const COVER_OBJECT_POSITION = 'center 26%'

function getCharacterMonogram(name: string) {
  return Array.from(name.trim())[0] ?? '?'
}

function DetailSection({
  children,
  title,
}: {
  children: ReactNode
  title: string
}) {
  return (
    <div className="space-y-2.5">
      <p className="text-[0.72rem] uppercase text-[var(--color-text-muted)]">{title}</p>
      {children}
    </div>
  )
}

export function CharacterDetailsDialog({
  coverUrl,
  deleting = false,
  onDelete,
  onEdit,
  exporting,
  onExport,
  onOpenChange,
  open,
  summary,
}: CharacterDetailsDialogProps) {
  const { t } = useTranslation()
  const monogram = summary ? getCharacterMonogram(summary.name) : '?'

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent aria-describedby={undefined} className="w-[min(96vw,64rem)] overflow-hidden">
        {summary ? (
          <>
            <DialogHeader className="border-b border-[var(--color-border-subtle)]">
              <div className="flex flex-wrap items-center gap-2">
                <p className="text-[0.72rem] uppercase text-[var(--color-text-muted)]">
                  {t('characters.card.idLabel')}
                </p>
                <Badge className="normal-case px-3 py-1" variant="subtle">
                  {summary.character_id}
                </Badge>
              </div>
              <DialogTitle className="text-[2.15rem]">{summary.name}</DialogTitle>
            </DialogHeader>

            <DialogBody className="max-h-[calc(92vh-13rem)] overflow-y-auto pt-6">
              <div className="grid gap-6 lg:grid-cols-[minmax(0,0.95fr)_minmax(0,1.05fr)]">
                <div className="overflow-hidden rounded-[1.7rem] border border-[var(--color-border-subtle)] bg-[linear-gradient(135deg,var(--color-accent-gold-soft),var(--color-accent-copper-soft))]">
                  {coverUrl ? (
                    <img
                      alt={t('characters.card.coverAlt', { name: summary.name })}
                      className="aspect-[4/3] h-full w-full object-cover"
                      src={coverUrl}
                      style={{ objectPosition: COVER_OBJECT_POSITION }}
                    />
                  ) : (
                    <div className="flex aspect-[4/3] h-full w-full items-center justify-center">
                      <span className="inline-flex size-24 items-center justify-center rounded-full border border-[rgba(255,255,255,0.12)] bg-[rgba(18,10,31,0.34)] font-display text-4xl text-[var(--color-text-primary)] shadow-[inset_0_1px_0_rgba(255,255,255,0.06)]">
                        {monogram}
                      </span>
                    </div>
                  )}
                </div>

                <div className="space-y-5">
                  <DetailSection title={t('characters.card.personality')}>
                    <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
                      {summary.personality}
                    </p>
                  </DetailSection>

                  <DetailSection title={t('characters.card.style')}>
                    <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
                      {summary.style}
                    </p>
                  </DetailSection>

                  <DetailSection title={t('characters.card.tendencies')}>
                    <div className="flex flex-wrap gap-2">
                      {summary.tendencies.length > 0 ? (
                        summary.tendencies.map((tendency) => (
                          <Badge className="normal-case px-3 py-1" key={tendency} variant="subtle">
                            {tendency}
                          </Badge>
                        ))
                      ) : (
                        <Badge className="normal-case px-3 py-1" variant="subtle">
                          {t('characters.card.noTendencies')}
                        </Badge>
                      )}
                    </div>
                  </DetailSection>
                </div>
              </div>
            </DialogBody>

            <DialogFooter className="sm:items-center">
              <DialogClose asChild>
                <Button size="md" variant="ghost">
                  {t('characters.actions.closeDetails')}
                </Button>
              </DialogClose>

              <div className="flex flex-wrap items-center justify-end gap-3 sm:ml-auto">
                <Button onClick={onEdit} size="md" variant="secondary">
                  {t('characters.actions.edit')}
                </Button>

                <Button disabled={exporting} onClick={onExport} size="md">
                  {exporting ? t('characters.actions.exporting') : t('characters.actions.export')}
                </Button>

                <Button
                  className="border-[var(--color-state-error-line)] bg-[var(--color-state-error)] text-[var(--color-accent-ink)] hover:bg-[color-mix(in_srgb,var(--color-state-error)_90%,black)]"
                  disabled={deleting}
                  onClick={onDelete}
                  size="md"
                >
                  {deleting ? t('characters.actions.deleting') : t('characters.actions.delete')}
                </Button>
              </div>
            </DialogFooter>
          </>
        ) : null}
      </DialogContent>
    </Dialog>
  )
}
