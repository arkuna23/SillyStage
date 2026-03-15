import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'

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
import { useToastMessage } from '../../components/ui/toast-context'
import { createApi, getApi, updateApi } from './api'
import { llmProviders, type ApiConfig, type LlmProvider } from './types'

type ApiFormDialogProps = {
  apiId?: string | null
  existingApiIds: ReadonlyArray<string>
  mode: 'create' | 'edit'
  onCompleted: (result: { api: ApiConfig; message: string }) => Promise<void> | void
  onOpenChange: (open: boolean) => void
  open: boolean
}

type FormState = {
  apiId: string
  apiKey: string
  baseUrl: string
  displayName: string
  model: string
  provider: LlmProvider
}

function createInitialState(): FormState {
  return {
    apiId: '',
    apiKey: '',
    baseUrl: '',
    displayName: '',
    model: '',
    provider: llmProviders[0],
  }
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

export function ApiFormDialog({
  apiId,
  existingApiIds,
  mode,
  onCompleted,
  onOpenChange,
  open,
}: ApiFormDialogProps) {
  const { t } = useTranslation()
  const [formState, setFormState] = useState<FormState>(createInitialState)
  const [isLoading, setIsLoading] = useState(false)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)
  useToastMessage(submitError)

  useEffect(() => {
    if (!open) {
      setFormState(createInitialState())
      setIsLoading(false)
      setIsSubmitting(false)
      setSubmitError(null)
      return
    }

    if (mode !== 'edit' || !apiId) {
      setFormState(createInitialState())
      setSubmitError(null)
      return
    }

    const controller = new AbortController()
    setIsLoading(true)
    setSubmitError(null)

    void getApi(apiId, controller.signal)
      .then((result) => {
        if (controller.signal.aborted) {
          return
        }

        setFormState({
          apiId: result.api_id,
          apiKey: '',
          baseUrl: result.base_url,
          displayName: result.display_name,
          model: result.model,
          provider: result.provider,
        })
      })
      .catch((error) => {
        if (!controller.signal.aborted) {
          setSubmitError(getErrorMessage(error, t('apis.apiForm.errors.loadFailed')))
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
  }, [apiId, mode, open, t])

  async function handleSubmit() {
    if (!formState.apiId.trim()) {
      setSubmitError(t('apis.apiForm.errors.apiIdRequired'))
      return
    }

    if (mode === 'create' && existingApiIds.includes(formState.apiId.trim())) {
      setSubmitError(t('apis.apiForm.errors.apiIdDuplicate'))
      return
    }

    if (!formState.displayName.trim()) {
      setSubmitError(t('apis.apiForm.errors.displayNameRequired'))
      return
    }

    if (!formState.baseUrl.trim()) {
      setSubmitError(t('apis.apiForm.errors.baseUrlRequired'))
      return
    }

    if (!formState.model.trim()) {
      setSubmitError(t('apis.apiForm.errors.modelRequired'))
      return
    }

    if (mode === 'create' && !formState.apiKey.trim()) {
      setSubmitError(t('apis.apiForm.errors.apiKeyRequired'))
      return
    }

    setSubmitError(null)
    setIsSubmitting(true)

    try {
      const api =
        mode === 'create'
          ? await createApi({
              api_id: formState.apiId.trim(),
              api_key: formState.apiKey.trim(),
              base_url: formState.baseUrl.trim(),
              display_name: formState.displayName.trim(),
              model: formState.model.trim(),
              provider: formState.provider,
            })
          : await updateApi({
              ...(formState.apiKey.trim() ? { api_key: formState.apiKey.trim() } : {}),
              api_id: formState.apiId.trim(),
              base_url: formState.baseUrl.trim(),
              display_name: formState.displayName.trim(),
              model: formState.model.trim(),
              provider: formState.provider,
            })

      await onCompleted({
        api,
        message:
          mode === 'create'
            ? t('apis.apiFeedback.created', { id: api.display_name })
            : t('apis.apiFeedback.updated', { id: api.display_name }),
      })

      onOpenChange(false)
    } catch (error) {
      setSubmitError(getErrorMessage(error, t('apis.apiForm.errors.submitFailed')))
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,40rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>
            {mode === 'create' ? t('apis.apiForm.createTitle') : t('apis.apiForm.editTitle')}
          </DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-5 pt-6">
          {isLoading ? (
            <div className="space-y-4">
              {Array.from({ length: 5 }).map((_, index) => (
                <div className="h-12 animate-pulse rounded-2xl bg-[var(--color-bg-elevated)]" key={index} />
              ))}
            </div>
          ) : (
            <>
              <label className="space-y-2.5">
                <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                  {t('apis.apiForm.fields.apiId')}
                </span>
                <Input
                  disabled={mode === 'edit'}
                  placeholder={t('apis.apiForm.placeholders.apiId')}
                  value={formState.apiId}
                  onChange={(event) => {
                    setFormState((current) => ({ ...current, apiId: event.target.value }))
                  }}
                />
              </label>

              <label className="space-y-2.5">
                <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                  {t('apis.apiForm.fields.displayName')}
                </span>
                <Input
                  placeholder={t('apis.apiForm.placeholders.displayName')}
                  value={formState.displayName}
                  onChange={(event) => {
                    setFormState((current) => ({ ...current, displayName: event.target.value }))
                  }}
                />
              </label>

              <label className="space-y-2.5">
                <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                  {t('apis.apiForm.fields.provider')}
                </span>
                <Select
                  items={llmProviders.map((provider) => ({
                    label: provider === 'open_ai' ? t('apis.providers.open_ai') : provider,
                    value: provider,
                  }))}
                  textAlign="start"
                  value={formState.provider}
                  onValueChange={(value) => {
                    setFormState((current) => ({
                      ...current,
                      provider: value as LlmProvider,
                    }))
                  }}
                />
              </label>

              <label className="space-y-2.5">
                <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                  {t('apis.apiForm.fields.baseUrl')}
                </span>
                <Input
                  placeholder={t('apis.apiForm.placeholders.baseUrl')}
                  value={formState.baseUrl}
                  onChange={(event) => {
                    setFormState((current) => ({ ...current, baseUrl: event.target.value }))
                  }}
                />
              </label>

              <label className="space-y-2.5">
                <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                  {t('apis.apiForm.fields.model')}
                </span>
                <Input
                  placeholder={t('apis.apiForm.placeholders.model')}
                  value={formState.model}
                  onChange={(event) => {
                    setFormState((current) => ({ ...current, model: event.target.value }))
                  }}
                />
              </label>

              <label className="space-y-2.5">
                <div className="space-y-1">
                  <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                    {t('apis.apiForm.fields.apiKey')}
                  </span>
                  {mode === 'edit' ? (
                    <p className="text-xs leading-6 text-[var(--color-text-muted)]">
                      {t('apis.apiForm.fields.apiKeyHint')}
                    </p>
                  ) : null}
                </div>
                <Input
                  placeholder={t('apis.apiForm.placeholders.apiKey')}
                  type="password"
                  value={formState.apiKey}
                  onChange={(event) => {
                    setFormState((current) => ({ ...current, apiKey: event.target.value }))
                  }}
                />
              </label>

            </>
          )}
        </DialogBody>

        <DialogFooter className="justify-end gap-2">
          <Button onClick={() => onOpenChange(false)} variant="ghost">
            {t('apis.actions.cancel')}
          </Button>
          <Button disabled={isLoading || isSubmitting} onClick={() => void handleSubmit()}>
            {isSubmitting ? t('apis.actions.saving') : t('apis.actions.save')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
