import { useCallback, useEffect, useRef, useState } from 'react'
import type { ChangeEvent } from 'react'
import { useTranslation } from 'react-i18next'

import { Badge } from '../../components/ui/badge'
import { Button } from '../../components/ui/button'
import {
  Card,
  CardContent,
  CardDescription,
  CardFooter,
  CardHeader,
  CardTitle,
} from '../../components/ui/card'
import { SectionHeader } from '../../components/ui/section-header'
import { cn } from '../../lib/cn'
import { isRpcConflict } from '../../lib/rpc'
import {
  createCoverDataUrl,
  downloadCharacterArchive,
  getCharacterCover,
  hasCharacterCardExtension,
  importCharacterArchive,
  listCharacters,
} from './api'
import { CreateCharacterDialog } from './create-character-dialog'
import type { CharacterSummary } from './types'

type NoticeTone = 'error' | 'success' | 'warning'

type Notice = {
  message: string
  tone: NoticeTone
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function StatusNotice({
  notice,
}: {
  notice: Notice
}) {
  return (
    <div
      className={cn(
        'rounded-[1.4rem] border px-4 py-3 text-sm leading-7 shadow-[0_14px_38px_rgba(0,0,0,0.12)] backdrop-blur',
        notice.tone === 'success'
          ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] text-[var(--color-text-primary)]'
          : notice.tone === 'warning'
            ? 'border-[var(--color-accent-copper-soft)] bg-[color-mix(in_srgb,var(--color-accent-copper-soft)_55%,transparent)] text-[var(--color-text-primary)]'
            : 'border-[rgba(239,68,68,0.24)] bg-[rgba(127,29,29,0.24)] text-[var(--color-text-primary)]',
      )}
      role="status"
    >
      {notice.message}
    </div>
  )
}

function LoadingGrid() {
  return (
    <div className="grid gap-4 md:grid-cols-2 2xl:grid-cols-3">
      {Array.from({ length: 6 }).map((_, index) => (
        <div
          className={cn(
            'overflow-hidden rounded-[1.75rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] shadow-[0_24px_80px_rgba(0,0,0,0.18)]',
            index > 0 ? 'panel-enter panel-enter-delay-1' : 'panel-enter',
          )}
          key={index}
        >
          <div className="h-44 animate-pulse bg-[color-mix(in_srgb,var(--color-accent-gold-soft)_55%,var(--color-bg-elevated))]" />
          <div className="space-y-3 p-6">
            <div className="h-3 w-20 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="h-7 w-2/3 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="h-3 w-full animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="h-3 w-4/5 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
          </div>
        </div>
      ))}
    </div>
  )
}

function CharacterCard({
  coverUrl,
  exporting,
  onExport,
  summary,
}: {
  coverUrl?: string
  exporting: boolean
  onExport: () => void
  summary: CharacterSummary
}) {
  const { t } = useTranslation()

  return (
    <Card className="h-full overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_94%,transparent)]">
      <div className="relative aspect-[4/3] overflow-hidden border-b border-[var(--color-border-subtle)] bg-[linear-gradient(135deg,rgba(217,167,74,0.12),rgba(115,183,255,0.14))]">
        {coverUrl ? (
          <img
            alt={t('characters.card.coverAlt', { name: summary.name })}
            className="h-full w-full object-cover"
            src={coverUrl}
          />
        ) : (
          <div className="flex h-full w-full items-end justify-between p-4">
            <Badge variant="subtle">{summary.character_id}</Badge>
            <div className="text-right">
              <p className="text-xs uppercase text-[var(--color-text-muted)]">
                {t('characters.card.coverPending')}
              </p>
              <p className="mt-1 text-sm text-[var(--color-text-secondary)]">
                {t('characters.card.coverMissing')}
              </p>
            </div>
          </div>
        )}
      </div>

      <CardHeader className="gap-2 p-5 pb-3">
        <div className="space-y-1.5">
          <CardTitle className="text-[1.6rem]">{summary.name}</CardTitle>
          <div className="flex flex-wrap items-center gap-2">
            <p className="text-[0.68rem] uppercase text-[var(--color-text-muted)]">
              {t('characters.card.idLabel')}
            </p>
            <CardDescription className="font-mono text-[0.72rem] leading-5 uppercase text-[var(--color-text-muted)]">
              {summary.character_id}
            </CardDescription>
          </div>
        </div>
      </CardHeader>

      <CardContent className="space-y-3 px-5 pb-5 pt-0">
        <div className="space-y-1.5">
          <p className="text-[0.68rem] uppercase text-[var(--color-text-muted)]">
            {t('characters.card.personality')}
          </p>
          <p className="text-sm leading-6 text-[var(--color-text-secondary)]">
            {summary.personality}
          </p>
        </div>

        <div className="space-y-1.5">
          <p className="text-[0.68rem] uppercase text-[var(--color-text-muted)]">
            {t('characters.card.style')}
          </p>
          <p className="text-sm leading-6 text-[var(--color-text-secondary)]">
            {summary.style}
          </p>
        </div>

        <div className="space-y-1.5">
          <p className="text-[0.68rem] uppercase text-[var(--color-text-muted)]">
            {t('characters.card.tendencies')}
          </p>
          <div className="flex flex-wrap gap-1.5">
            {summary.tendencies.length > 0 ? (
              summary.tendencies.map((tendency) => (
                <Badge
                  className="normal-case px-3 py-1"
                  key={tendency}
                  variant="subtle"
                >
                  {tendency}
                </Badge>
              ))
            ) : (
              <Badge className="normal-case px-3 py-1" variant="subtle">
                {t('characters.card.noTendencies')}
              </Badge>
            )}
          </div>
        </div>
      </CardContent>

      <CardFooter className="border-t border-[var(--color-border-subtle)] px-5 pb-5 pt-3">
        <Button
          className="w-full"
          disabled={exporting}
          onClick={onExport}
          size="md"
          variant="secondary"
        >
          {exporting ? t('characters.actions.exporting') : t('characters.actions.export')}
        </Button>
      </CardFooter>
    </Card>
  )
}

export function CharacterManagementPage() {
  const { t } = useTranslation()
  const importInputRef = useRef<HTMLInputElement | null>(null)
  const coverCacheRef = useRef<Map<string, string>>(new Map())
  const [characters, setCharacters] = useState<CharacterSummary[]>([])
  const [coverUrls, setCoverUrls] = useState<Record<string, string>>({})
  const [exportingCharacterId, setExportingCharacterId] = useState<string | null>(null)
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false)
  const [isImporting, setIsImporting] = useState(false)
  const [isLoading, setIsLoading] = useState(true)
  const [notice, setNotice] = useState<Notice | null>(null)

  const coveredCharacters = characters.filter(
    (character) => character.cover_file_name && character.cover_mime_type,
  ).length

  const refreshCharacters = useCallback(
    async (signal?: AbortSignal) => {
      setIsLoading(true)

      try {
        const summaries = await listCharacters(signal)

        if (signal?.aborted) {
          return
        }

        setCharacters(summaries)

        const cachedCoverUrls: Record<string, string> = {}

        for (const summary of summaries) {
          const cachedCoverUrl = coverCacheRef.current.get(summary.character_id)

          if (cachedCoverUrl) {
            cachedCoverUrls[summary.character_id] = cachedCoverUrl
          }
        }

        setCoverUrls(cachedCoverUrls)

        const summariesNeedingCover = summaries.filter(
          (summary) =>
            summary.cover_file_name &&
            summary.cover_mime_type &&
            !coverCacheRef.current.has(summary.character_id),
        )

        if (summariesNeedingCover.length === 0) {
          return
        }

        const coverResults = await Promise.allSettled(
          summariesNeedingCover.map(async (summary) => {
            const cover = await getCharacterCover(summary.character_id, signal)

            return {
              characterId: summary.character_id,
              coverUrl: createCoverDataUrl({
                coverBase64: cover.cover_base64,
                coverMimeType: cover.cover_mime_type,
              }),
            }
          }),
        )

        if (signal?.aborted) {
          return
        }

        const nextCoverUrls: Record<string, string> = {}

        for (const result of coverResults) {
          if (result.status !== 'fulfilled') {
            continue
          }

          coverCacheRef.current.set(result.value.characterId, result.value.coverUrl)
          nextCoverUrls[result.value.characterId] = result.value.coverUrl
        }

        if (Object.keys(nextCoverUrls).length > 0) {
          setCoverUrls((currentCoverUrls) => ({
            ...currentCoverUrls,
            ...nextCoverUrls,
          }))
        }
      } catch (error) {
        if (signal?.aborted) {
          return
        }

        setNotice({
          message: getErrorMessage(error, t('characters.feedback.loadFailed')),
          tone: 'error',
        })
      } finally {
        if (!signal?.aborted) {
          setIsLoading(false)
        }
      }
    },
    [t],
  )

  useEffect(() => {
    const controller = new AbortController()

    void refreshCharacters(controller.signal)

    return () => {
      controller.abort()
    }
  }, [refreshCharacters])

  async function handleImportSelection(event: ChangeEvent<HTMLInputElement>) {
    const selectedFile = event.target.files?.[0]

    event.target.value = ''

    if (!selectedFile) {
      return
    }

    if (!hasCharacterCardExtension(selectedFile.name)) {
      setNotice({
        message: t('characters.feedback.invalidImportType'),
        tone: 'error',
      })
      return
    }

    setIsImporting(true)

    try {
      const importedCharacter = await importCharacterArchive(selectedFile)

      setNotice({
        message: t('characters.feedback.imported', { name: importedCharacter.name }),
        tone: 'success',
      })

      await refreshCharacters()
    } catch (error) {
      setNotice({
        message: getErrorMessage(error, t('characters.feedback.importFailed')),
        tone: 'error',
      })
    } finally {
      setIsImporting(false)
    }
  }

  async function handleExport(summary: CharacterSummary) {
    setExportingCharacterId(summary.character_id)

    try {
      await downloadCharacterArchive(summary.character_id)

      setNotice({
        message: t('characters.feedback.exported', { name: summary.name }),
        tone: 'success',
      })
    } catch (error) {
      setNotice({
        message: isRpcConflict(error)
          ? t('characters.feedback.exportNeedsCover', { name: summary.name })
          : getErrorMessage(error, t('characters.feedback.exportFailed')),
        tone: isRpcConflict(error) ? 'warning' : 'error',
      })
    } finally {
      setExportingCharacterId(null)
    }
  }

  return (
    <div className="flex flex-col gap-6">
      <CreateCharacterDialog
        onCompleted={async (result) => {
          setNotice({
            message: result.message,
            tone: result.tone,
          })

          await refreshCharacters()
        }}
        onOpenChange={setIsCreateDialogOpen}
        open={isCreateDialogOpen}
      />

      <input
        accept=".chr,application/octet-stream"
        className="sr-only"
        onChange={(event) => {
          void handleImportSelection(event)
        }}
        ref={importInputRef}
        type="file"
      />

      <Card className="panel-enter overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_94%,transparent)]">
        <CardHeader className="gap-6 border-b border-[var(--color-border-subtle)]">
          <SectionHeader
            actions={
              <div className="flex flex-wrap items-center gap-3">
                <Button
                  onClick={() => {
                    setIsCreateDialogOpen(true)
                  }}
                  size="md"
                >
                  {t('characters.actions.create')}
                </Button>

                <Button
                  disabled={isImporting}
                  onClick={() => {
                    importInputRef.current?.click()
                  }}
                  size="md"
                  variant="secondary"
                >
                  {isImporting
                    ? t('characters.actions.importing')
                    : t('characters.actions.import')}
                </Button>
              </div>
            }
            title={t('characters.title')}
          />

          <div className="grid gap-3 sm:grid-cols-2 xl:max-w-[28rem]">
            <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
              <p className="text-xs uppercase text-[var(--color-text-muted)]">
                {t('characters.metrics.total')}
              </p>
              <p className="mt-3 font-display text-4xl text-[var(--color-text-primary)]">
                {characters.length}
              </p>
            </div>

            <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
              <p className="text-xs uppercase text-[var(--color-text-muted)]">
                {t('characters.metrics.covered')}
              </p>
              <p className="mt-3 font-display text-4xl text-[var(--color-text-primary)]">
                {coveredCharacters}
              </p>
            </div>
          </div>
        </CardHeader>

        <CardContent className="pt-6">
          {notice ? <StatusNotice notice={notice} /> : null}

          <div className={notice ? 'mt-5' : undefined}>
            {isLoading ? (
              <LoadingGrid />
            ) : characters.length === 0 ? (
              <div className="rounded-[1.6rem] border border-dashed border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-6 py-12 text-center">
                <h3 className="font-display text-3xl text-[var(--color-text-primary)]">
                  {t('characters.empty.title')}
                </h3>

                <div className="mt-7 flex flex-wrap justify-center gap-3">
                  <Button
                    onClick={() => {
                      setIsCreateDialogOpen(true)
                    }}
                    size="md"
                  >
                    {t('characters.actions.create')}
                  </Button>
                  <Button
                    disabled={isImporting}
                    onClick={() => {
                      importInputRef.current?.click()
                    }}
                    size="md"
                    variant="secondary"
                  >
                    {t('characters.actions.import')}
                  </Button>
                </div>
              </div>
            ) : (
              <div>
                <div className="grid gap-4 md:grid-cols-2 2xl:grid-cols-3">
                  {characters.map((summary, index) => (
                    <div
                      className={cn(
                        'panel-enter',
                        index % 3 === 1
                          ? 'panel-enter-delay-1'
                          : index % 3 === 2
                            ? 'panel-enter-delay-2'
                            : undefined,
                      )}
                      key={summary.character_id}
                    >
                      <CharacterCard
                        coverUrl={coverUrls[summary.character_id]}
                        exporting={exportingCharacterId === summary.character_id}
                        onExport={() => {
                          void handleExport(summary)
                        }}
                        summary={summary}
                      />
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        </CardContent>
      </Card>
    </div>
  )
}
