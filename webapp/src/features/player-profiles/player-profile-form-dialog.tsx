import { useEffect, useId, useState } from 'react'
import { useTranslation } from 'react-i18next'

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
import { Input } from '../../components/ui/input'
import { Textarea } from '../../components/ui/textarea'
import { useToastMessage } from '../../components/ui/toast-context'
import { createPlayerProfile, getPlayerProfile, updatePlayerProfile } from './api'
import type { PlayerProfile } from './types'

type PlayerProfileFormDialogMode = 'create' | 'edit'

type PlayerProfileFormDialogProps = {
  existingProfileIds: ReadonlyArray<string>
  mode: PlayerProfileFormDialogMode
  onCompleted: (result: { message: string; profile: PlayerProfile }) => Promise<void> | void
  onOpenChange: (open: boolean) => void
  open: boolean
  playerProfileId?: string | null
}

type FormState = {
  description: string
  displayName: string
  playerProfileId: string
}

function createInitialFormState(): FormState {
  return {
    description: '',
    displayName: '',
    playerProfileId: '',
  }
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function LoadingSkeleton() {
  return (
    <div className="space-y-4">
      <div className="grid gap-4 md:grid-cols-2">
        <div className="h-12 animate-pulse rounded-2xl bg-[var(--color-bg-elevated)]" />
        <div className="h-12 animate-pulse rounded-2xl bg-[var(--color-bg-elevated)]" />
      </div>
      <div className="h-32 animate-pulse rounded-[1.45rem] bg-[var(--color-bg-elevated)]" />
    </div>
  )
}

export function PlayerProfileFormDialog({
  existingProfileIds,
  mode,
  onCompleted,
  onOpenChange,
  open,
  playerProfileId,
}: PlayerProfileFormDialogProps) {
  const { t } = useTranslation()
  const fieldIdPrefix = useId()
  const [formState, setFormState] = useState<FormState>(createInitialFormState)
  const [initialProfile, setInitialProfile] = useState<PlayerProfile | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)
  useToastMessage(submitError)

  const fieldIds = {
    description: `${fieldIdPrefix}-description`,
    displayName: `${fieldIdPrefix}-display-name`,
    playerProfileId: `${fieldIdPrefix}-player-profile-id`,
  } as const

  const isEditMode = mode === 'edit'

  useEffect(() => {
    if (!open) {
      setFormState(createInitialFormState())
      setInitialProfile(null)
      setIsLoading(false)
      setIsSubmitting(false)
      setSubmitError(null)
      return
    }

    if (mode === 'create') {
      setFormState(createInitialFormState())
      setInitialProfile(null)
      setIsLoading(false)
      setIsSubmitting(false)
      setSubmitError(null)
      return
    }

    if (!playerProfileId) {
      return
    }

    const controller = new AbortController()
    setIsLoading(true)
    setIsSubmitting(false)
    setSubmitError(null)

    void getPlayerProfile(playerProfileId, controller.signal)
      .then((profile) => {
        if (controller.signal.aborted) {
          return
        }

        setInitialProfile(profile)
        setFormState({
          description: profile.description,
          displayName: profile.display_name,
          playerProfileId: profile.player_profile_id,
        })
      })
      .catch((error) => {
        if (!controller.signal.aborted) {
          setSubmitError(getErrorMessage(error, t('playerProfiles.feedback.loadFailed')))
        }
      })
      .finally(() => {
        if (!controller.signal.aborted) {
          setIsLoading(false)
        }
      })

    return () => {
      controller.abort()
    }
  }, [mode, open, playerProfileId, t])

  function validateForm() {
    const nextProfileId = formState.playerProfileId.trim()

    if (nextProfileId.length === 0) {
      return t('playerProfiles.form.errors.playerProfileIdRequired')
    }

    if (
      mode === 'create' &&
      existingProfileIds.some((existingId) => existingId === nextProfileId)
    ) {
      return t('playerProfiles.form.errors.playerProfileIdDuplicate')
    }

    if (formState.displayName.trim().length === 0) {
      return t('playerProfiles.form.errors.displayNameRequired')
    }

    if (formState.description.trim().length === 0) {
      return t('playerProfiles.form.errors.descriptionRequired')
    }

    return null
  }

  async function handleSubmit() {
    const validationError = validateForm()

    setSubmitError(null)

    if (validationError) {
      setSubmitError(validationError)
      return
    }

    if (isEditMode && !initialProfile) {
      setSubmitError(t('playerProfiles.feedback.loadFailed'))
      return
    }

    const nextPlayerProfileId = formState.playerProfileId.trim()
    const nextDisplayName = formState.displayName.trim()
    const nextDescription = formState.description.trim()

    setIsSubmitting(true)

    try {
      const result =
        mode === 'create'
          ? await createPlayerProfile({
              description: nextDescription,
              display_name: nextDisplayName,
              player_profile_id: nextPlayerProfileId,
            })
          : await updatePlayerProfile({
              description: nextDescription,
              display_name: nextDisplayName,
              player_profile_id: nextPlayerProfileId,
            })

      await onCompleted({
        message:
          mode === 'create'
            ? t('playerProfiles.feedback.created', { name: result.display_name })
            : t('playerProfiles.feedback.updated', { name: result.display_name }),
        profile: result,
      })

      onOpenChange(false)
    } catch (error) {
      setSubmitError(getErrorMessage(error, t('playerProfiles.form.errors.submitFailed')))
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent
        aria-describedby={undefined}
        className="w-[min(92vw,42rem)]"
        onEscapeKeyDown={(event) => {
          if (isSubmitting) {
            event.preventDefault()
          }
        }}
        onInteractOutside={(event) => {
          if (isSubmitting) {
            event.preventDefault()
          }
        }}
      >
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>
            {isEditMode
              ? t('playerProfiles.form.editTitle')
              : t('playerProfiles.form.createTitle')}
          </DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-5 pt-6">
          {isLoading ? (
            <LoadingSkeleton />
          ) : (
            <div className="grid gap-5">
              <div className="grid gap-4 md:grid-cols-2">
                <div className="space-y-2.5">
                  <label
                    className="block text-sm font-medium text-[var(--color-text-primary)]"
                    htmlFor={fieldIds.playerProfileId}
                  >
                    {t('playerProfiles.form.fields.playerProfileId')}
                  </label>
                  <Input
                    autoFocus={!isEditMode}
                    id={fieldIds.playerProfileId}
                    onChange={(event) => {
                      setFormState((current) => ({
                        ...current,
                        playerProfileId: event.target.value,
                      }))
                    }}
                    placeholder={t('playerProfiles.form.placeholders.playerProfileId')}
                    readOnly={isEditMode}
                    value={formState.playerProfileId}
                  />
                  {isEditMode ? (
                    <p className="text-xs leading-6 text-[var(--color-text-muted)]">
                      {t('playerProfiles.form.fields.playerProfileIdHint')}
                    </p>
                  ) : null}
                </div>

                <div className="space-y-2.5">
                  <label
                    className="block text-sm font-medium text-[var(--color-text-primary)]"
                    htmlFor={fieldIds.displayName}
                  >
                    {t('playerProfiles.form.fields.displayName')}
                  </label>
                  <Input
                    autoFocus={isEditMode}
                    id={fieldIds.displayName}
                    onChange={(event) => {
                      setFormState((current) => ({
                        ...current,
                        displayName: event.target.value,
                      }))
                    }}
                    placeholder={t('playerProfiles.form.placeholders.displayName')}
                    value={formState.displayName}
                  />
                </div>
              </div>

              <div className="space-y-2.5">
                <label
                  className="block text-sm font-medium text-[var(--color-text-primary)]"
                  htmlFor={fieldIds.description}
                >
                  {t('playerProfiles.form.fields.description')}
                </label>
                <Textarea
                  className="min-h-36"
                  id={fieldIds.description}
                  onChange={(event) => {
                    setFormState((current) => ({
                      ...current,
                      description: event.target.value,
                    }))
                  }}
                  placeholder={t('playerProfiles.form.placeholders.description')}
                  value={formState.description}
                />
              </div>
            </div>
          )}
        </DialogBody>

        <DialogFooter>
          <DialogClose asChild>
            <Button disabled={isSubmitting} variant="ghost">
              {t('playerProfiles.actions.cancel')}
            </Button>
          </DialogClose>

          <Button disabled={isLoading || isSubmitting} onClick={() => void handleSubmit()}>
            {isSubmitting
              ? t('playerProfiles.actions.saving')
              : isEditMode
                ? t('playerProfiles.actions.saveChanges')
                : t('playerProfiles.actions.create')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
