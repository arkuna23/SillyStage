import { useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { cn } from '../../lib/cn'
import { Button } from '../../components/ui/button'
import {
  Dialog,
  DialogBody,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../../components/ui/dialog'
import { Input } from '../../components/ui/input'
import { Select } from '../../components/ui/select'
import { agentApiRoleKeys, type AgentApiIds, type LlmApi } from '../apis/types'
import type { PlayerProfile } from '../player-profiles/types'
import type { StorySummary } from '../stories/types'
import { getStageCopy } from './copy'
import { startSession } from './api'
import type { SessionConfigMode, StartedSession, StartSessionInput } from './types'

const NO_PLAYER_PROFILE_OPTION_VALUE = '__none__'

type SessionStartDialogProps = {
  apis: ReadonlyArray<LlmApi>
  onCompleted: (result: { message: string; session: StartedSession }) => Promise<void> | void
  onOpenChange: (open: boolean) => void
  open: boolean
  playerProfiles: ReadonlyArray<PlayerProfile>
  stories: ReadonlyArray<StorySummary>
}

type FormState = {
  configMode: SessionConfigMode
  displayName: string
  playerProfileId: string
  sessionApiIds: AgentApiIds | null
  storyId: string
}

function buildDefaultSessionApiIds(apis: ReadonlyArray<LlmApi>) {
  const fallbackApiId = apis[0]?.api_id

  if (!fallbackApiId) {
    return null
  }

  return {
    actor_api_id: fallbackApiId,
    architect_api_id: fallbackApiId,
    director_api_id: fallbackApiId,
    keeper_api_id: fallbackApiId,
    narrator_api_id: fallbackApiId,
    planner_api_id: fallbackApiId,
    replyer_api_id: fallbackApiId,
  } satisfies AgentApiIds
}

function createInitialState(apis: ReadonlyArray<LlmApi>, stories: ReadonlyArray<StorySummary>): FormState {
  return {
    configMode: 'use_global',
    displayName: '',
    playerProfileId: NO_PLAYER_PROFILE_OPTION_VALUE,
    sessionApiIds: buildDefaultSessionApiIds(apis),
    storyId: stories[0]?.story_id ?? '',
  }
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

export function SessionStartDialog({
  apis,
  onCompleted,
  onOpenChange,
  open,
  playerProfiles,
  stories,
}: SessionStartDialogProps) {
  const { i18n } = useTranslation()
  const copy = getStageCopy(i18n.language)
  const [formState, setFormState] = useState<FormState>(() => createInitialState(apis, stories))
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)

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
      { label: '—', value: NO_PLAYER_PROFILE_OPTION_VALUE },
      ...playerProfiles.map((profile) => ({
        label: profile.display_name,
        value: profile.player_profile_id,
      })),
    ],
    [playerProfiles],
  )

  const apiOptions = useMemo(
    () =>
      apis.map((api) => ({
        label: `${api.api_id} · ${api.model}`,
        value: api.api_id,
      })),
    [apis],
  )

  const selectedStory = useMemo(
    () => stories.find((story) => story.story_id === formState.storyId) ?? null,
    [formState.storyId, stories],
  )

  useEffect(() => {
    if (!open) {
      setFormState(createInitialState(apis, stories))
      setSubmitError(null)
      setIsSubmitting(false)
      return
    }

    setFormState((current) => ({
      ...createInitialState(apis, stories),
      storyId: current.storyId || stories[0]?.story_id || '',
    }))
  }, [apis, open, stories])

  function updateApiRole(roleKey: keyof AgentApiIds, apiId: string) {
    setFormState((current) => ({
      ...current,
      sessionApiIds: current.sessionApiIds
        ? {
            ...current.sessionApiIds,
            [roleKey]: apiId,
          }
        : current.sessionApiIds,
    }))
  }

  async function handleSubmit() {
    if (!formState.storyId.trim()) {
      setSubmitError(copy.createSession.emptyStories)
      return
    }

    if (formState.configMode === 'use_session') {
      const missingRole = agentApiRoleKeys.find(
        (roleKey) => !formState.sessionApiIds?.[roleKey]?.trim(),
      )

      if (missingRole) {
        setSubmitError(`${copy.apiPanel.roles[missingRole]}: ${copy.createSession.emptyApis}`)
        return
      }
    }

    setSubmitError(null)
    setIsSubmitting(true)

    try {
      const params: StartSessionInput = {
        config_mode: formState.configMode,
        ...(formState.displayName.trim() ? { display_name: formState.displayName.trim() } : {}),
        ...(formState.playerProfileId !== NO_PLAYER_PROFILE_OPTION_VALUE
          ? { player_profile_id: formState.playerProfileId }
          : {}),
        ...(formState.configMode === 'use_session' && formState.sessionApiIds
          ? { session_api_ids: formState.sessionApiIds }
          : {}),
        story_id: formState.storyId,
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
          {stories.length === 0 ? (
            <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4 text-sm leading-7 text-[var(--color-text-secondary)]">
              {copy.createSession.emptyStories}
            </div>
          ) : (
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

              <div className="space-y-3">
                <p className="text-sm font-medium text-[var(--color-text-primary)]">
                  {copy.createSession.configMode}
                </p>

                <div className="grid gap-3 md:grid-cols-2">
                  <button
                    className={cn(
                      'rounded-[1.35rem] border px-4 py-4 text-left transition',
                      formState.configMode === 'use_global'
                        ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] text-[var(--color-text-primary)]'
                        : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] text-[var(--color-text-secondary)]',
                    )}
                    onClick={() => {
                      setFormState((current) => ({ ...current, configMode: 'use_global' }))
                    }}
                    type="button"
                  >
                    <p className="font-medium">{copy.apiPanel.modeOptions.useGlobal}</p>
                  </button>

                  <button
                    className={cn(
                      'rounded-[1.35rem] border px-4 py-4 text-left transition',
                      formState.configMode === 'use_session'
                        ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] text-[var(--color-text-primary)]'
                        : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] text-[var(--color-text-secondary)]',
                    )}
                    onClick={() => {
                      setFormState((current) => ({
                        ...current,
                        configMode: 'use_session',
                        sessionApiIds: current.sessionApiIds ?? buildDefaultSessionApiIds(apis),
                      }))
                    }}
                    type="button"
                  >
                    <p className="font-medium">{copy.apiPanel.modeOptions.useSession}</p>
                  </button>
                </div>
              </div>

              {formState.configMode === 'use_session' ? (
                apiOptions.length === 0 || !formState.sessionApiIds ? (
                  <div className="rounded-[1.35rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_72%,transparent)] px-4 py-4 text-sm leading-7 text-[var(--color-text-secondary)]">
                    {copy.createSession.emptyApis}
                  </div>
                ) : (
                  <div className="grid gap-4 md:grid-cols-2">
                    {agentApiRoleKeys.map((roleKey) => (
                      <label className="space-y-2.5" key={roleKey}>
                        <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                          {copy.apiPanel.roles[roleKey]}
                        </span>
                        <Select
                          items={apiOptions}
                          textAlign="start"
                          value={formState.sessionApiIds?.[roleKey] ?? ''}
                          onValueChange={(apiId) => {
                            updateApiRole(roleKey, apiId)
                          }}
                        />
                      </label>
                    ))}
                  </div>
                )
              ) : null}

              {playerProfiles.length === 0 ? (
                <div className="rounded-[1.35rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_72%,transparent)] px-4 py-4 text-sm leading-7 text-[var(--color-text-secondary)]">
                  {copy.createSession.emptyPlayerProfiles}
                </div>
              ) : null}
            </>
          )}

          {submitError ? (
            <div className="rounded-[1.35rem] border border-[var(--color-state-error-line)] bg-[var(--color-state-error-soft)] px-4 py-3 text-sm leading-7 text-[var(--color-text-primary)]">
              {submitError}
            </div>
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
          <Button disabled={isSubmitting || stories.length === 0} onClick={() => void handleSubmit()}>
            {isSubmitting ? copy.createSession.creating : copy.createSession.create}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
