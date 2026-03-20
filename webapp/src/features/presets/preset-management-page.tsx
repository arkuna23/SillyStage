import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react'
import { faCheckDouble } from '@fortawesome/free-solid-svg-icons/faCheckDouble'
import { faDownload } from '@fortawesome/free-solid-svg-icons/faDownload'
import { faPen } from '@fortawesome/free-solid-svg-icons/faPen'
import { faPlus } from '@fortawesome/free-solid-svg-icons/faPlus'
import { faSquareCheck } from '@fortawesome/free-solid-svg-icons/faSquareCheck'
import { faWandMagicSparkles } from '@fortawesome/free-solid-svg-icons/faWandMagicSparkles'
import { faTrashCan } from '@fortawesome/free-solid-svg-icons/faTrashCan'
import { faUpload } from '@fortawesome/free-solid-svg-icons/faUpload'
import { faXmark } from '@fortawesome/free-solid-svg-icons/faXmark'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { useTranslation } from 'react-i18next'

import { WorkspacePanelShell } from '../../components/layout/workspace-panel-shell'
import { useWorkspaceLayoutContext } from '../../components/layout/workspace-context'
import { Badge } from '../../components/ui/badge'
import { Card, CardContent, CardHeader } from '../../components/ui/card'
import { IconButton } from '../../components/ui/icon-button'
import { SelectionToggleButton } from '../../components/ui/selection-toggle-button'
import { SectionHeader } from '../../components/ui/section-header'
import { useToastNotice } from '../../components/ui/toast-context'
import { runBatchDelete } from '../../lib/batch-delete'
import { createJsonExportFileName, downloadJsonFile, readJsonFile } from '../../lib/json-transfer'
import { createPreset, deletePreset, getPreset, listPresets } from '../apis/api'
import { hasPresetAgentConfiguration, type Preset } from '../apis/types'
import { DeletePresetDialog } from './delete-preset-dialog'
import { PresetFormDialog } from './preset-form-dialog'
import { getOrderedAgentRoleKeys } from './preset-labels'
import { buildPresetTemplateDefinitions } from './preset-presets'
import { createPresetBundle, isPresetBundle } from './preset-transfer'
import { PresetTemplateDialog } from './preset-template-dialog'

type NoticeTone = 'error' | 'success' | 'warning'

type Notice = {
  message: string
  tone: NoticeTone
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function PresetsListSkeleton() {
  return (
    <div className="space-y-5">
      <div className="h-8 w-44 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
      <div className="divide-y divide-[var(--color-border-subtle)]">
        {Array.from({ length: 5 }).map((_, index) => (
          <div
            className="grid gap-4 py-4 lg:grid-cols-[minmax(0,1fr)_auto] lg:items-center"
            key={index}
          >
            <div className="space-y-2.5">
              <div className="h-5 w-32 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
              <div className="h-3 w-24 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            </div>
            <div className="flex justify-end gap-2">
              <div className="h-9 w-16 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
              <div className="h-9 w-16 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
              <div className="h-9 w-16 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}

function countConfiguredPresetAgents(preset: Preset) {
  return getOrderedAgentRoleKeys().reduce((count, roleKey) => {
    return count + (hasPresetAgentConfiguration(preset.agents[roleKey]) ? 1 : 0)
  }, 0)
}

export function PresetManagementPage() {
  const { t } = useTranslation()
  const { setRailContent } = useWorkspaceLayoutContext()
  const importInputRef = useRef<HTMLInputElement | null>(null)
  const [presets, setPresets] = useState<Preset[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [notice, setNotice] = useState<Notice | null>(null)
  const [isDeleting, setIsDeleting] = useState(false)
  const [isCreatingTemplates, setIsCreatingTemplates] = useState(false)
  const [isImporting, setIsImporting] = useState(false)
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false)
  const [isTemplateDialogOpen, setIsTemplateDialogOpen] = useState(false)
  const [editPresetId, setEditPresetId] = useState<string | null>(null)
  const [selectionMode, setSelectionMode] = useState(false)
  const [selectedPresetIds, setSelectedPresetIds] = useState<string[]>([])
  const [deleteTargetIds, setDeleteTargetIds] = useState<string[]>([])
  useToastNotice(notice)

  const presetIds = useMemo(() => presets.map((preset) => preset.preset_id), [presets])
  const existingPresetIdSet = useMemo(() => new Set(presetIds), [presetIds])
  const deleteTargets = useMemo(
    () =>
      deleteTargetIds
        .map((presetId) => presets.find((preset) => preset.preset_id === presetId))
        .filter((preset): preset is Preset => preset !== undefined),
    [deleteTargetIds, presets],
  )
  const configuredPresetCount = useMemo(
    () => presets.filter((preset) => countConfiguredPresetAgents(preset) > 0).length,
    [presets],
  )
  const presetTemplates = useMemo(() => buildPresetTemplateDefinitions(t), [t])

  const refreshPresets = useCallback(
    async (signal?: AbortSignal) => {
      setIsLoading(true)

      try {
        const nextPresets = await listPresets(signal)

        if (!signal?.aborted) {
          setPresets(nextPresets)
        }
      } catch (error) {
        if (!signal?.aborted) {
          setNotice({
            message: getErrorMessage(error, t('presetsPage.feedback.loadListFailed')),
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
    void refreshPresets(controller.signal)

    return () => {
      controller.abort()
    }
  }, [refreshPresets])

  useEffect(() => {
    const availablePresetIds = new Set(presets.map((preset) => preset.preset_id))

    setSelectedPresetIds((currentSelection) =>
      currentSelection.filter((presetId) => availablePresetIds.has(presetId)),
    )
    setDeleteTargetIds((currentSelection) =>
      currentSelection.filter((presetId) => availablePresetIds.has(presetId)),
    )

    if (editPresetId !== null && !availablePresetIds.has(editPresetId)) {
      setEditPresetId(null)
    }

  }, [editPresetId, presets])

  useLayoutEffect(() => {
    setRailContent({
      description: t('presetsPage.rail.description'),
      stats: [
        {
          label: t('presetsPage.metrics.total'),
          value: presets.length,
        },
        {
          label: t('presetsPage.metrics.configured'),
          value: configuredPresetCount,
        },
      ],
      title: t('presetsPage.title'),
    })

    return () => {
      setRailContent(null)
    }
  }, [configuredPresetCount, presets.length, setRailContent, t])

  async function handleDeletePreset() {
    if (deleteTargets.length === 0) {
      return
    }

    setIsDeleting(true)

    try {
      const result = await runBatchDelete(deleteTargets, async (target) => {
        await deletePreset(target.preset_id)
      })

      setDeleteTargetIds([])

      if (result.deleted.length > 0) {
        const deletedIds = new Set(result.deleted.map((target) => target.preset_id))

        setSelectedPresetIds((currentSelection) =>
          currentSelection.filter((presetId) => !deletedIds.has(presetId)),
        )
        setDeleteTargetIds([])

        if (editPresetId !== null && deletedIds.has(editPresetId)) {
          setEditPresetId(null)
        }

      }

      if (result.failed.length === 0) {
        setNotice({
          message:
            result.deleted.length > 1
              ? t('presetsPage.feedback.deletedMany', { count: result.deleted.length })
              : t('presetsPage.feedback.deleted', { id: result.deleted[0]?.display_name ?? '' }),
          tone: 'success',
        })
        if (selectionMode) {
          setSelectionMode(false)
          setSelectedPresetIds([])
        }
      } else if (result.deleted.length > 0) {
        setNotice({
          message: t('presetsPage.feedback.deletedPartial', {
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
                ? t('presetsPage.deleteDialog.conflictMany')
                : t('presetsPage.deleteDialog.conflict')
              : t('presetsPage.feedback.deleteFailed'),
          tone: result.conflictCount > 0 ? 'warning' : 'error',
        })
      }

      await refreshPresets()
    } finally {
      setIsDeleting(false)
    }
  }

  function togglePresetSelection(presetId: string) {
    setSelectedPresetIds((currentSelection) =>
      currentSelection.includes(presetId)
        ? currentSelection.filter((currentPresetId) => currentPresetId !== presetId)
        : [...currentSelection, presetId],
    )
  }

  async function handleExportSelection() {
    const exportTargets = presets.filter((preset) => selectedPresetIds.includes(preset.preset_id))

    if (exportTargets.length === 0) {
      return
    }

    try {
      const fullPresets = await Promise.all(
        exportTargets.map((preset) => getPreset(preset.preset_id)),
      )

      downloadJsonFile(
        createJsonExportFileName('sillystage-presets'),
        createPresetBundle(fullPresets),
      )
      setNotice({
        message: t('presetsPage.feedback.exported', { count: exportTargets.length }),
        tone: 'success',
      })
    } catch (error) {
      setNotice({
        message: getErrorMessage(error, t('presetsPage.feedback.exportFailed')),
        tone: 'error',
      })
    }
  }

  async function handleImportSelection(file: File) {
    setIsImporting(true)

    try {
      const payload = await readJsonFile(file)

      if (!isPresetBundle(payload)) {
        setNotice({
          message: t('presetsPage.feedback.importInvalid'),
          tone: 'error',
        })
        return
      }

      const existingIds = new Set(presets.map((preset) => preset.preset_id))
      const createdNames: string[] = []
      const skippedNames: string[] = []
      const failedNames: string[] = []

      for (const preset of payload.presets) {
        if (existingIds.has(preset.preset_id)) {
          skippedNames.push(preset.display_name)
          continue
        }

        try {
          await createPreset({
            agents: preset.agents,
            display_name: preset.display_name,
            preset_id: preset.preset_id,
          })
          createdNames.push(preset.display_name)
          existingIds.add(preset.preset_id)
        } catch {
          failedNames.push(preset.display_name)
        }
      }

      if (createdNames.length > 0) {
        await refreshPresets()
      }

      if (createdNames.length > 0 && skippedNames.length === 0 && failedNames.length === 0) {
        setNotice({
          message: t('presetsPage.feedback.imported', { count: createdNames.length }),
          tone: 'success',
        })
      } else if (createdNames.length > 0 || skippedNames.length > 0) {
        setNotice({
          message: t('presetsPage.feedback.importedPartial', {
            failed: failedNames.length,
            skipped: skippedNames.length,
            success: createdNames.length,
          }),
          tone: failedNames.length > 0 ? 'warning' : 'success',
        })
      } else {
        setNotice({
          message: t('presetsPage.feedback.importSkipped', { count: skippedNames.length }),
          tone: 'warning',
        })
      }
    } catch (error) {
      setNotice({
        message: getErrorMessage(error, t('presetsPage.feedback.importFailed')),
        tone: 'error',
      })
    } finally {
      setIsImporting(false)
    }
  }

  async function handleCreateTemplates(
    templateKinds: ReadonlyArray<(typeof presetTemplates)[number]['kind']>,
  ) {
    const templateLookup = new Map(presetTemplates.map((preset) => [preset.kind, preset]))
    const createdNames: string[] = []
    const failedNames: string[] = []

    setIsCreatingTemplates(true)

    try {
      for (const templateKind of templateKinds) {
        const preset = templateLookup.get(templateKind)

        if (!preset || existingPresetIdSet.has(preset.presetId)) {
          continue
        }

        try {
          await createPreset({
            agents: preset.agents,
            display_name: preset.displayName,
            preset_id: preset.presetId,
          })
          createdNames.push(preset.displayName)
        } catch {
          failedNames.push(preset.displayName)
        }
      }

      if (createdNames.length > 0) {
        await refreshPresets()
      }

      if (failedNames.length === 0 && createdNames.length > 0) {
        setNotice({
          message: t('presetsPage.feedback.templatesCreated', {
            names: createdNames.join('、'),
          }),
          tone: 'success',
        })
      } else if (createdNames.length > 0 && failedNames.length > 0) {
        setNotice({
          message: t('presetsPage.feedback.templatesCreatedPartial', {
            created: createdNames.join('、'),
            failed: failedNames.join('、'),
          }),
          tone: 'warning',
        })
      } else if (failedNames.length > 0) {
        setNotice({
          message: t('presetsPage.feedback.templatesCreateFailed', {
            failed: failedNames.join('、'),
          }),
          tone: 'error',
        })
      }

      setIsTemplateDialogOpen(false)
    } finally {
      setIsCreatingTemplates(false)
    }
  }

  return (
    <div className="flex h-full min-h-0 flex-col gap-6">
      <PresetFormDialog
        existingPresetIds={presetIds}
        mode="create"
        onCompleted={async ({ message }) => {
          setNotice({ message, tone: 'success' })
          await refreshPresets()
        }}
        onOpenChange={setIsCreateDialogOpen}
        open={isCreateDialogOpen}
      />

      <PresetTemplateDialog
        creating={isCreatingTemplates}
        existingPresetIds={existingPresetIdSet}
        onConfirm={async (templateKinds) => {
          await handleCreateTemplates(templateKinds)
        }}
        onOpenChange={setIsTemplateDialogOpen}
        open={isTemplateDialogOpen}
        presets={presetTemplates}
      />

      <PresetFormDialog
        existingPresetIds={presetIds}
        mode="edit"
        onCompleted={async ({ message }) => {
          setNotice({ message, tone: 'success' })
          await refreshPresets()
        }}
        onOpenChange={(open) => {
          if (!open) {
            setEditPresetId(null)
          }
        }}
        open={editPresetId !== null}
        presetId={editPresetId}
      />

      <DeletePresetDialog
        deleting={isDeleting}
        onConfirm={() => void handleDeletePreset()}
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
        name="preset_import"
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

      <WorkspacePanelShell className="h-full min-h-0">
        <Card className="flex h-full min-h-0 flex-col overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_94%,transparent)] shadow-none">
          <CardHeader className="border-b border-[var(--color-border-subtle)] md:min-h-[92px]">
            <SectionHeader
              actions={
                <div className="flex flex-wrap items-center gap-2">
                  {selectionMode ? (
                    <>
                      <Badge className="normal-case px-3.5 py-2" variant="subtle">
                        {t('presetsPage.selection.count', { count: selectedPresetIds.length })}
                      </Badge>
                      <IconButton
                        disabled={presets.length === 0}
                        icon={<FontAwesomeIcon icon={faCheckDouble} />}
                        label={t('presetsPage.actions.selectAll')}
                        onClick={() => {
                          setSelectedPresetIds(presets.map((preset) => preset.preset_id))
                        }}
                        variant="secondary"
                      />
                      <IconButton
                        disabled={selectedPresetIds.length === 0}
                        icon={<FontAwesomeIcon icon={faDownload} />}
                        label={t('presetsPage.actions.export')}
                        onClick={() => {
                          void handleExportSelection()
                        }}
                        variant="secondary"
                      />
                      <IconButton
                        disabled={selectedPresetIds.length === 0}
                        icon={<FontAwesomeIcon icon={faTrashCan} />}
                        label={t('presetsPage.actions.deleteSelected')}
                        onClick={() => {
                          setDeleteTargetIds(selectedPresetIds)
                        }}
                        variant="danger"
                      />
                      <IconButton
                        icon={<FontAwesomeIcon icon={faXmark} />}
                        label={t('presetsPage.actions.cancelSelection')}
                        onClick={() => {
                          setSelectionMode(false)
                          setSelectedPresetIds([])
                        }}
                        variant="secondary"
                      />
                    </>
                  ) : (
                    <>
                      <IconButton
                        icon={<FontAwesomeIcon icon={faSquareCheck} />}
                        label={t('presetsPage.actions.selectMode')}
                        onClick={() => {
                          setSelectionMode(true)
                          setSelectedPresetIds([])
                        }}
                        variant="secondary"
                      />
                      <IconButton
                        disabled={isImporting}
                        icon={<FontAwesomeIcon icon={faUpload} />}
                        label={
                          isImporting
                            ? t('presetsPage.actions.importing')
                            : t('presetsPage.actions.import')
                        }
                        onClick={() => {
                          importInputRef.current?.click()
                        }}
                        variant="secondary"
                      />
                      <IconButton
                        icon={<FontAwesomeIcon icon={faWandMagicSparkles} />}
                        label={t('presetsPage.actions.createTemplates')}
                        onClick={() => {
                          setIsTemplateDialogOpen(true)
                        }}
                        variant="secondary"
                      />
                      <IconButton
                        icon={<FontAwesomeIcon icon={faPlus} />}
                        label={t('presetsPage.actions.create')}
                        onClick={() => {
                          setIsCreateDialogOpen(true)
                        }}
                      />
                    </>
                  )}
                </div>
              }
              title={t('presetsPage.title')}
            />
          </CardHeader>

          <CardContent className="scrollbar-none min-h-0 flex-1 overflow-y-auto pt-6">
            <div className="space-y-6">
              {isLoading ? (
                <PresetsListSkeleton />
              ) : presets.length === 0 ? (
                <div className="rounded-[1.45rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_72%,transparent)] px-4 py-5 text-sm leading-7 text-[var(--color-text-secondary)]">
                  {t('presetsPage.empty.title')}
                </div>
              ) : (
                <div className="space-y-5">
                  <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
                    {t('presetsPage.list.title')}
                  </p>

                  <div className="divide-y divide-[var(--color-border-subtle)]">
                    {presets.map((preset) => (
                      <div
                        className="grid gap-4 py-4 lg:grid-cols-[minmax(0,1fr)_auto] lg:items-center"
                        key={preset.preset_id}
                      >
                        <div className="space-y-2">
                          <p className="font-medium text-[var(--color-text-primary)]">
                            {preset.display_name}
                          </p>
                          <p className="text-xs text-[var(--color-text-muted)]">{preset.preset_id}</p>
                        </div>

                        <div className="flex justify-start gap-2 lg:justify-end">
                          {selectionMode ? (
                            <SelectionToggleButton
                              label={
                                selectedPresetIds.includes(preset.preset_id)
                                  ? t('presetsPage.actions.deselect')
                                  : t('presetsPage.actions.select')
                              }
                              onClick={() => {
                                togglePresetSelection(preset.preset_id)
                              }}
                              selected={selectedPresetIds.includes(preset.preset_id)}
                            />
                          ) : (
                            <>
                              <IconButton
                                icon={<FontAwesomeIcon icon={faPen} />}
                                label={t('presetsPage.actions.edit')}
                                onClick={() => {
                                  setEditPresetId(preset.preset_id)
                                }}
                                size="sm"
                                variant="secondary"
                              />
                              <IconButton
                                icon={<FontAwesomeIcon icon={faTrashCan} />}
                                label={t('presetsPage.actions.delete')}
                                onClick={() => {
                                  setDeleteTargetIds([preset.preset_id])
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
