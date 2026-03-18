import { useEffect, useId, useMemo, useState } from 'react'
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
import { useToast, useToastMessage } from '../../components/ui/toast-context'
import { createApi, getApi, listApiModels, updateApi } from './api'
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
  const { pushToast } = useToast()
  const fieldIdPrefix = useId()
  const [formState, setFormState] = useState<FormState>(createInitialState)
  const [isLoading, setIsLoading] = useState(false)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [isFetchingModels, setIsFetchingModels] = useState(false)
  const [availableModels, setAvailableModels] = useState<string[]>([])
  const [submitError, setSubmitError] = useState<string | null>(null)
  const [modelsError, setModelsError] = useState<string | null>(null)
  useToastMessage(submitError)
  useToastMessage(modelsError)

  const fieldIds = {
    apiId: `${fieldIdPrefix}-api-id`,
    apiKey: `${fieldIdPrefix}-api-key`,
    baseUrl: `${fieldIdPrefix}-base-url`,
    displayName: `${fieldIdPrefix}-display-name`,
    model: `${fieldIdPrefix}-model`,
    modelSelect: `${fieldIdPrefix}-model-select`,
    provider: `${fieldIdPrefix}-provider`,
  } as const

  const modelOptions = useMemo(
    () =>
      availableModels.map((model) => ({
        label: model,
        value: model,
      })),
    [availableModels],
  )
  const canFetchModels =
    formState.baseUrl.trim().length > 0 && formState.apiKey.trim().length > 0

  useEffect(() => {
    if (!open) {
      setFormState(createInitialState())
      setIsLoading(false)
      setIsSubmitting(false)
      setIsFetchingModels(false)
      setAvailableModels([])
      setSubmitError(null)
      setModelsError(null)
      return
    }

    if (mode !== 'edit' || !apiId) {
      setFormState(createInitialState())
      setAvailableModels([])
      setSubmitError(null)
      setModelsError(null)
      return
    }

    const controller = new AbortController()
    setIsLoading(true)
    setSubmitError(null)
    setModelsError(null)
    setAvailableModels([])

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

  async function handleFetchModels() {
    if (!formState.baseUrl.trim()) {
      setModelsError(t('apis.apiForm.errors.baseUrlRequired'))
      return
    }

    if (!formState.apiKey.trim()) {
      setModelsError(
        mode === 'edit'
          ? t('apis.apiForm.errors.apiKeyRequiredForProbe')
          : t('apis.apiForm.errors.apiKeyRequired'),
      )
      return
    }

    setModelsError(null)
    setIsFetchingModels(true)

    try {
      const result = await listApiModels({
        api_key: formState.apiKey.trim(),
        base_url: formState.baseUrl.trim(),
        provider: formState.provider,
      })
      const nextModels = Array.from(
        new Set(
          result.models
            .map((model) => model.trim())
            .filter((model) => model.length > 0),
        ),
      )

      setAvailableModels(nextModels)

      pushToast({
        message:
          nextModels.length > 0
            ? t('apis.apiForm.feedback.modelsLoaded', { count: nextModels.length })
            : t('apis.apiForm.feedback.modelsEmpty'),
        tone: nextModels.length > 0 ? 'success' : 'warning',
      })
    } catch (error) {
      setModelsError(getErrorMessage(error, t('apis.apiForm.errors.listModelsFailed')))
    } finally {
      setIsFetchingModels(false)
    }
  }

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

        <DialogBody className="flex flex-col gap-7 pt-7">
          {isLoading ? (
            <div className="space-y-4">
              {Array.from({ length: 5 }).map((_, index) => (
                <div className="h-12 animate-pulse rounded-2xl bg-[var(--color-bg-elevated)]" key={index} />
              ))}
            </div>
          ) : (
            <>
              <label className="flex flex-col gap-3">
                <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                  {t('apis.apiForm.fields.apiId')}
                </span>
                <Input
                  disabled={mode === 'edit'}
                  id={fieldIds.apiId}
                  name="api_id"
                  placeholder={t('apis.apiForm.placeholders.apiId')}
                  value={formState.apiId}
                  onChange={(event) => {
                    setFormState((current) => ({ ...current, apiId: event.target.value }))
                  }}
                />
              </label>

              <label className="flex flex-col gap-3">
                <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                  {t('apis.apiForm.fields.displayName')}
                </span>
                <Input
                  id={fieldIds.displayName}
                  name="display_name"
                  placeholder={t('apis.apiForm.placeholders.displayName')}
                  value={formState.displayName}
                  onChange={(event) => {
                    setFormState((current) => ({ ...current, displayName: event.target.value }))
                  }}
                />
              </label>

              <label className="flex flex-col gap-3">
                <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                  {t('apis.apiForm.fields.provider')}
                </span>
                <Select
                  items={llmProviders.map((provider) => ({
                    label: provider === 'open_ai' ? t('apis.providers.open_ai') : provider,
                    value: provider,
                  }))}
                  triggerId={fieldIds.provider}
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

              <label className="flex flex-col gap-3">
                <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                  {t('apis.apiForm.fields.baseUrl')}
                </span>
                <Input
                  id={fieldIds.baseUrl}
                  name="base_url"
                  placeholder={t('apis.apiForm.placeholders.baseUrl')}
                  value={formState.baseUrl}
                  onChange={(event) => {
                    setFormState((current) => ({ ...current, baseUrl: event.target.value }))
                  }}
                />
              </label>

              <label className="flex flex-col gap-3">
                <div className="space-y-1.5">
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
                  id={fieldIds.apiKey}
                  name="api_key"
                  placeholder={t('apis.apiForm.placeholders.apiKey')}
                  type="password"
                  value={formState.apiKey}
                  onChange={(event) => {
                    setFormState((current) => ({ ...current, apiKey: event.target.value }))
                  }}
                />
              </label>

              <div className="space-y-3">
                <div className="flex items-center justify-between gap-3">
                  <label
                    className="block text-sm font-medium text-[var(--color-text-primary)]"
                    htmlFor={fieldIds.model}
                  >
                    {t('apis.apiForm.fields.model')}
                  </label>
                  <Button
                    disabled={isLoading || isSubmitting || isFetchingModels || !canFetchModels}
                    onClick={() => {
                      void handleFetchModels()
                    }}
                    size="sm"
                    variant="secondary"
                  >
                    {isFetchingModels
                      ? t('apis.apiForm.actions.fetchingModels')
                      : availableModels.length > 0
                        ? t('apis.apiForm.actions.refetchModels')
                        : t('apis.apiForm.actions.fetchModels')}
                  </Button>
                </div>

                {availableModels.length > 0 ? (
                  <Select
                    allowClear
                    clearLabel={t('apis.apiForm.placeholders.modelSelectClear')}
                    items={modelOptions}
                    onValueChange={(value) => {
                      setFormState((current) => ({ ...current, model: value }))
                    }}
                    placeholder={t('apis.apiForm.placeholders.modelSelect')}
                    textAlign="start"
                    triggerId={fieldIds.modelSelect}
                    value={availableModels.includes(formState.model) ? formState.model : undefined}
                  />
                ) : null}

                <Input
                  id={fieldIds.model}
                  name="model"
                  placeholder={t('apis.apiForm.placeholders.model')}
                  value={formState.model}
                  onChange={(event) => {
                    setFormState((current) => ({ ...current, model: event.target.value }))
                  }}
                />

                <p className="text-xs leading-6 text-[var(--color-text-muted)]">
                  {availableModels.length > 0
                    ? t('apis.apiForm.hints.modelEditable')
                    : t('apis.apiForm.hints.modelProbe')}
                </p>
              </div>

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
