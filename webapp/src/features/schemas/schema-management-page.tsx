import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react'
import { faCheckDouble } from '@fortawesome/free-solid-svg-icons/faCheckDouble'
import { faDownload } from '@fortawesome/free-solid-svg-icons/faDownload'
import { faPen } from '@fortawesome/free-solid-svg-icons/faPen'
import { faPlus } from '@fortawesome/free-solid-svg-icons/faPlus'
import { faSquareCheck } from '@fortawesome/free-solid-svg-icons/faSquareCheck'
import { faTrashCan } from '@fortawesome/free-solid-svg-icons/faTrashCan'
import { faUpload } from '@fortawesome/free-solid-svg-icons/faUpload'
import { faWandMagicSparkles } from '@fortawesome/free-solid-svg-icons/faWandMagicSparkles'
import { faXmark } from '@fortawesome/free-solid-svg-icons/faXmark'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { useTranslation } from 'react-i18next'

import { WorkspacePanelShell } from '../../components/layout/workspace-panel-shell'
import { useWorkspaceLayoutContext } from '../../components/layout/workspace-context'
import { Badge } from '../../components/ui/badge'
import { Button } from '../../components/ui/button'
import { Card, CardContent, CardHeader } from '../../components/ui/card'
import { IconButton } from '../../components/ui/icon-button'
import { SelectionToggleButton } from '../../components/ui/selection-toggle-button'
import { SectionHeader } from '../../components/ui/section-header'
import { useToastNotice } from '../../components/ui/toast-context'
import { runBatchDelete } from '../../lib/batch-delete'
import { createJsonExportFileName, downloadJsonFile, readJsonFile } from '../../lib/json-transfer'
import { createSchema, deleteSchema, listSchemas } from './api'
import { DeleteSchemaDialog } from './delete-schema-dialog'
import { SchemaFormDialog } from './schema-form-dialog'
import { SchemaPresetDialog } from './schema-preset-dialog'
import { buildSchemaPresetDefinitions } from './schema-presets'
import { createSchemaBundle, isSchemaBundle } from './schema-transfer'
import type { SchemaResource } from './types'

type NoticeTone = 'success' | 'warning' | 'error'

type NoticeState = {
  message: string
  tone: NoticeTone
}

function SchemaListSkeleton() {
  return (
    <div className="divide-y divide-[var(--color-border-subtle)]">
      {Array.from({ length: 5 }).map((_, index) => (
        <div className="grid gap-4 py-4 lg:grid-cols-[minmax(0,1.2fr)_minmax(0,0.9fr)_auto] lg:items-center" key={index}>
          <div className="space-y-2">
            <div className="h-6 w-40 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="h-3 w-56 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
          </div>
          <div className="flex flex-wrap gap-2">
            <div className="h-8 w-16 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            <div className="h-8 w-20 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
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

function countTaggedSchemas(schemas: ReadonlyArray<SchemaResource>, tag: string) {
  return schemas.filter((schema) => schema.tags.includes(tag)).length
}

export function SchemaManagementPage() {
  const { t } = useTranslation()
  const { setRailContent } = useWorkspaceLayoutContext()
  const importInputRef = useRef<HTMLInputElement | null>(null)
  const [schemas, setSchemas] = useState<SchemaResource[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [isDeleting, setIsDeleting] = useState(false)
  const [isCreatingPresets, setIsCreatingPresets] = useState(false)
  const [isImporting, setIsImporting] = useState(false)
  const [notice, setNotice] = useState<NoticeState | null>(null)
  const [createOpen, setCreateOpen] = useState(false)
  const [presetDialogOpen, setPresetDialogOpen] = useState(false)
  const [editSchemaId, setEditSchemaId] = useState<string | null>(null)
  const [selectionMode, setSelectionMode] = useState(false)
  const [selectedSchemaIds, setSelectedSchemaIds] = useState<string[]>([])
  const [deleteTargetIds, setDeleteTargetIds] = useState<string[]>([])
  useToastNotice(notice)

  const existingSchemaIds = useMemo(
    () => schemas.map((schema) => schema.schema_id),
    [schemas],
  )
  const existingSchemaIdSet = useMemo(() => new Set(existingSchemaIds), [existingSchemaIds])
  const deleteTargets = useMemo(
    () =>
      deleteTargetIds
        .map((schemaId) => schemas.find((schema) => schema.schema_id === schemaId))
        .filter((schema): schema is SchemaResource => schema !== undefined),
    [deleteTargetIds, schemas],
  )
  const schemaPresets = useMemo(() => buildSchemaPresetDefinitions(t), [t])

  const refreshSchemas = useCallback(async (signal?: AbortSignal) => {
    try {
      const nextSchemas = await listSchemas(signal)
      setSchemas(nextSchemas)
    } catch (error) {
      if (signal?.aborted) {
        return
      }

      setNotice({
        message: error instanceof Error ? error.message : t('schemas.feedback.loadFailed'),
        tone: 'error',
      })
    } finally {
      if (!signal?.aborted) {
        setIsLoading(false)
      }
    }
  }, [t])

  useEffect(() => {
    const controller = new AbortController()

    void refreshSchemas(controller.signal)

    return () => {
      controller.abort()
    }
  }, [refreshSchemas])

  useEffect(() => {
    const availableSchemaIds = new Set(schemas.map((schema) => schema.schema_id))

    setSelectedSchemaIds((currentSelection) =>
      currentSelection.filter((schemaId) => availableSchemaIds.has(schemaId)),
    )
    setDeleteTargetIds((currentSelection) =>
      currentSelection.filter((schemaId) => availableSchemaIds.has(schemaId)),
    )

    if (editSchemaId !== null && !availableSchemaIds.has(editSchemaId)) {
      setEditSchemaId(null)
    }
  }, [editSchemaId, schemas])

  useLayoutEffect(() => {
    setRailContent({
      description: t('schemas.rail.description'),
      stats: [
        { label: t('schemas.metrics.total'), value: schemas.length },
        { label: t('schemas.metrics.player'), value: countTaggedSchemas(schemas, 'player') },
        { label: t('schemas.metrics.world'), value: countTaggedSchemas(schemas, 'world') },
        { label: t('schemas.metrics.actor'), value: countTaggedSchemas(schemas, 'actor') },
      ],
      title: t('schemas.title'),
    })

    return () => {
      setRailContent(null)
    }
  }, [schemas, setRailContent, t])

  async function handleDelete() {
    if (deleteTargets.length === 0) {
      return
    }

    setIsDeleting(true)

    try {
      const result = await runBatchDelete(deleteTargets, async (target) => {
        await deleteSchema(target.schema_id)
      })

      setDeleteTargetIds([])

      if (result.deleted.length > 0) {
        const deletedIds = new Set(result.deleted.map((target) => target.schema_id))
        setSelectedSchemaIds((currentSelection) =>
          currentSelection.filter((schemaId) => !deletedIds.has(schemaId)),
        )
        setDeleteTargetIds([])

        if (editSchemaId !== null && deletedIds.has(editSchemaId)) {
          setEditSchemaId(null)
        }
      }

      if (result.failed.length === 0) {
        setNotice({
          message:
            result.deleted.length > 1
              ? t('schemas.feedback.deletedMany', { count: result.deleted.length })
              : t('schemas.feedback.deleted', { name: result.deleted[0]?.display_name ?? '' }),
          tone: 'success',
        })
        if (selectionMode) {
          setSelectionMode(false)
          setSelectedSchemaIds([])
        }
      } else if (result.deleted.length > 0) {
        setNotice({
          message: t('schemas.feedback.deletedPartial', {
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
                ? t('schemas.deleteDialog.conflictMany')
                : t('schemas.deleteDialog.conflict')
              : t('schemas.feedback.deleteFailed'),
          tone: result.conflictCount > 0 ? 'warning' : 'error',
        })
      }

      await refreshSchemas()
    } finally {
      setIsDeleting(false)
    }
  }

  function toggleSchemaSelection(schemaId: string) {
    setSelectedSchemaIds((currentSelection) =>
      currentSelection.includes(schemaId)
        ? currentSelection.filter((currentSchemaId) => currentSchemaId !== schemaId)
        : [...currentSelection, schemaId],
    )
  }

  async function handleExportSelection() {
    const exportTargets = schemas.filter((schema) => selectedSchemaIds.includes(schema.schema_id))

    if (exportTargets.length === 0) {
      return
    }

    try {
      downloadJsonFile(
        createJsonExportFileName('sillystage-schemas'),
        createSchemaBundle(exportTargets),
      )
      setNotice({
        message: t('schemas.feedback.exported', { count: exportTargets.length }),
        tone: 'success',
      })
    } catch (error) {
      setNotice({
        message: error instanceof Error ? error.message : t('schemas.feedback.exportFailed'),
        tone: 'error',
      })
    }
  }

  async function handleImportSelection(file: File) {
    setIsImporting(true)

    try {
      const payload = await readJsonFile(file)

      if (!isSchemaBundle(payload)) {
        setNotice({
          message: t('schemas.feedback.importInvalid'),
          tone: 'error',
        })
        return
      }

      const existingIds = new Set(schemas.map((schema) => schema.schema_id))
      const createdNames: string[] = []
      const skippedNames: string[] = []
      const failedNames: string[] = []

      for (const schema of payload.schemas) {
        if (existingIds.has(schema.schema_id)) {
          skippedNames.push(schema.display_name)
          continue
        }

        try {
          await createSchema({
            display_name: schema.display_name,
            fields: schema.fields,
            schema_id: schema.schema_id,
            tags: schema.tags,
          })
          createdNames.push(schema.display_name)
          existingIds.add(schema.schema_id)
        } catch {
          failedNames.push(schema.display_name)
        }
      }

      if (createdNames.length > 0) {
        await refreshSchemas()
      }

      if (createdNames.length > 0 && skippedNames.length === 0 && failedNames.length === 0) {
        setNotice({
          message: t('schemas.feedback.imported', { count: createdNames.length }),
          tone: 'success',
        })
      } else if (createdNames.length > 0 || skippedNames.length > 0) {
        setNotice({
          message: t('schemas.feedback.importedPartial', {
            failed: failedNames.length,
            skipped: skippedNames.length,
            success: createdNames.length,
          }),
          tone: failedNames.length > 0 ? 'warning' : 'success',
        })
      } else {
        setNotice({
          message: t('schemas.feedback.importSkipped', { count: skippedNames.length }),
          tone: 'warning',
        })
      }
    } catch (error) {
      setNotice({
        message: error instanceof Error ? error.message : t('schemas.feedback.importFailed'),
        tone: 'error',
      })
    } finally {
      setIsImporting(false)
    }
  }

  async function handleCreatePresets(presetKinds: ReadonlyArray<(typeof schemaPresets)[number]['kind']>) {
    const presetLookup = new Map(schemaPresets.map((preset) => [preset.kind, preset]))
    const createdNames: string[] = []
    const failedNames: string[] = []

    setIsCreatingPresets(true)

    try {
      for (const presetKind of presetKinds) {
        const preset = presetLookup.get(presetKind)

        if (!preset || existingSchemaIdSet.has(preset.schemaId)) {
          continue
        }

        try {
          await createSchema({
            display_name: preset.displayName,
            fields: preset.fields,
            schema_id: preset.schemaId,
            tags: preset.tags,
          })
          createdNames.push(preset.displayName)
        } catch {
          failedNames.push(preset.displayName)
        }
      }

      if (createdNames.length > 0) {
        await refreshSchemas()
      }

      if (failedNames.length === 0 && createdNames.length > 0) {
        setNotice({
          message: t('schemas.feedback.presetsCreated', {
            names: createdNames.join('、'),
          }),
          tone: 'success',
        })
      } else if (createdNames.length > 0 && failedNames.length > 0) {
        setNotice({
          message: t('schemas.feedback.presetsCreatedPartial', {
            created: createdNames.join('、'),
            failed: failedNames.join('、'),
          }),
          tone: 'warning',
        })
      } else if (failedNames.length > 0) {
        setNotice({
          message: t('schemas.feedback.presetsCreateFailed', {
            failed: failedNames.join('、'),
          }),
          tone: 'error',
        })
      }

      setPresetDialogOpen(false)
    } finally {
      setIsCreatingPresets(false)
    }
  }

  return (
    <div className="flex h-full min-h-0 flex-col gap-6">
      <SchemaFormDialog
        existingSchemaIds={existingSchemaIds}
        mode="create"
        onCompleted={async ({ message }) => {
          setNotice({ message, tone: 'success' })
          await refreshSchemas()
        }}
        onOpenChange={setCreateOpen}
        open={createOpen}
      />

      <SchemaPresetDialog
        creating={isCreatingPresets}
        existingSchemaIds={existingSchemaIdSet}
        onConfirm={async (presetKinds) => {
          await handleCreatePresets(presetKinds)
        }}
        onOpenChange={setPresetDialogOpen}
        open={presetDialogOpen}
        presets={schemaPresets}
      />

      <SchemaFormDialog
        existingSchemaIds={existingSchemaIds}
        mode="edit"
        onCompleted={async ({ message }) => {
          setNotice({ message, tone: 'success' })
          await refreshSchemas()
        }}
        onOpenChange={(open) => {
          if (!open) {
            setEditSchemaId(null)
          }
        }}
        open={editSchemaId !== null}
        schemaId={editSchemaId}
      />

      <DeleteSchemaDialog
        deleting={isDeleting}
        onConfirm={() => {
          void handleDelete()
        }}
        onOpenChange={() => {
          setDeleteTargetIds([])
        }}
        targets={deleteTargets}
      />

      <input
        accept="application/json,.json"
        className="sr-only"
        name="schema_import"
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
                      {t('schemas.selection.count', { count: selectedSchemaIds.length })}
                    </Badge>
                    <IconButton
                      disabled={schemas.length === 0}
                      icon={<FontAwesomeIcon icon={faCheckDouble} />}
                      label={t('schemas.actions.selectAll')}
                      onClick={() => {
                        setSelectedSchemaIds(schemas.map((schema) => schema.schema_id))
                      }}
                      size="md"
                      variant="secondary"
                    />
                    <IconButton
                      disabled={selectedSchemaIds.length === 0}
                      icon={<FontAwesomeIcon icon={faDownload} />}
                      label={t('schemas.actions.export')}
                      onClick={() => {
                        void handleExportSelection()
                      }}
                      size="md"
                      variant="secondary"
                    />
                    <IconButton
                      disabled={selectedSchemaIds.length === 0}
                      icon={<FontAwesomeIcon icon={faTrashCan} />}
                      label={t('schemas.actions.deleteSelected')}
                      onClick={() => {
                        setDeleteTargetIds(selectedSchemaIds)
                      }}
                      size="md"
                      variant="danger"
                    />
                    <IconButton
                      icon={<FontAwesomeIcon icon={faXmark} />}
                      label={t('schemas.actions.cancelSelection')}
                      onClick={() => {
                        setSelectionMode(false)
                        setSelectedSchemaIds([])
                      }}
                      size="md"
                      variant="secondary"
                    />
                  </>
                ) : (
                  <>
                    <IconButton
                      icon={<FontAwesomeIcon icon={faSquareCheck} />}
                      label={t('schemas.actions.selectMode')}
                      onClick={() => {
                        setSelectionMode(true)
                        setSelectedSchemaIds([])
                      }}
                      size="md"
                      variant="secondary"
                    />
                    <IconButton
                      disabled={isImporting}
                      icon={<FontAwesomeIcon icon={faUpload} />}
                      label={isImporting ? t('schemas.actions.importing') : t('schemas.actions.import')}
                      onClick={() => {
                        importInputRef.current?.click()
                      }}
                      size="md"
                      variant="secondary"
                    />
                    <IconButton
                      icon={<FontAwesomeIcon icon={faWandMagicSparkles} />}
                      label={t('schemas.actions.createPreset')}
                      onClick={() => {
                        setPresetDialogOpen(true)
                      }}
                      size="md"
                      variant="secondary"
                    />
                    <IconButton
                      icon={<FontAwesomeIcon icon={faPlus} />}
                      label={t('schemas.actions.create')}
                      onClick={() => {
                        setCreateOpen(true)
                      }}
                      size="md"
                    />
                  </>
                )}
              </div>
            }
            title={t('schemas.title')}
          />
        </CardHeader>

        <CardContent className="min-h-0 flex-1 overflow-y-auto pt-6">
          <div className="space-y-6 pr-1">
            {isLoading ? (
              <SchemaListSkeleton />
            ) : schemas.length === 0 ? (
              <div className="py-12 text-center">
                <h3 className="font-display text-3xl text-[var(--color-text-primary)]">
                  {t('schemas.empty.title')}
                </h3>

                <div className="mt-7 flex justify-center">
                  <div className="flex flex-wrap justify-center gap-3">
                    <Button
                      onClick={() => {
                        setPresetDialogOpen(true)
                      }}
                      variant="secondary"
                    >
                      {t('schemas.actions.createPreset')}
                    </Button>
                    <Button
                      onClick={() => {
                        setCreateOpen(true)
                      }}
                    >
                      {t('schemas.actions.create')}
                    </Button>
                  </div>
                </div>
              </div>
            ) : (
              <div className="divide-y divide-[var(--color-border-subtle)]">
                {schemas.map((schema) => (
                  <div
                    className="grid gap-4 py-4 lg:grid-cols-[minmax(0,1.2fr)_minmax(0,0.9fr)_auto] lg:items-center"
                    key={schema.schema_id}
                  >
                    <div className="min-w-0 space-y-2">
                      <h3 className="truncate font-display text-[1.32rem] leading-tight text-[var(--color-text-primary)]">
                        {schema.display_name}
                      </h3>
                      <p className="truncate font-mono text-[0.76rem] leading-5 text-[var(--color-text-muted)]">
                        {schema.schema_id}
                      </p>
                    </div>

                    <div className="space-y-2">
                      <p className="text-xs text-[var(--color-text-muted)]">
                        {t('schemas.list.fieldsCount', { count: Object.keys(schema.fields).length })}
                      </p>
                      <div className="flex flex-wrap gap-2">
                        {schema.tags.length > 0 ? (
                          schema.tags.map((tag) => (
                            <Badge className="normal-case px-3 py-1.5" key={tag} variant="subtle">
                              {tag}
                            </Badge>
                          ))
                        ) : (
                          <Badge className="normal-case px-3 py-1.5" variant="subtle">
                            {t('schemas.list.noTags')}
                          </Badge>
                        )}
                      </div>
                    </div>

                    <div className="flex flex-wrap items-center justify-start gap-2 lg:justify-end">
                      {selectionMode ? (
                        <SelectionToggleButton
                          label={
                            selectedSchemaIds.includes(schema.schema_id)
                              ? t('schemas.actions.deselect')
                              : t('schemas.actions.select')
                          }
                          onClick={() => {
                            toggleSchemaSelection(schema.schema_id)
                          }}
                          selected={selectedSchemaIds.includes(schema.schema_id)}
                        />
                      ) : (
                        <>
                          <IconButton
                            icon={<FontAwesomeIcon icon={faPen} />}
                            label={t('schemas.actions.edit')}
                            onClick={() => {
                              setEditSchemaId(schema.schema_id)
                            }}
                            size="sm"
                            variant="secondary"
                          />
                          <IconButton
                            icon={<FontAwesomeIcon icon={faTrashCan} />}
                            label={t('schemas.actions.delete')}
                            onClick={() => {
                              setDeleteTargetIds([schema.schema_id])
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
            )}
          </div>
        </CardContent>
        </Card>
      </WorkspacePanelShell>
    </div>
  )
}
