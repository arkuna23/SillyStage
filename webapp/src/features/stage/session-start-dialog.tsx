import { useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { appPaths } from '../../app/paths'
import { Button } from '../../components/ui/button'
import {
  Dialog,
  DialogBody,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../../components/ui/dialog'
import { DialogRouteButton } from '../../components/ui/dialog-route-button'
import { Input } from '../../components/ui/input'
import { Select } from '../../components/ui/select'
import { useToastMessage } from '../../components/ui/toast-context'
import type { ApiConfig, ApiGroup, Preset } from '../apis/types'
import type { PlayerProfile } from '../player-profiles/types'
import type { StorySummary } from '../stories/types'
import { startSession } from './api'
import { getStageCopy } from './copy'
import type { StartedSession, StartSessionInput } from './types'

const NO_PLAYER_PROFILE_OPTION_VALUE = '__none__'

type SessionStartDialogProps = {
  apis: ReadonlyArray<ApiConfig>
  apiGroups: ReadonlyArray<ApiGroup>
  onCompleted: (result: { message: string; session: StartedSession }) => Promise<void> | void
  onOpenChange: (open: boolean) => void
  open: boolean
  playerProfiles: ReadonlyArray<PlayerProfile>
  presets: ReadonlyArray<Preset>
  stories: ReadonlyArray<StorySummary>
}

type FormState = {
  apiGroupId: string
  displayName: string
  playerProfileId: string
  presetId: string
  storyId: string
}

function createInitialState(
  apiGroups: ReadonlyArray<ApiGroup>,
  presets: ReadonlyArray<Preset>,
  stories: ReadonlyArray<StorySummary>,
): FormState {
  return {
    apiGroupId: apiGroups[0]?.api_group_id ?? '',
    displayName: '',
    playerProfileId: NO_PLAYER_PROFILE_OPTION_VALUE,
    presetId: presets[0]?.preset_id ?? '',
    storyId: stories[0]?.story_id ?? '',
  }
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

export function SessionStartDialog({
  apis,
  apiGroups,
  onCompleted,
  onOpenChange,
  open,
  playerProfiles,
  presets,
  stories,
}: SessionStartDialogProps) {
  const { i18n } = useTranslation()
  const copy = getStageCopy(i18n.language)
  const [formState, setFormState] = useState<FormState>(() =>
    createInitialState(apiGroups, presets, stories),
  )
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)
  useToastMessage(submitError)

  const storyOptions = useMemo(
    () =>
      stories.map((story) => ({
        label: story.display_name,
        value: story.story_id,
      })),
    [stories],
  )

  const playerProfileOptions = useMemo(
    () => [
      { label: copy.settings.playerProfile.noProfile, value: NO_PLAYER_PROFILE_OPTION_VALUE },
      ...playerProfiles.map((profile) => ({
        label: profile.display_name,
        value: profile.player_profile_id,
      })),
    ],
    [copy.settings.playerProfile.noProfile, playerProfiles],
  )

  const apiGroupOptions = useMemo(
    () =>
      apiGroups.map((apiGroup) => ({
        label: apiGroup.display_name,
        value: apiGroup.api_group_id,
      })),
    [apiGroups],
  )

  const presetOptions = useMemo(
    () =>
      presets.map((preset) => ({
        label: preset.display_name,
        value: preset.preset_id,
      })),
    [presets],
  )

  const selectedStory = useMemo(
    () => stories.find((story) => story.story_id === formState.storyId) ?? null,
    [formState.storyId, stories],
  )

  useEffect(() => {
    if (!open) {
      setFormState(createInitialState(apiGroups, presets, stories))
      setSubmitError(null)
      setIsSubmitting(false)
      return
    }

    setFormState((current) => ({
      apiGroupId: current.apiGroupId || apiGroups[0]?.api_group_id || '',
      displayName: current.displayName,
      playerProfileId: current.playerProfileId || NO_PLAYER_PROFILE_OPTION_VALUE,
      presetId: current.presetId || presets[0]?.preset_id || '',
      storyId: current.storyId || stories[0]?.story_id || '',
    }))
    setSubmitError(null)
  }, [apiGroups, open, presets, stories])

  async function handleSubmit() {
    if (!formState.storyId.trim()) {
      setSubmitError(copy.createSession.emptyStories)
      return
    }

    if (!formState.apiGroupId.trim()) {
      setSubmitError(copy.createSession.emptyApiGroups)
      return
    }

    if (!formState.presetId.trim()) {
      setSubmitError(copy.createSession.emptyPresets)
      return
    }

    setSubmitError(null)
    setIsSubmitting(true)

    try {
      const params: StartSessionInput = {
        api_group_id: formState.apiGroupId.trim(),
        ...(formState.displayName.trim() ? { display_name: formState.displayName.trim() } : {}),
        ...(formState.playerProfileId !== NO_PLAYER_PROFILE_OPTION_VALUE
          ? { player_profile_id: formState.playerProfileId }
          : {}),
        preset_id: formState.presetId.trim(),
        story_id: formState.storyId.trim(),
      }

      const session = await startSession(params)

      await onCompleted({
        message: copy.notice.created,
        session,
      })

      onOpenChange(false)
    } catch (error) {
      setSubmitError(getErrorMessage(error, copy.notice.createFailed))
    } finally {
      setIsSubmitting(false)
    }
  }

  const hasStories = stories.length > 0
  const hasApis = apis.length > 0
  const hasApiGroups = apiGroups.length > 0
  const hasPresets = presets.length > 0

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent
        aria-describedby={undefined}
        className="w-[min(92vw,46rem)]"
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
          <DialogTitle>{copy.createSession.title}</DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-5 pt-6">
          {!hasStories ? (
            <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4 text-sm leading-7 text-[var(--color-text-secondary)]">
              {copy.createSession.emptyStories}
            </div>
          ) : null}

          {!hasApis ? (
            <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4 text-sm leading-7 text-[var(--color-text-secondary)]">
              <p>{copy.createSession.emptyApis}</p>
              <div className="mt-4">
                <DialogRouteButton
                  onRequestClose={() => onOpenChange(false)}
                  to={appPaths.apis}
                  variant="secondary"
                >
                  {copy.createSession.configureApis}
                </DialogRouteButton>
              </div>
            </div>
          ) : null}

          {hasApis && !hasApiGroups ? (
            <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4 text-sm leading-7 text-[var(--color-text-secondary)]">
              <p>{copy.createSession.emptyApiGroups}</p>
              <div className="mt-4">
                <DialogRouteButton
                  onRequestClose={() => onOpenChange(false)}
                  to={appPaths.apis}
                  variant="secondary"
                >
                  {copy.createSession.configureApiGroups}
                </DialogRouteButton>
              </div>
            </div>
          ) : null}

          {hasApis && hasApiGroups && !hasPresets ? (
            <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4 text-sm leading-7 text-[var(--color-text-secondary)]">
              <p>{copy.createSession.emptyPresets}</p>
              <div className="mt-4">
                <DialogRouteButton
                  onRequestClose={() => onOpenChange(false)}
                  to={appPaths.presets}
                  variant="secondary"
                >
                  {copy.createSession.configurePresets}
                </DialogRouteButton>
              </div>
            </div>
          ) : null}

          {hasStories && hasApis && hasApiGroups && hasPresets ? (
            <>
              <div className="space-y-5">
                <label className="space-y-2.5">
                  <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                    {copy.createSession.story}
                  </span>
                  <Select
                    items={storyOptions}
                    textAlign="start"
                    value={formState.storyId}
                    onValueChange={(storyId) => {
                      setFormState((current) => ({ ...current, storyId }))
                    }}
                  />
                </label>

                {selectedStory ? (
                  <div className="mt-2 rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
                    <p className="text-xs text-[var(--color-text-muted)]">{copy.intro.section}</p>
                    <p className="mt-2 text-sm leading-7 text-[var(--color-text-primary)]">
                      {selectedStory.introduction}
                    </p>
                  </div>
                ) : null}
              </div>

              <div className="grid gap-4 md:grid-cols-2">
                <label className="space-y-2.5">
                  <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                    {copy.createSession.displayName}
                  </span>
                  <Input
                    id="stage-session-display-name"
                    name="stage-session-display-name"
                    onChange={(event) => {
                      setFormState((current) => ({
                        ...current,
                        displayName: event.target.value,
                      }))
                    }}
                    placeholder={copy.createSession.displayName}
                    value={formState.displayName}
                  />
                </label>

                <label className="space-y-2.5">
                  <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                    {copy.createSession.playerProfile}
                  </span>
                  <Select
                    items={playerProfileOptions}
                    textAlign="start"
                    value={formState.playerProfileId}
                    onValueChange={(playerProfileId) => {
                      setFormState((current) => ({ ...current, playerProfileId }))
                    }}
                  />
                </label>
              </div>

              <div className="grid gap-4 md:grid-cols-2">
                <label className="space-y-2.5">
                  <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                    {copy.createSession.apiGroup}
                  </span>
                  <Select
                    items={apiGroupOptions}
                    placeholder={copy.createSession.apiGroupPlaceholder}
                    textAlign="start"
                    value={formState.apiGroupId}
                    onValueChange={(apiGroupId) => {
                      setFormState((current) => ({ ...current, apiGroupId }))
                    }}
                  />
                </label>

                <label className="space-y-2.5">
                  <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                    {copy.createSession.preset}
                  </span>
                  <Select
                    items={presetOptions}
                    placeholder={copy.createSession.presetPlaceholder}
                    textAlign="start"
                    value={formState.presetId}
                    onValueChange={(presetId) => {
                      setFormState((current) => ({ ...current, presetId }))
                    }}
                  />
                </label>
              </div>

              {playerProfiles.length === 0 ? (
                <div className="rounded-[1.35rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_72%,transparent)] px-4 py-4 text-sm leading-7 text-[var(--color-text-secondary)]">
                  {copy.createSession.emptyPlayerProfiles}
                </div>
              ) : null}
            </>
          ) : null}
        </DialogBody>

        <DialogFooter className="justify-end">
          <Button
            disabled={isSubmitting}
            onClick={() => {
              onOpenChange(false)
            }}
            variant="secondary"
          >
            {copy.createSession.cancel}
          </Button>
          <Button
            disabled={isSubmitting || !hasStories || !hasApiGroups || !hasPresets}
            onClick={() => void handleSubmit()}
          >
            {isSubmitting ? copy.createSession.creating : copy.createSession.create}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
