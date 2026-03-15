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
import { useToastMessage } from '../../components/ui/toast-context'
import { getStory, updateStory } from './api'
import type { StoryDetail } from './types'

type StoryFormDialogProps = {
  onCompleted: (result: { message: string; story: StoryDetail }) => Promise<void> | void
  onOpenChange: (open: boolean) => void
  open: boolean
  storyId?: string | null
}

type FormState = {
  displayName: string
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

export function StoryFormDialog({
  onCompleted,
  onOpenChange,
  open,
  storyId,
}: StoryFormDialogProps) {
  const { t } = useTranslation()
  const fieldIdPrefix = useId()
  const [formState, setFormState] = useState<FormState>({ displayName: '' })
  const [initialStory, setInitialStory] = useState<StoryDetail | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)
  useToastMessage(submitError)

  const fieldIds = {
    displayName: `${fieldIdPrefix}-display-name`,
  } as const

  useEffect(() => {
    if (!open || !storyId) {
      setFormState({ displayName: '' })
      setInitialStory(null)
      setIsLoading(false)
      setIsSubmitting(false)
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
        setFormState({ displayName: story.display_name })
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

    setSubmitError(null)
    setIsSubmitting(true)

    try {
      const story = await updateStory({
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
        className="w-[min(92vw,38rem)]"
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
            </div>
          ) : (
            <>
              {initialStory ? (
                <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
                  <p className="text-xs text-[var(--color-text-muted)]">
                    {t('stories.form.fields.storyId')}
                  </p>
                  <p className="mt-2 font-mono text-sm leading-6 text-[var(--color-text-primary)]">
                    {initialStory.story_id}
                  </p>
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
                  placeholder={t('stories.form.placeholders.displayName')}
                  value={formState.displayName}
                  onChange={(event) => {
                    const { value } = event.target

                    setFormState({ displayName: value })
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
