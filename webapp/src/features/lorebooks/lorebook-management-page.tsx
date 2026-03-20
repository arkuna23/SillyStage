import { faPen } from '@fortawesome/free-solid-svg-icons/faPen'
import { faPlus } from '@fortawesome/free-solid-svg-icons/faPlus'
import { faCheckDouble } from '@fortawesome/free-solid-svg-icons/faCheckDouble'
import { faDownload } from '@fortawesome/free-solid-svg-icons/faDownload'
import { faSquareCheck } from '@fortawesome/free-solid-svg-icons/faSquareCheck'
import { faTrashCan } from '@fortawesome/free-solid-svg-icons/faTrashCan'
import { faUpload } from '@fortawesome/free-solid-svg-icons/faUpload'
import { faWandMagicSparkles } from '@fortawesome/free-solid-svg-icons/faWandMagicSparkles'
import { faXmark } from '@fortawesome/free-solid-svg-icons/faXmark'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { WorkspacePanelShell } from '../../components/layout/workspace-panel-shell'
import { useWorkspaceLayoutContext } from '../../components/layout/workspace-context'
import { Badge } from '../../components/ui/badge'
import { Button } from '../../components/ui/button'
import { Card, CardContent, CardHeader, CardTitle } from '../../components/ui/card'
import { IconButton } from '../../components/ui/icon-button'
import { SelectionToggleButton } from '../../components/ui/selection-toggle-button'
import { SectionHeader } from '../../components/ui/section-header'
import { useToastNotice } from '../../components/ui/toast-context'
import { runBatchDelete } from '../../lib/batch-delete'
import { createJsonExportFileName, downloadJsonFile, readJsonFile } from '../../lib/json-transfer'
import { isRpcConflict } from '../../lib/rpc'
import { buildDemoLorebook } from '../demo-content/lorebook-sample-data'
import { InsertSampleDialog } from '../demo-content/insert-sample-dialog'
import { createLorebook, deleteLorebook, listLorebooks } from './api'
import { DeleteLorebookDialog } from './delete-lorebook-dialog'
import { LorebookFormDialog } from './lorebook-form-dialog'
import { createLorebookBundle, isLorebookBundle } from './lorebook-transfer'
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
  const { i18n, t } = useTranslation()
  const { setRailContent } = useWorkspaceLayoutContext()
  const sampleLanguage = i18n.resolvedLanguage ?? i18n.language ?? 'en'
  const importInputRef = useRef<HTMLInputElement | null>(null)
  const [lorebooks, setLorebooks] = useState<Lorebook[]>([])
  const [notice, setNotice] = useState<Notice | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [isDeleting, setIsDeleting] = useState(false)
  const [isCreatingSample, setIsCreatingSample] = useState(false)
  const [isImporting, setIsImporting] = useState(false)
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false)
  const [isSampleDialogOpen, setIsSampleDialogOpen] = useState(false)
  const [editLorebookId, setEditLorebookId] = useState<string | null>(null)
  const [selectionMode, setSelectionMode] = useState(false)
  const [selectedLorebookIds, setSelectedLorebookIds] = useState<string[]>([])
  const [deleteTargetIds, setDeleteTargetIds] = useState<string[]>([])
  useToastNotice(notice)

  const existingLorebookIds = useMemo(
    () => lorebooks.map((lorebook) => lorebook.lorebook_id),
    [lorebooks],
  )
  const demoLorebook = useMemo(
    () => buildDemoLorebook(sampleLanguage),
    [sampleLanguage],
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
    [demoLorebook.lorebookId, existingLorebookIds],
  )
  const deleteTargets = useMemo(
    () =>
      deleteTargetIds
        .map((lorebookId) => lorebooks.find((lorebook) => lorebook.lorebook_id === lorebookId))
        .filter((lorebook): lorebook is Lorebook => lorebook !== undefined),
    [deleteTargetIds, lorebooks],
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

  useEffect(() => {
    const availableLorebookIds = new Set(lorebooks.map((lorebook) => lorebook.lorebook_id))

    setSelectedLorebookIds((currentSelection) =>
      currentSelection.filter((lorebookId) => availableLorebookIds.has(lorebookId)),
    )
    setDeleteTargetIds((currentSelection) =>
      currentSelection.filter((lorebookId) => availableLorebookIds.has(lorebookId)),
    )

    if (editLorebookId !== null && !availableLorebookIds.has(editLorebookId)) {
      setEditLorebookId(null)
    }
  }, [editLorebookId, lorebooks])

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
    if (deleteTargets.length === 0) {
      return
    }

    setIsDeleting(true)

    try {
      const result = await runBatchDelete(deleteTargets, async (target) => {
        await deleteLorebook(target.lorebook_id)
      })

      setDeleteTargetIds([])

      if (result.deleted.length > 0) {
        const deletedIds = new Set(result.deleted.map((target) => target.lorebook_id))
        setSelectedLorebookIds((currentSelection) =>
          currentSelection.filter((lorebookId) => !deletedIds.has(lorebookId)),
        )
        setDeleteTargetIds([])

        if (editLorebookId !== null && deletedIds.has(editLorebookId)) {
          setEditLorebookId(null)
        }
      }

      if (result.failed.length === 0) {
        setNotice({
          message:
            result.deleted.length > 1
              ? t('lorebooks.feedback.deletedMany', { count: result.deleted.length })
              : t('lorebooks.feedback.deleted', { name: result.deleted[0]?.display_name ?? '' }),
          tone: 'success',
        })
        if (selectionMode) {
          setSelectionMode(false)
          setSelectedLorebookIds([])
        }
      } else if (result.deleted.length > 0) {
        setNotice({
          message: t('lorebooks.feedback.deletedPartial', {
            failed: result.failed.length,
            success: result.deleted.length,
          }),
          tone: 'warning',
        })
      } else {
        setNotice({
          message:
            result.conflictCount > 0
              ? deleteTargets.length > 1
                ? t('lorebooks.deleteDialog.conflictMany')
                : t('lorebooks.deleteDialog.conflict')
              : t('lorebooks.feedback.deleteFailed'),
          tone: result.conflictCount > 0 ? 'warning' : 'error',
        })
      }

      await refreshLorebooks()
    } finally {
      setIsDeleting(false)
    }
  }

  function toggleLorebookSelection(lorebookId: string) {
    setSelectedLorebookIds((currentSelection) =>
      currentSelection.includes(lorebookId)
        ? currentSelection.filter((currentLorebookId) => currentLorebookId !== lorebookId)
        : [...currentSelection, lorebookId],
    )
  }

  async function handleExportSelection() {
    const exportTargets = lorebooks.filter((lorebook) =>
      selectedLorebookIds.includes(lorebook.lorebook_id),
    )

    if (exportTargets.length === 0) {
      return
    }

    try {
      downloadJsonFile(
        createJsonExportFileName('sillystage-lorebooks'),
        createLorebookBundle(exportTargets),
      )
      setNotice({
        message: t('lorebooks.feedback.exported', { count: exportTargets.length }),
        tone: 'success',
      })
    } catch (error) {
      setNotice({
        message: getErrorMessage(error, t('lorebooks.feedback.exportFailed')),
        tone: 'error',
      })
    }
  }

  async function handleImportSelection(file: File) {
    setIsImporting(true)

    try {
      const payload = await readJsonFile(file)

      if (!isLorebookBundle(payload)) {
        setNotice({
          message: t('lorebooks.feedback.importInvalid'),
          tone: 'error',
        })
        return
      }

      const existingIds = new Set(lorebooks.map((lorebook) => lorebook.lorebook_id))
      const createdNames: string[] = []
      const skippedNames: string[] = []
      const failedNames: string[] = []

      for (const lorebook of payload.lorebooks) {
        if (existingIds.has(lorebook.lorebook_id)) {
          skippedNames.push(lorebook.display_name)
          continue
        }

        try {
          await createLorebook({
            display_name: lorebook.display_name,
            entries: lorebook.entries,
            lorebook_id: lorebook.lorebook_id,
          })
          createdNames.push(lorebook.display_name)
          existingIds.add(lorebook.lorebook_id)
        } catch {
          failedNames.push(lorebook.display_name)
        }
      }

      if (createdNames.length > 0) {
        await refreshLorebooks()
      }

      if (createdNames.length > 0 && skippedNames.length === 0 && failedNames.length === 0) {
        setNotice({
          message: t('lorebooks.feedback.imported', { count: createdNames.length }),
          tone: 'success',
        })
      } else if (createdNames.length > 0 || skippedNames.length > 0) {
        setNotice({
          message: t('lorebooks.feedback.importedPartial', {
            failed: failedNames.length,
            skipped: skippedNames.length,
            success: createdNames.length,
          }),
          tone: failedNames.length > 0 ? 'warning' : 'success',
        })
      } else {
        setNotice({
          message: t('lorebooks.feedback.importSkipped', { count: skippedNames.length }),
          tone: 'warning',
        })
      }
    } catch (error) {
      setNotice({
        message: getErrorMessage(error, t('lorebooks.feedback.importFailed')),
        tone: 'error',
      })
    } finally {
      setIsImporting(false)
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
        onConfirm={() => {
          void handleDeleteLorebook()
        }}
        onOpenChange={(open) => {
          if (!open) {
            setDeleteTargetIds([])
          }
        }}
        targets={deleteTargets}
      />

      <input
        accept="application/json,.json"
        className="sr-only"
        name="lorebook_import"
        onChange={(event) => {
          const selectedFile = event.target.files?.[0]

          event.target.value = ''

          if (selectedFile) {
            void handleImportSelection(selectedFile)
          }
        }}
        ref={importInputRef}
        type="file"
      />

      <WorkspacePanelShell className="flex min-h-0 flex-1">
        <Card className="flex min-h-0 flex-1 flex-col overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_94%,transparent)] shadow-none">
          <CardHeader className="gap-4 border-b border-[var(--color-border-subtle)] px-6 py-5 md:min-h-[5.75rem] md:px-7 md:py-5">
            <SectionHeader
              actions={
                <div className="flex min-h-10 flex-wrap items-center justify-end gap-2">
                  {selectionMode ? (
                    <>
                      <Badge className="normal-case px-3.5 py-2" variant="subtle">
                        {t('lorebooks.selection.count', { count: selectedLorebookIds.length })}
                      </Badge>
                      <IconButton
                        disabled={lorebooks.length === 0}
                        icon={<FontAwesomeIcon icon={faCheckDouble} />}
                        label={t('lorebooks.actions.selectAll')}
                        onClick={() => {
                          setSelectedLorebookIds(lorebooks.map((lorebook) => lorebook.lorebook_id))
                        }}
                        size="md"
                        variant="secondary"
                      />
                      <IconButton
                        disabled={selectedLorebookIds.length === 0}
                        icon={<FontAwesomeIcon icon={faDownload} />}
                        label={t('lorebooks.actions.export')}
                        onClick={() => {
                          void handleExportSelection()
                        }}
                        size="md"
                        variant="secondary"
                      />
                      <IconButton
                        disabled={selectedLorebookIds.length === 0}
                        icon={<FontAwesomeIcon icon={faTrashCan} />}
                        label={t('lorebooks.actions.deleteSelected')}
                        onClick={() => {
                          setDeleteTargetIds(selectedLorebookIds)
                        }}
                        size="md"
                        variant="danger"
                      />
                      <IconButton
                        icon={<FontAwesomeIcon icon={faXmark} />}
                        label={t('lorebooks.actions.cancelSelection')}
                        onClick={() => {
                          setSelectionMode(false)
                          setSelectedLorebookIds([])
                        }}
                        size="md"
                        variant="secondary"
                      />
                    </>
                  ) : (
                    <>
                      <IconButton
                        icon={<FontAwesomeIcon icon={faSquareCheck} />}
                        label={t('lorebooks.actions.selectMode')}
                        onClick={() => {
                          setSelectionMode(true)
                          setSelectedLorebookIds([])
                        }}
                        size="md"
                        variant="secondary"
                      />
                      <IconButton
                        disabled={isImporting}
                        icon={<FontAwesomeIcon icon={faUpload} />}
                        label={isImporting ? t('lorebooks.actions.importing') : t('lorebooks.actions.import')}
                        onClick={() => {
                          importInputRef.current?.click()
                        }}
                        size="md"
                        variant="secondary"
                      />
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
                    </>
                  )}
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
                          {selectionMode ? (
                            <SelectionToggleButton
                              label={
                                selectedLorebookIds.includes(lorebook.lorebook_id)
                                  ? t('lorebooks.actions.deselect')
                                  : t('lorebooks.actions.select')
                              }
                              onClick={() => {
                                toggleLorebookSelection(lorebook.lorebook_id)
                              }}
                              selected={selectedLorebookIds.includes(lorebook.lorebook_id)}
                            />
                          ) : (
                            <>
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
                                  setDeleteTargetIds([lorebook.lorebook_id])
                                }}
                                size="sm"
                                variant="danger"
                              />
                            </>
                          )}
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
