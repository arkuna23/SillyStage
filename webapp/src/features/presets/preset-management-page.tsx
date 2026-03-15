import { useCallback, useEffect, useLayoutEffect, useMemo, useState } from 'react'
import { faEye } from '@fortawesome/free-solid-svg-icons/faEye'
import { faPen } from '@fortawesome/free-solid-svg-icons/faPen'
import { faPlus } from '@fortawesome/free-solid-svg-icons/faPlus'
import { faWandMagicSparkles } from '@fortawesome/free-solid-svg-icons/faWandMagicSparkles'
import { faTrashCan } from '@fortawesome/free-solid-svg-icons/faTrashCan'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { useTranslation } from 'react-i18next'

import { WorkspacePanelShell } from '../../components/layout/workspace-panel-shell'
import { useWorkspaceLayoutContext } from '../../components/layout/workspace-context'
import { Badge } from '../../components/ui/badge'
import { Card, CardContent, CardHeader } from '../../components/ui/card'
import { IconButton } from '../../components/ui/icon-button'
import { SectionHeader } from '../../components/ui/section-header'
import { useToastNotice } from '../../components/ui/toast-context'
import { isRpcConflict } from '../../lib/rpc'
import { createPreset, deletePreset, listPresets } from '../apis/api'
import { agentRoleKeys, type AgentRoleKey, type Preset } from '../apis/types'
import { DeletePresetDialog } from './delete-preset-dialog'
import { PresetDetailsDialog } from './preset-details-dialog'
import { PresetFormDialog } from './preset-form-dialog'
import { buildPresetTemplateDefinitions } from './preset-presets'
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
            className="grid gap-4 py-4 lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_auto] lg:items-center"
            key={index}
          >
            <div className="space-y-2.5">
              <div className="h-5 w-32 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
              <div className="h-3 w-24 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            </div>
            <div className="flex flex-wrap gap-2">
              {Array.from({ length: 3 }).map((__, badgeIndex) => (
                <div
                  className="h-8 w-24 animate-pulse rounded-full bg-[var(--color-bg-elevated)]"
                  key={badgeIndex}
                />
              ))}
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
  return agentRoleKeys.reduce((count, roleKey) => {
    const agent = preset.agents[roleKey]
    const hasConfig =
      agent.temperature !== undefined ||
      agent.max_tokens !== undefined ||
      agent.extra !== undefined

    return count + (hasConfig ? 1 : 0)
  }, 0)
}

function describePresetAgent(
  preset: Preset,
  roleKey: AgentRoleKey,
  emptyLabel: string,
) {
  const agent = preset.agents[roleKey]
  const parts = [
    agent.temperature !== undefined && agent.temperature !== null ? `T ${agent.temperature}` : null,
    agent.max_tokens !== undefined && agent.max_tokens !== null ? `Max ${agent.max_tokens}` : null,
    agent.extra ? 'extra' : null,
  ].filter((value): value is string => Boolean(value))

  return parts.length > 0 ? parts.join(' · ') : emptyLabel
}

export function PresetManagementPage() {
  const { t } = useTranslation()
  const { setRailContent } = useWorkspaceLayoutContext()
  const [presets, setPresets] = useState<Preset[]>([])
  const [isLoading, setIsLoading] = useState(true)
  const [notice, setNotice] = useState<Notice | null>(null)
  const [isDeleting, setIsDeleting] = useState(false)
  const [isCreatingTemplates, setIsCreatingTemplates] = useState(false)
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false)
  const [isTemplateDialogOpen, setIsTemplateDialogOpen] = useState(false)
  const [editPresetId, setEditPresetId] = useState<string | null>(null)
  const [detailsPresetId, setDetailsPresetId] = useState<string | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<Preset | null>(null)
  useToastNotice(notice)

  const presetIds = useMemo(() => presets.map((preset) => preset.preset_id), [presets])
  const existingPresetIdSet = useMemo(() => new Set(presetIds), [presetIds])
  const configuredPresetCount = useMemo(
    () => presets.filter((preset) => countConfiguredPresetAgents(preset) > 0).length,
    [presets],
  )
  const presetTemplates = useMemo(() => buildPresetTemplateDefinitions(t), [t])

  const roleLabels: Record<AgentRoleKey, string> = useMemo(
    () => ({
      actor: t('presetsPage.roles.actor'),
      architect: t('presetsPage.roles.architect'),
      director: t('presetsPage.roles.director'),
      keeper: t('presetsPage.roles.keeper'),
      narrator: t('presetsPage.roles.narrator'),
      planner: t('presetsPage.roles.planner'),
      replyer: t('presetsPage.roles.replyer'),
    }),
    [t],
  )

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
    if (!deleteTarget) {
      return
    }

    const target = deleteTarget
    setIsDeleting(true)

    try {
      await deletePreset(target.preset_id)
      setNotice({
        message: t('presetsPage.feedback.deleted', { id: target.display_name }),
        tone: 'success',
      })
      setDeleteTarget(null)
      await refreshPresets()
    } catch (error) {
      setDeleteTarget(null)
      setNotice({
        message: isRpcConflict(error)
          ? t('presetsPage.deleteDialog.conflict')
          : getErrorMessage(error, t('presetsPage.feedback.deleteFailed')),
        tone: isRpcConflict(error) ? 'warning' : 'error',
      })
    } finally {
      setIsDeleting(false)
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

      <PresetDetailsDialog
        onOpenChange={(open) => {
          if (!open) {
            setDetailsPresetId(null)
          }
        }}
        open={detailsPresetId !== null}
        presetId={detailsPresetId}
      />

      <DeletePresetDialog
        deleting={isDeleting}
        onConfirm={() => void handleDeletePreset()}
        onOpenChange={(open) => {
          if (!open) {
            setDeleteTarget(null)
          }
        }}
        open={deleteTarget !== null}
        preset={deleteTarget}
      />

      <WorkspacePanelShell className="h-full min-h-0">
        <Card className="flex h-full min-h-0 flex-col overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_94%,transparent)] shadow-none">
          <CardHeader className="border-b border-[var(--color-border-subtle)] md:min-h-[92px]">
            <SectionHeader
              actions={
                <div className="flex items-center gap-3">
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
                        className="grid gap-4 py-4 lg:grid-cols-[minmax(0,1fr)_minmax(0,1fr)_auto] lg:items-center"
                        key={preset.preset_id}
                      >
                        <div className="space-y-2">
                          <p className="font-medium text-[var(--color-text-primary)]">
                            {preset.display_name}
                          </p>
                          <p className="text-xs text-[var(--color-text-muted)]">{preset.preset_id}</p>
                        </div>

                        <div className="flex flex-wrap gap-2">
                          {agentRoleKeys.slice(0, 3).map((roleKey) => (
                            <Badge key={roleKey} variant="subtle">
                              {roleLabels[roleKey]} ·{' '}
                              {describePresetAgent(preset, roleKey, t('presetsPage.list.unset'))}
                            </Badge>
                          ))}
                        </div>

                        <div className="flex justify-start gap-2 lg:justify-end">
                          <IconButton
                            icon={<FontAwesomeIcon icon={faEye} />}
                            label={t('presetsPage.actions.view')}
                            onClick={() => {
                              setDetailsPresetId(preset.preset_id)
                            }}
                            size="sm"
                            variant="ghost"
                          />
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
                              setDeleteTarget(preset)
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
