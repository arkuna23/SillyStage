import { useCallback, useEffect, useLayoutEffect, useMemo, useState } from 'react'
import { faPen } from '@fortawesome/free-solid-svg-icons/faPen'
import { faPlus } from '@fortawesome/free-solid-svg-icons/faPlus'
import { faTrashCan } from '@fortawesome/free-solid-svg-icons/faTrashCan'
import { faWandMagicSparkles } from '@fortawesome/free-solid-svg-icons/faWandMagicSparkles'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { useTranslation } from 'react-i18next'

import { WorkspacePanelShell } from '../../components/layout/workspace-panel-shell'
import { useWorkspaceLayoutContext } from '../../components/layout/workspace-context'
import { Badge } from '../../components/ui/badge'
import { Button } from '../../components/ui/button'
import { Card, CardContent, CardHeader } from '../../components/ui/card'
import { IconButton } from '../../components/ui/icon-button'
import { SectionHeader } from '../../components/ui/section-header'
import { useToastNotice } from '../../components/ui/toast-context'
import { isRpcConflict } from '../../lib/rpc'
import { createSchema, deleteSchema, listSchemas } from './api'
import { DeleteSchemaDialog } from './delete-schema-dialog'
import { SchemaFormDialog } from './schema-form-dialog'
import { SchemaPresetDialog } from './schema-preset-dialog'
import { buildSchemaPresetDefinitions } from './schema-presets'
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
  const [schemas, setSchemas] = useState<SchemaResource[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [isDeleting, setIsDeleting] = useState(false)
  const [isCreatingPresets, setIsCreatingPresets] = useState(false)
  const [notice, setNotice] = useState<NoticeState | null>(null)
  const [createOpen, setCreateOpen] = useState(false)
  const [presetDialogOpen, setPresetDialogOpen] = useState(false)
  const [editSchemaId, setEditSchemaId] = useState<string | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<SchemaResource | null>(null)
  useToastNotice(notice)

  const existingSchemaIds = useMemo(
    () => schemas.map((schema) => schema.schema_id),
    [schemas],
  )
  const existingSchemaIdSet = useMemo(() => new Set(existingSchemaIds), [existingSchemaIds])
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
    if (!deleteTarget) {
      return
    }

    const target = deleteTarget
    setIsDeleting(true)

    try {
      await deleteSchema(target.schema_id)
      setNotice({
        message: t('schemas.feedback.deleted', { name: target.display_name }),
        tone: 'success',
      })
      setDeleteTarget(null)
      await refreshSchemas()
    } catch (error) {
      setDeleteTarget(null)
      setNotice({
        message:
          isRpcConflict(error)
            ? t('schemas.deleteDialog.conflict')
            : error instanceof Error
              ? error.message
              : t('schemas.feedback.deleteFailed'),
        tone: isRpcConflict(error) ? 'warning' : 'error',
      })
    } finally {
      setIsDeleting(false)
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
          setDeleteTarget(null)
        }}
        schema={deleteTarget}
      />

      <WorkspacePanelShell className="flex min-h-0 flex-1">
        <Card className="flex min-h-0 flex-1 flex-col overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_94%,transparent)] shadow-none">
        <CardHeader className="gap-4 border-b border-[var(--color-border-subtle)] px-6 py-5 md:min-h-[5.75rem] md:px-7 md:py-5">
          <SectionHeader
            actions={
              <div className="flex min-h-10 items-center justify-end gap-2">
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
                          setDeleteTarget(schema)
                        }}
                        size="sm"
                        variant="danger"
                      />
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
