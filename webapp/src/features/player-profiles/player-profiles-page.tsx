import { useCallback, useEffect, useLayoutEffect, useMemo, useState } from 'react'
import { faCheckDouble } from '@fortawesome/free-solid-svg-icons/faCheckDouble'
import { faPlus } from '@fortawesome/free-solid-svg-icons/faPlus'
import { faSquareCheck } from '@fortawesome/free-solid-svg-icons/faSquareCheck'
import { faWandMagicSparkles } from '@fortawesome/free-solid-svg-icons/faWandMagicSparkles'
import { faTrashCan } from '@fortawesome/free-solid-svg-icons/faTrashCan'
import { faXmark } from '@fortawesome/free-solid-svg-icons/faXmark'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { useTranslation } from 'react-i18next'

import { demoPlayerProfile } from '../demo-content/konosuba-sample-data'
import { InsertSampleDialog } from '../demo-content/insert-sample-dialog'
import { WorkspacePanelShell } from '../../components/layout/workspace-panel-shell'
import { useWorkspaceLayoutContext } from '../../components/layout/workspace-context'
import { Badge } from '../../components/ui/badge'
import { Button } from '../../components/ui/button'
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '../../components/ui/card'
import { IconButton } from '../../components/ui/icon-button'
import { SelectionToggleButton } from '../../components/ui/selection-toggle-button'
import { SectionHeader } from '../../components/ui/section-header'
import { useToastNotice } from '../../components/ui/toast-context'
import { runBatchDelete } from '../../lib/batch-delete'
import { createPlayerProfile, deletePlayerProfile, listPlayerProfiles } from './api'
import { DeletePlayerProfileDialog } from './delete-player-profile-dialog'
import { PlayerProfileFormDialog } from './player-profile-form-dialog'
import type { PlayerProfile } from './types'

type NoticeTone = 'error' | 'success' | 'warning'

type Notice = {
  message: string
  tone: NoticeTone
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function PlayerProfilesListSkeleton() {
  return (
    <div className="space-y-5">
      <div className="h-8 w-48 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
      <div className="divide-y divide-[var(--color-border-subtle)]">
        {Array.from({ length: 4 }).map((_, index) => (
          <div
            className="grid gap-4 py-4 lg:grid-cols-[minmax(0,0.9fr)_minmax(0,1.2fr)_auto] lg:items-center"
            key={index}
          >
            <div className="space-y-2.5">
              <div className="h-6 w-36 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
              <div className="h-3 w-40 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            </div>
            <div className="space-y-2">
              <div className="h-3 w-20 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
              <div className="h-3 w-full animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            </div>
            <div className="flex justify-end gap-2">
              <div className="h-9 w-20 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
              <div className="h-9 w-20 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
            </div>
          </div>
        ))}
      </div>
    </div>
  )
}

function summarizeDescription(description: string) {
  return description.replace(/\s+/g, ' ').trim()
}

export function PlayerProfilesPage() {
  const { t } = useTranslation()
  const { setRailContent } = useWorkspaceLayoutContext()
  const [profiles, setProfiles] = useState<PlayerProfile[]>([])
  const [notice, setNotice] = useState<Notice | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [isDeleting, setIsDeleting] = useState(false)
  const [isCreatingSample, setIsCreatingSample] = useState(false)
  const [isSampleDialogOpen, setIsSampleDialogOpen] = useState(false)
  const [isCreateDialogOpen, setIsCreateDialogOpen] = useState(false)
  const [editProfileId, setEditProfileId] = useState<string | null>(null)
  const [selectionMode, setSelectionMode] = useState(false)
  const [selectedProfileIds, setSelectedProfileIds] = useState<string[]>([])
  const [deleteTargetIds, setDeleteTargetIds] = useState<string[]>([])
  useToastNotice(notice)

  const existingProfileIds = useMemo(
    () => profiles.map((profile) => profile.player_profile_id),
    [profiles],
  )
  const deleteTargets = useMemo(
    () =>
      deleteTargetIds
        .map((profileId) => profiles.find((profile) => profile.player_profile_id === profileId))
        .filter((profile): profile is PlayerProfile => profile !== undefined),
    [deleteTargetIds, profiles],
  )

  const refreshProfiles = useCallback(
    async (signal?: AbortSignal) => {
      setIsLoading(true)

      try {
        const nextProfiles = await listPlayerProfiles(signal)

        if (!signal?.aborted) {
          setProfiles(nextProfiles)
        }
      } catch (error) {
        if (!signal?.aborted) {
          setNotice({
            message: getErrorMessage(error, t('playerProfiles.feedback.loadListFailed')),
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

    void refreshProfiles(controller.signal)

    return () => {
      controller.abort()
    }
  }, [refreshProfiles])

  useEffect(() => {
    const availableProfileIds = new Set(profiles.map((profile) => profile.player_profile_id))

    setSelectedProfileIds((currentSelection) =>
      currentSelection.filter((profileId) => availableProfileIds.has(profileId)),
    )
    setDeleteTargetIds((currentSelection) =>
      currentSelection.filter((profileId) => availableProfileIds.has(profileId)),
    )

    if (editProfileId !== null && !availableProfileIds.has(editProfileId)) {
      setEditProfileId(null)
    }
  }, [editProfileId, profiles])

  useLayoutEffect(() => {
    setRailContent({
      description: t('playerProfiles.rail.description'),
      stats: [
        {
          label: t('playerProfiles.metrics.total'),
          value: profiles.length,
        },
      ],
      title: t('playerProfiles.title'),
    })

    return () => {
      setRailContent(null)
    }
  }, [profiles.length, setRailContent, t])

  async function handleDeleteProfile() {
    if (deleteTargets.length === 0) {
      return
    }

    setIsDeleting(true)

    try {
      const result = await runBatchDelete(deleteTargets, async (target) => {
        await deletePlayerProfile(target.player_profile_id)
      })

      setDeleteTargetIds([])

      if (result.deleted.length > 0) {
        const deletedIds = new Set(result.deleted.map((target) => target.player_profile_id))
        setSelectedProfileIds((currentSelection) =>
          currentSelection.filter((profileId) => !deletedIds.has(profileId)),
        )
        setDeleteTargetIds([])

        if (editProfileId !== null && deletedIds.has(editProfileId)) {
          setEditProfileId(null)
        }
      }

      if (result.failed.length === 0) {
        setNotice({
          message:
            result.deleted.length > 1
              ? t('playerProfiles.feedback.deletedMany', { count: result.deleted.length })
              : t('playerProfiles.feedback.deleted', { name: result.deleted[0]?.display_name ?? '' }),
          tone: 'success',
        })
        if (selectionMode) {
          setSelectionMode(false)
          setSelectedProfileIds([])
        }
      } else if (result.deleted.length > 0) {
        setNotice({
          message: t('playerProfiles.feedback.deletedPartial', {
            failed: result.failed.length,
            success: result.deleted.length,
          }),
          tone: 'warning',
        })
      } else {
        setNotice({
          message: t('playerProfiles.feedback.deleteFailed'),
          tone: 'error',
        })
      }

      await refreshProfiles()
    } finally {
      setIsDeleting(false)
    }
  }

  function toggleProfileSelection(playerProfileId: string) {
    setSelectedProfileIds((currentSelection) =>
      currentSelection.includes(playerProfileId)
        ? currentSelection.filter((currentProfileId) => currentProfileId !== playerProfileId)
        : [...currentSelection, playerProfileId],
    )
  }

  async function handleCreateSampleProfile() {
    if (existingProfileIds.includes(demoPlayerProfile.playerProfileId)) {
      setNotice({
        message: t('playerProfiles.feedback.sampleExists'),
        tone: 'warning',
      })
      return
    }

    setIsCreatingSample(true)

    try {
      const profile = await createPlayerProfile({
        description: demoPlayerProfile.description,
        display_name: demoPlayerProfile.displayName,
        player_profile_id: demoPlayerProfile.playerProfileId,
      })

      setNotice({
        message: t('playerProfiles.feedback.sampleCreated', { name: profile.display_name }),
        tone: 'success',
      })
      await refreshProfiles()
    } catch (error) {
      setNotice({
        message: getErrorMessage(error, t('playerProfiles.feedback.sampleCreateFailed')),
        tone: 'error',
      })
    } finally {
      setIsCreatingSample(false)
    }
  }

  return (
    <div className="flex h-full min-h-0 flex-col gap-6">
      <PlayerProfileFormDialog
        existingProfileIds={existingProfileIds}
        mode="create"
        onCompleted={async ({ message }) => {
          setNotice({ message, tone: 'success' })
          await refreshProfiles()
        }}
        onOpenChange={setIsCreateDialogOpen}
        open={isCreateDialogOpen}
      />

      <PlayerProfileFormDialog
        existingProfileIds={existingProfileIds}
        mode="edit"
        onCompleted={async ({ message }) => {
          setNotice({ message, tone: 'success' })
          await refreshProfiles()
        }}
        onOpenChange={(open) => {
          if (!open) {
            setEditProfileId(null)
          }
        }}
        open={editProfileId !== null}
        playerProfileId={editProfileId}
      />

      <DeletePlayerProfileDialog
        deleting={isDeleting}
        onConfirm={() => {
          void handleDeleteProfile()
        }}
        onOpenChange={() => {
          setDeleteTargetIds([])
        }}
        targets={deleteTargets}
      />

      <InsertSampleDialog
        cancelLabel={t('playerProfiles.actions.cancel')}
        confirmLabel={t('playerProfiles.sampleDialog.confirm')}
        confirmDisabled={existingProfileIds.includes(demoPlayerProfile.playerProfileId)}
        description={t('playerProfiles.sampleDialog.description')}
        existingLabel={t('playerProfiles.sampleDialog.existing')}
        items={[
          {
            description: demoPlayerProfile.playerProfileId,
            label: demoPlayerProfile.displayName,
            status: existingProfileIds.includes(demoPlayerProfile.playerProfileId)
              ? 'existing'
              : 'new',
          },
        ]}
        newLabel={t('playerProfiles.sampleDialog.new')}
        onConfirm={() => {
          void handleCreateSampleProfile()
          setIsSampleDialogOpen(false)
        }}
        onOpenChange={setIsSampleDialogOpen}
        open={isSampleDialogOpen}
        pending={isCreatingSample}
        pendingLabel={t('playerProfiles.actions.creatingSample')}
        title={t('playerProfiles.sampleDialog.title')}
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
                      {t('playerProfiles.selection.count', { count: selectedProfileIds.length })}
                    </Badge>
                    <IconButton
                      disabled={profiles.length === 0}
                      icon={<FontAwesomeIcon icon={faCheckDouble} />}
                      label={t('playerProfiles.actions.selectAll')}
                      onClick={() => {
                        setSelectedProfileIds(profiles.map((profile) => profile.player_profile_id))
                      }}
                      size="md"
                      variant="secondary"
                    />
                    <IconButton
                      disabled={selectedProfileIds.length === 0}
                      icon={<FontAwesomeIcon icon={faTrashCan} />}
                      label={t('playerProfiles.actions.deleteSelected')}
                      onClick={() => {
                        setDeleteTargetIds(selectedProfileIds)
                      }}
                      size="md"
                      variant="danger"
                    />
                    <IconButton
                      icon={<FontAwesomeIcon icon={faXmark} />}
                      label={t('playerProfiles.actions.cancelSelection')}
                      onClick={() => {
                        setSelectionMode(false)
                        setSelectedProfileIds([])
                      }}
                      size="md"
                      variant="secondary"
                    />
                  </>
                ) : (
                  <>
                    <IconButton
                      icon={<FontAwesomeIcon icon={faSquareCheck} />}
                      label={t('playerProfiles.actions.selectMode')}
                      onClick={() => {
                        setSelectionMode(true)
                        setSelectedProfileIds([])
                      }}
                      size="md"
                      variant="secondary"
                    />
                    <IconButton
                      disabled={isCreatingSample}
                      icon={<FontAwesomeIcon icon={faWandMagicSparkles} />}
                      label={
                        isCreatingSample
                          ? t('playerProfiles.actions.creatingSample')
                          : t('playerProfiles.actions.createSample')
                      }
                      onClick={() => {
                        setIsSampleDialogOpen(true)
                      }}
                      size="md"
                      variant="secondary"
                    />
                    <IconButton
                      icon={<FontAwesomeIcon icon={faPlus} />}
                      label={t('playerProfiles.actions.create')}
                      onClick={() => {
                        setIsCreateDialogOpen(true)
                      }}
                      size="md"
                    />
                  </>
                )}
              </div>
            }
            title={t('playerProfiles.title')}
          />
        </CardHeader>

        <CardContent className="min-h-0 flex-1 overflow-y-auto pt-6">
          <div className="space-y-6 pr-1">
            <section className="space-y-5">
              <div className="space-y-2">
                <CardTitle className="text-[1.85rem]">{t('playerProfiles.list.title')}</CardTitle>
                <CardDescription>{t('playerProfiles.list.description')}</CardDescription>
              </div>

              {isLoading ? (
                <PlayerProfilesListSkeleton />
              ) : profiles.length === 0 ? (
                <div className="py-12 text-center">
                  <h3 className="font-display text-3xl text-[var(--color-text-primary)]">
                    {t('playerProfiles.empty.title')}
                  </h3>

                  <p className="mt-3 text-sm leading-7 text-[var(--color-text-secondary)]">
                    {t('playerProfiles.empty.description')}
                  </p>

                  <div className="mt-7 flex justify-center">
                    <Button
                      onClick={() => {
                        setIsCreateDialogOpen(true)
                      }}
                    >
                      {t('playerProfiles.actions.create')}
                    </Button>
                  </div>
                </div>
              ) : (
                <div className="divide-y divide-[var(--color-border-subtle)]">
                  {profiles.map((profile) => (
                    <div
                      className="grid gap-4 py-4 lg:grid-cols-[minmax(0,0.9fr)_minmax(0,1.2fr)_auto] lg:items-center"
                      key={profile.player_profile_id}
                    >
                      <div className="min-w-0 space-y-2">
                        <h3 className="truncate font-display text-[1.32rem] leading-tight text-[var(--color-text-primary)]">
                          {profile.display_name}
                        </h3>
                        <p className="truncate font-mono text-[0.76rem] leading-5 text-[var(--color-text-muted)]">
                          {profile.player_profile_id}
                        </p>
                      </div>

                      <div className="min-w-0 space-y-1.5">
                        <p className="text-xs text-[var(--color-text-muted)]">
                          {t('playerProfiles.list.descriptionLabel')}
                        </p>
                        <p className="line-clamp-2 text-sm leading-7 text-[var(--color-text-primary)]">
                          {summarizeDescription(profile.description)}
                        </p>
                      </div>

                      <div className="flex flex-wrap items-center justify-start gap-2 lg:justify-end">
                        {selectionMode ? (
                          <SelectionToggleButton
                            label={
                              selectedProfileIds.includes(profile.player_profile_id)
                                ? t('playerProfiles.actions.deselect')
                                : t('playerProfiles.actions.select')
                            }
                            onClick={() => {
                              toggleProfileSelection(profile.player_profile_id)
                            }}
                            selected={selectedProfileIds.includes(profile.player_profile_id)}
                          />
                        ) : (
                          <>
                            <Button
                              onClick={() => {
                                setEditProfileId(profile.player_profile_id)
                              }}
                              size="sm"
                              variant="secondary"
                            >
                              {t('playerProfiles.actions.edit')}
                            </Button>
                            <Button
                              onClick={() => {
                                setDeleteTargetIds([profile.player_profile_id])
                              }}
                              size="sm"
                              variant="danger"
                            >
                              {t('playerProfiles.actions.delete')}
                            </Button>
                          </>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </section>
          </div>
        </CardContent>
        </Card>
      </WorkspacePanelShell>
    </div>
  )
}
