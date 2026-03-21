import { useEffect, useId, useMemo, useState } from 'react'
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
import { useToastMessage } from '../../components/ui/toast-context'
import type { CharacterSummary } from '../characters/types'
import { getStoryResourceOptionLabel } from '../story-resources/story-resource-display'
import type { StoryResource } from '../story-resources/types'
import { getStory, updateStory } from './api'
import {
  createStoryCommonVariableDrafts,
  type StoryCommonVariableDraft,
  type StoryCommonVariableDraftErrors,
  serializeStoryCommonVariableDrafts,
  validateStoryCommonVariableDrafts,
} from './story-common-variable-drafts'
import { useStoryCommonVariableSchemaCatalog } from './story-common-variable-schema-catalog'
import { StoryCommonVariablesEditor } from './story-common-variables-editor'
import type { StoryDetail } from './types'

type StoryFormDialogProps = {
  availableCharacters: ReadonlyArray<CharacterSummary>
  availableResources: ReadonlyArray<StoryResource>
  onCompleted: (result: { message: string; story: StoryDetail }) => Promise<void> | void
  onOpenChange: (open: boolean) => void
  open: boolean
  storyId?: string | null
}

type FormState = {
  commonVariables: StoryCommonVariableDraft[]
  displayName: string
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

export function StoryFormDialog({
  availableCharacters,
  availableResources,
  onCompleted,
  onOpenChange,
  open,
  storyId,
}: StoryFormDialogProps) {
  const { t } = useTranslation()
  const fieldIdPrefix = useId()
  const [formState, setFormState] = useState<FormState>({
    commonVariables: [],
    displayName: '',
  })
  const [initialStory, setInitialStory] = useState<StoryDetail | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)
  const [commonVariableErrors, setCommonVariableErrors] = useState<StoryCommonVariableDraftErrors>(
    {},
  )
  useToastMessage(submitError)

  const fieldIds = {
    displayName: `${fieldIdPrefix}-display-name`,
  } as const

  useEffect(() => {
    if (!open || !storyId) {
      setFormState({ commonVariables: [], displayName: '' })
      setInitialStory(null)
      setIsLoading(false)
      setIsSubmitting(false)
      setCommonVariableErrors({})
      setSubmitError(null)
      return
    }

    const controller = new AbortController()
    setIsLoading(true)
    setIsSubmitting(false)
    setSubmitError(null)

    void getStory(storyId, controller.signal)
      .then((story) => {
        if (controller.signal.aborted) {
          return
        }

        setInitialStory(story)
        setFormState({
          commonVariables: createStoryCommonVariableDrafts(story.common_variables),
          displayName: story.display_name,
        })
        setCommonVariableErrors({})
      })
      .catch((error) => {
        if (!controller.signal.aborted) {
          setSubmitError(getErrorMessage(error, t('stories.feedback.loadStoryFailed')))
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
  }, [open, storyId, t])

  const resourceCharacterIds = useMemo(
    () =>
      availableResources.find((resource) => resource.resource_id === initialStory?.resource_id)
        ?.character_ids ??
      initialStory?.common_variables
        .filter((definition) => definition.scope === 'character' && definition.character_id)
        .map((definition) => definition.character_id ?? '') ??
      [],
    [availableResources, initialStory],
  )
  const linkedResource = useMemo(
    () =>
      availableResources.find((resource) => resource.resource_id === initialStory?.resource_id) ??
      null,
    [availableResources, initialStory?.resource_id],
  )
  const commonVariableCharacterIds = useMemo(() => {
    const knownCharacterIds = new Set(resourceCharacterIds)

    formState.commonVariables.forEach((draft) => {
      if (draft.scope !== 'character') {
        return
      }

      const characterId = draft.character_id.trim()

      if (characterId.length > 0) {
        knownCharacterIds.add(characterId)
      }
    })

    return Array.from(knownCharacterIds)
  }, [formState.commonVariables, resourceCharacterIds])
  const commonVariableSchemaCatalog = useStoryCommonVariableSchemaCatalog({
    characterIds: commonVariableCharacterIds,
    enabled: open && Boolean(initialStory),
    playerSchemaId: initialStory?.player_schema_id,
    worldSchemaId: initialStory?.world_schema_id,
  })

  async function handleSubmit() {
    if (!initialStory) {
      setSubmitError(t('stories.feedback.loadStoryFailed'))
      return
    }

    const displayName = formState.displayName.trim()

    if (displayName.length === 0) {
      setSubmitError(t('stories.form.errors.displayNameRequired'))
      return
    }

    const nextCommonVariableErrors = validateStoryCommonVariableDrafts(
      formState.commonVariables,
      new Set(resourceCharacterIds),
    )

    if (Object.keys(nextCommonVariableErrors).length > 0) {
      setCommonVariableErrors(nextCommonVariableErrors)
      setSubmitError(t('stories.form.errors.commonVariablesInvalid'))
      return
    }

    setSubmitError(null)
    setCommonVariableErrors({})
    setIsSubmitting(true)

    try {
      const story = await updateStory({
        common_variables: serializeStoryCommonVariableDrafts(formState.commonVariables),
        display_name: displayName,
        story_id: initialStory.story_id,
      })

      await onCompleted({
        message: t('stories.feedback.updated', { name: story.display_name }),
        story,
      })

      onOpenChange(false)
    } catch (error) {
      setSubmitError(getErrorMessage(error, t('stories.form.errors.submitFailed')))
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent
        aria-describedby={undefined}
        className="w-[min(96vw,56rem)]"
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
          <DialogTitle>{t('stories.form.editTitle')}</DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-5 pt-6">
          {isLoading ? (
            <div className="space-y-4">
              <div className="h-12 animate-pulse rounded-2xl bg-[var(--color-bg-elevated)]" />
              <div className="h-24 animate-pulse rounded-[1.45rem] bg-[var(--color-bg-elevated)]" />
              <div className="h-64 animate-pulse rounded-[1.45rem] bg-[var(--color-bg-elevated)]" />
            </div>
          ) : (
            <>
              {initialStory ? (
                <div className="grid gap-3 md:grid-cols-2">
                  <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
                    <p className="text-xs text-[var(--color-text-muted)]">
                      {t('stories.form.fields.storyId')}
                    </p>
                    <p className="mt-2 font-mono text-sm leading-6 text-[var(--color-text-primary)]">
                      {initialStory.story_id}
                    </p>
                  </div>
                  <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
                    <p className="text-xs text-[var(--color-text-muted)]">
                      {t('stories.form.fields.resourceId')}
                    </p>
                    <p className="mt-2 text-sm leading-7 text-[var(--color-text-primary)]">
                      {linkedResource
                        ? getStoryResourceOptionLabel(linkedResource)
                        : initialStory.resource_id}
                    </p>
                  </div>
                </div>
              ) : null}

              <div className="space-y-2.5">
                <label
                  className="block text-sm font-medium text-[var(--color-text-primary)]"
                  htmlFor={fieldIds.displayName}
                >
                  {t('stories.form.fields.displayName')}
                </label>
                <Input
                  id={fieldIds.displayName}
                  name={fieldIds.displayName}
                  placeholder={t('stories.form.placeholders.displayName')}
                  value={formState.displayName}
                  onChange={(event) => {
                    const { value } = event.target

                    setFormState((currentFormState) => ({
                      ...currentFormState,
                      displayName: value,
                    }))
                  }}
                />
              </div>

              <div className="space-y-2.5">
                <div className="space-y-1">
                  <h3 className="text-sm font-medium text-[var(--color-text-primary)]">
                    {t('stories.form.fields.commonVariables')}
                  </h3>
                </div>

                <StoryCommonVariablesEditor
                  characters={availableCharacters}
                  disabled={isSubmitting}
                  drafts={formState.commonVariables}
                  errors={commonVariableErrors}
                  resourceCharacterIds={resourceCharacterIds}
                  schemaCatalog={commonVariableSchemaCatalog}
                  onChange={(commonVariables) => {
                    setCommonVariableErrors({})
                    setFormState((currentFormState) => ({
                      ...currentFormState,
                      commonVariables,
                    }))
                  }}
                />
              </div>
            </>
          )}
        </DialogBody>

        <DialogFooter className="justify-end">
          <DialogClose asChild>
            <Button disabled={isSubmitting} variant="secondary">
              {t('stories.actions.cancel')}
            </Button>
          </DialogClose>
          <Button
            disabled={isLoading || isSubmitting || initialStory === null}
            onClick={() => void handleSubmit()}
          >
            {isSubmitting ? t('stories.actions.saving') : t('stories.actions.save')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
