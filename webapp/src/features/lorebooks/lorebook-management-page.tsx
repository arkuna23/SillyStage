import { faPen } from '@fortawesome/free-solid-svg-icons/faPen'
import { faPlus } from '@fortawesome/free-solid-svg-icons/faPlus'
import { faTrashCan } from '@fortawesome/free-solid-svg-icons/faTrashCan'
import { faWandMagicSparkles } from '@fortawesome/free-solid-svg-icons/faWandMagicSparkles'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { useCallback, useEffect, useLayoutEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { WorkspacePanelShell } from '../../components/layout/workspace-panel-shell'
import { useWorkspaceLayoutContext } from '../../components/layout/workspace-context'
import { Badge } from '../../components/ui/badge'
import { Button } from '../../components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '../../components/ui/card'
import { IconButton } from '../../components/ui/icon-button'
import { SectionHeader } from '../../components/ui/section-header'
import { useToastNotice } from '../../components/ui/toast-context'
import { isRpcConflict } from '../../lib/rpc'
import { demoLorebook } from '../demo-content/lorebook-sample-data'
import { InsertSampleDialog } from '../demo-content/insert-sample-dialog'
import { createLorebook, deleteLorebook, listLorebooks } from './api'
import { DeleteLorebookDialog } from './delete-lorebook-dialog'
import { LorebookFormDialog } from './lorebook-form-dialog'
import type { Lorebook } from './types'

type NoticeTone = 'error' | 'success' | 'warning'

type Notice = {
  message: string
  tone: NoticeTone
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function LorebooksListSkeleton() {
  return (
    <div className="divide-y divide-[var(--color-border-subtle)]">
      {Array.from({ length: 4 }).map((_, index) => (
        <div
          className="grid gap-4 py-4 lg:grid-cols-[minmax(0,1.15fr)_minmax(0,0.95fr)_auto] lg:items-center"
          key={index}
        >
          <div className="space-y-2.5">
            <div className="h-5 w-40 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="h-3 w-28 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="h-3 w-full animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
          </div>
          <div className="flex flex-wrap gap-2">
            <div className="h-8 w-24 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="h-8 w-24 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
          </div>
          <div className="flex justify-start gap-2 lg:justify-end">
            <div className="h-9 w-9 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="h-9 w-9 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
          </div>
        </div>
      ))}
    </div>
  )
}

function countEnabledEntries(lorebook: Lorebook) {
  return lorebook.entries.filter((entry) => entry.enabled).length
}

function countAlwaysIncludeEntries(lorebook: Lorebook) {
  return lorebook.entries.filter((entry) => entry.always_include).length
}

function summarizeLorebook(lorebook: Lorebook, emptyLabel: string) {
  const titledEntries = lorebook.entries
    .filter((entry) => entry.enabled)
    .map((entry) => entry.title.trim())
    .filter((title) => title.length > 0)

  if (titledEntries.length > 0) {
    return titledEntries.slice(0, 2).join(' · ')
  }

  const fallbackContent = lorebook.entries
    .map((entry) => entry.content.replace(/\s+/g, ' ').trim())
    .find((content) => content.length > 0)

  return fallbackContent ?? emptyLabel
}

export function LorebookManagementPage() {
  const { t } = useTranslation()
  const { setRailContent } = useWorkspaceLayoutContext()
  const [lorebooks, setLorebooks] = useState<Lorebook[]>([])
  const [notice, setNotice] = useState<Notice | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [isDeleting, setIsDeleting] = useState(false)
  const [isCreatingSample, setIsCreatingSample] = useState(false)
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false)
  const [isSampleDialogOpen, setIsSampleDialogOpen] = useState(false)
  const [editLorebookId, setEditLorebookId] = useState<string | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<Lorebook | null>(null)
  useToastNotice(notice)

  const existingLorebookIds = useMemo(
    () => lorebooks.map((lorebook) => lorebook.lorebook_id),
    [lorebooks],
  )
  const totalEntryCount = useMemo(
    () => lorebooks.reduce((total, lorebook) => total + lorebook.entries.length, 0),
    [lorebooks],
  )
  const totalAlwaysIncludeCount = useMemo(
    () =>
      lorebooks.reduce(
        (total, lorebook) => total + countAlwaysIncludeEntries(lorebook),
        0,
      ),
    [lorebooks],
  )
  const sampleLorebookExists = useMemo(
    () => existingLorebookIds.includes(demoLorebook.lorebookId),
    [existingLorebookIds],
  )

  const refreshLorebooks = useCallback(
    async (signal?: AbortSignal) => {
      setIsLoading(true)

      try {
        const nextLorebooks = await listLorebooks(signal)

        if (!signal?.aborted) {
          setLorebooks(nextLorebooks)
        }
      } catch (error) {
        if (!signal?.aborted) {
          setNotice({
            message: getErrorMessage(error, t('lorebooks.feedback.loadFailed')),
            tone: 'error',
          })
        }
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
    void refreshLorebooks(controller.signal)

    return () => {
      controller.abort()
    }
  }, [refreshLorebooks])

  useLayoutEffect(() => {
    setRailContent({
      description: t('lorebooks.rail.description'),
      stats: [
        { label: t('lorebooks.metrics.total'), value: lorebooks.length },
        { label: t('lorebooks.metrics.entries'), value: totalEntryCount },
        {
          label: t('lorebooks.metrics.alwaysInclude'),
          value: totalAlwaysIncludeCount,
        },
      ],
      title: t('lorebooks.title'),
    })

    return () => {
      setRailContent(null)
    }
  }, [
    lorebooks.length,
    setRailContent,
    t,
    totalAlwaysIncludeCount,
    totalEntryCount,
  ])

  async function handleDeleteLorebook() {
    if (!deleteTarget) {
      return
    }

    const target = deleteTarget
    setIsDeleting(true)

    try {
      await deleteLorebook(target.lorebook_id)
      setNotice({
        message: t('lorebooks.feedback.deleted', { name: target.display_name }),
        tone: 'success',
      })
      setDeleteTarget(null)
      await refreshLorebooks()
    } catch (error) {
      setDeleteTarget(null)
      setNotice({
        message: isRpcConflict(error)
          ? t('lorebooks.deleteDialog.conflict')
          : getErrorMessage(error, t('lorebooks.feedback.deleteFailed')),
        tone: isRpcConflict(error) ? 'warning' : 'error',
      })
    } finally {
      setIsDeleting(false)
    }
  }

  async function handleCreateSampleLorebook() {
    if (sampleLorebookExists) {
      setNotice({
        message: t('lorebooks.feedback.sampleExists'),
        tone: 'warning',
      })
      return
    }

    setIsCreatingSample(true)

    try {
      await createLorebook({
        display_name: demoLorebook.displayName,
        entries: demoLorebook.entries,
        lorebook_id: demoLorebook.lorebookId,
      })
      setNotice({
        message: t('lorebooks.feedback.sampleCreated', {
          name: demoLorebook.displayName,
        }),
        tone: 'success',
      })
      await refreshLorebooks()
    } catch (error) {
      setNotice({
        message: isRpcConflict(error)
          ? t('lorebooks.feedback.sampleExists')
          : getErrorMessage(error, t('lorebooks.feedback.sampleCreateFailed')),
        tone: isRpcConflict(error) ? 'warning' : 'error',
      })
    } finally {
      setIsCreatingSample(false)
    }
  }

  return (
    <div className="flex h-full min-h-0 flex-col gap-6">
      <LorebookFormDialog
        existingLorebookIds={existingLorebookIds}
        mode="create"
        onCompleted={async ({ message }) => {
          setNotice({ message, tone: 'success' })
          await refreshLorebooks()
        }}
        onOpenChange={setIsCreateDialogOpen}
        open={isCreateDialogOpen}
      />

      <LorebookFormDialog
        existingLorebookIds={existingLorebookIds}
        lorebookId={editLorebookId}
        mode="edit"
        onCompleted={async ({ message }) => {
          setNotice({ message, tone: 'success' })
          await refreshLorebooks()
        }}
        onOpenChange={(open) => {
          if (!open) {
            setEditLorebookId(null)
          }
        }}
        open={editLorebookId !== null}
      />

      <InsertSampleDialog
        cancelLabel={t('lorebooks.actions.cancel')}
        confirmLabel={t('lorebooks.sampleDialog.confirm')}
        confirmDisabled={sampleLorebookExists}
        description={t('lorebooks.sampleDialog.description')}
        existingLabel={t('lorebooks.sampleDialog.existing')}
        items={[
          {
            description: demoLorebook.lorebookId,
            label: demoLorebook.displayName,
            status: sampleLorebookExists ? 'existing' : 'new',
          },
        ]}
        newLabel={t('lorebooks.sampleDialog.new')}
        onConfirm={() => {
          void handleCreateSampleLorebook()
          setIsSampleDialogOpen(false)
        }}
        onOpenChange={setIsSampleDialogOpen}
        open={isSampleDialogOpen}
        pending={isCreatingSample}
        pendingLabel={t('lorebooks.actions.creatingSample')}
        title={t('lorebooks.sampleDialog.title')}
      />

      <DeleteLorebookDialog
        deleting={isDeleting}
        lorebook={deleteTarget}
        onConfirm={() => {
          void handleDeleteLorebook()
        }}
        onOpenChange={(open) => {
          if (!open) {
            setDeleteTarget(null)
          }
        }}
      />

      <WorkspacePanelShell className="flex min-h-0 flex-1">
        <Card className="flex min-h-0 flex-1 flex-col overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_94%,transparent)] shadow-none">
          <CardHeader className="gap-4 border-b border-[var(--color-border-subtle)] px-6 py-5 md:min-h-[5.75rem] md:px-7 md:py-5">
            <SectionHeader
              actions={
                <div className="flex min-h-10 items-center justify-end">
                  <div className="flex items-center gap-2.5">
                    <IconButton
                      disabled={isCreatingSample}
                      icon={<FontAwesomeIcon icon={faWandMagicSparkles} />}
                      label={
                        isCreatingSample
                          ? t('lorebooks.actions.creatingSample')
                          : t('lorebooks.actions.createSample')
                      }
                      onClick={() => {
                        setIsSampleDialogOpen(true)
                      }}
                      size="md"
                      variant="secondary"
                    />
                    <IconButton
                      icon={<FontAwesomeIcon icon={faPlus} />}
                      label={t('lorebooks.actions.create')}
                      onClick={() => {
                        setIsCreateDialogOpen(true)
                      }}
                      size="md"
                    />
                  </div>
                </div>
              }
              title={t('lorebooks.title')}
            />
          </CardHeader>

          <CardContent className="min-h-0 flex-1 overflow-y-auto pt-6">
            <div className="space-y-6 pr-1">
              {isLoading ? (
                <LorebooksListSkeleton />
              ) : lorebooks.length === 0 ? (
                <div className="py-12 text-center">
                  <h3 className="font-display text-3xl text-[var(--color-text-primary)]">
                    {t('lorebooks.empty.title')}
                  </h3>

                  <p className="mt-3 text-sm leading-7 text-[var(--color-text-secondary)]">
                    {t('lorebooks.empty.description')}
                  </p>

                  <div className="mt-7 flex justify-center gap-3">
                    <Button
                      disabled={isCreatingSample}
                      onClick={() => {
                        setIsSampleDialogOpen(true)
                      }}
                      size="md"
                      variant="secondary"
                    >
                      <FontAwesomeIcon className="text-sm" icon={faWandMagicSparkles} />
                      {isCreatingSample
                        ? t('lorebooks.actions.creatingSample')
                        : t('lorebooks.actions.createSample')}
                    </Button>
                    <Button
                      onClick={() => {
                        setIsCreateDialogOpen(true)
                      }}
                      size="md"
                    >
                      {t('lorebooks.actions.create')}
                    </Button>
                  </div>
                </div>
              ) : (
                <div className="space-y-5">
                  <CardTitle className="text-[1.85rem]">
                    {t('lorebooks.list.title')}
                  </CardTitle>

                  <div className="divide-y divide-[var(--color-border-subtle)]">
                    {lorebooks.map((lorebook) => (
                      <div
                        className="grid gap-4 py-4 lg:grid-cols-[minmax(0,1.15fr)_minmax(0,0.95fr)_auto] lg:items-center"
                        key={lorebook.lorebook_id}
                      >
                        <div className="min-w-0 space-y-2">
                          <h3 className="truncate font-display text-[1.2rem] leading-tight text-[var(--color-text-primary)]">
                            {lorebook.display_name}
                          </h3>
                          <p className="text-xs uppercase tracking-[0.08em] text-[var(--color-text-muted)]">
                            {lorebook.lorebook_id}
                          </p>
                          <p className="line-clamp-2 text-sm leading-7 text-[var(--color-text-secondary)]">
                            {summarizeLorebook(lorebook, t('lorebooks.list.emptySummary'))}
                          </p>
                        </div>

                        <div className="flex flex-wrap gap-2">
                          <Badge className="normal-case px-3 py-1.5" variant="subtle">
                            {t('lorebooks.list.entriesCount', {
                              count: lorebook.entries.length,
                            })}
                          </Badge>
                          <Badge className="normal-case px-3 py-1.5" variant="subtle">
                            {t('lorebooks.list.enabledCount', {
                              count: countEnabledEntries(lorebook),
                            })}
                          </Badge>
                          <Badge className="normal-case px-3 py-1.5" variant="info">
                            {t('lorebooks.list.alwaysIncludeCount', {
                              count: countAlwaysIncludeEntries(lorebook),
                            })}
                          </Badge>
                        </div>

                        <div className="flex justify-start gap-2 lg:justify-end">
                          <IconButton
                            icon={<FontAwesomeIcon icon={faPen} />}
                            label={t('lorebooks.actions.edit')}
                            onClick={() => {
                              setEditLorebookId(lorebook.lorebook_id)
                            }}
                            size="sm"
                            variant="secondary"
                          />
                          <IconButton
                            icon={<FontAwesomeIcon icon={faTrashCan} />}
                            label={t('lorebooks.actions.delete')}
                            onClick={() => {
                              setDeleteTarget(lorebook)
                            }}
                            size="sm"
                            variant="danger"
                          />
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </CardContent>
        </Card>
      </WorkspacePanelShell>
    </div>
  )
}
