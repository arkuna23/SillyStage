import { useEffect, useId, useState } from 'react'
import type { PropsWithChildren } from 'react'
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
import { Select } from '../../components/ui/select'
import { createLlmApi, getLlmApi, updateLlmApi } from './api'
import { llmProviders, type LlmApi, type LlmProvider } from './types'

type LlmApiFormDialogProps = {
  apiId?: string | null
  existingApiIds: ReadonlyArray<string>
  mode: 'create' | 'edit'
  onCompleted: (result: { api: LlmApi; message: string }) => Promise<void> | void
  onOpenChange: (open: boolean) => void
  open: boolean
}

type FormState = {
  apiId: string
  apiKey: string
  baseUrl: string
  maxTokens: string
  model: string
  provider: LlmProvider
  temperature: string
}

function createInitialFormState(): FormState {
  return {
    apiId: '',
    apiKey: '',
    baseUrl: '',
    maxTokens: '',
    model: '',
    provider: llmProviders[0],
    temperature: '',
  }
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function providerOptions(openAiLabel: string) {
  return llmProviders.map((provider) => ({
    label: provider === 'open_ai' ? openAiLabel : provider,
    value: provider,
  }))
}

function Field({
  children,
  description,
  htmlFor,
  label,
}: PropsWithChildren<{
  description?: string
  htmlFor?: string
  label: string
}>) {
  return (
    <div className="space-y-2.5">
      <label
        className="block text-sm font-medium text-[var(--color-text-primary)]"
        htmlFor={htmlFor}
      >
        {label}
      </label>
      {children}
      {description ? (
        <p className="text-xs leading-6 text-[var(--color-text-muted)]">{description}</p>
      ) : null}
    </div>
  )
}

export function LlmApiFormDialog({
  apiId,
  existingApiIds,
  mode,
  onCompleted,
  onOpenChange,
  open,
}: LlmApiFormDialogProps) {
  const { t } = useTranslation()
  const openAiLabel = String(t('apis.providers.open_ai'))
  const fieldIdPrefix = useId()
  const [formState, setFormState] = useState<FormState>(createInitialFormState)
  const [initialApi, setInitialApi] = useState<LlmApi | null>(null)
  const [isLoading, setIsLoading] = useState(false)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)

  const fieldIds = {
    apiId: `${fieldIdPrefix}-api-id`,
    apiKey: `${fieldIdPrefix}-api-key`,
    baseUrl: `${fieldIdPrefix}-base-url`,
    maxTokens: `${fieldIdPrefix}-max-tokens`,
    model: `${fieldIdPrefix}-model`,
    provider: `${fieldIdPrefix}-provider`,
    temperature: `${fieldIdPrefix}-temperature`,
  } as const

  useEffect(() => {
    if (!open) {
      setFormState(createInitialFormState())
      setInitialApi(null)
      setIsLoading(false)
      setIsSubmitting(false)
      setSubmitError(null)
      return
    }

    if (mode === 'create') {
      setFormState(createInitialFormState())
      setInitialApi(null)
      setSubmitError(null)
      return
    }

    if (!apiId) {
      return
    }

    const controller = new AbortController()

    setIsLoading(true)
    setSubmitError(null)

    void getLlmApi(apiId, controller.signal)
      .then((api) => {
        if (controller.signal.aborted) {
          return
        }

        setInitialApi(api)
        setFormState({
          apiId: api.api_id,
          apiKey: '',
          baseUrl: api.base_url,
          maxTokens: api.max_tokens?.toString() ?? '',
          model: api.model,
          provider: api.provider,
          temperature: api.temperature?.toString() ?? '',
        })
      })
      .catch((error) => {
        if (!controller.signal.aborted) {
          setSubmitError(getErrorMessage(error, t('apis.form.errors.loadFailed')))
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

  function validateForm() {
    const nextApiId = formState.apiId.trim()
    const nextBaseUrl = formState.baseUrl.trim()
    const nextMaxTokens = formState.maxTokens.trim()
    const nextModel = formState.model.trim()
    const nextApiKey = formState.apiKey.trim()
    const nextTemperature = formState.temperature.trim()

    if (nextApiId.length === 0) {
      return t('apis.form.errors.apiIdRequired')
    }

    if (
      mode === 'create' &&
      existingApiIds.some((existingApiId) => existingApiId === nextApiId)
    ) {
      return t('apis.form.errors.apiIdDuplicate')
    }

    if (nextBaseUrl.length === 0) {
      return t('apis.form.errors.baseUrlRequired')
    }

    if (nextModel.length === 0) {
      return t('apis.form.errors.modelRequired')
    }

    if (mode === 'create' && nextApiKey.length === 0) {
      return t('apis.form.errors.apiKeyRequired')
    }

    if (nextTemperature.length > 0 && Number.isNaN(Number(nextTemperature))) {
      return t('apis.form.errors.temperatureInvalid')
    }

    if (
      nextMaxTokens.length > 0 &&
      (!Number.isInteger(Number(nextMaxTokens)) || Number(nextMaxTokens) < 1)
    ) {
      return t('apis.form.errors.maxTokensInvalid')
    }

    return null
  }

  async function handleSubmit() {
    const validationError = validateForm()

    if (validationError) {
      setSubmitError(validationError)
      return
    }

    const nextApiId = formState.apiId.trim()
    const nextBaseUrl = formState.baseUrl.trim()
    const nextMaxTokens = formState.maxTokens.trim()
    const nextModel = formState.model.trim()
    const nextApiKey = formState.apiKey.trim()
    const nextTemperature = formState.temperature.trim()
    const parsedMaxTokens = nextMaxTokens.length > 0 ? Number(nextMaxTokens) : null
    const parsedTemperature = nextTemperature.length > 0 ? Number(nextTemperature) : null

    setIsSubmitting(true)
    setSubmitError(null)

    try {
      if (mode === 'edit' && !initialApi) {
        setSubmitError(t('apis.form.errors.loadFailed'))
        return
      }

      if (
        mode === 'edit' &&
        initialApi &&
        formState.provider === initialApi.provider &&
        nextBaseUrl === initialApi.base_url &&
        parsedMaxTokens === (initialApi.max_tokens ?? null) &&
        nextModel === initialApi.model &&
        nextApiKey.length === 0 &&
        parsedTemperature === (initialApi.temperature ?? null)
      ) {
        onOpenChange(false)
        return
      }

      const result =
        mode === 'create'
          ? await createLlmApi({
              api_id: nextApiId,
              api_key: nextApiKey,
              base_url: nextBaseUrl,
              ...(parsedMaxTokens !== null ? { max_tokens: parsedMaxTokens } : {}),
              model: nextModel,
              provider: formState.provider,
              ...(parsedTemperature !== null ? { temperature: parsedTemperature } : {}),
            })
          : await updateLlmApi({
              api_id: nextApiId,
              ...(initialApi && formState.provider !== initialApi.provider
                ? { provider: formState.provider }
                : {}),
              ...(initialApi && nextBaseUrl !== initialApi.base_url
                ? { base_url: nextBaseUrl }
                : {}),
              ...(initialApi && parsedMaxTokens !== (initialApi.max_tokens ?? null)
                ? { max_tokens: parsedMaxTokens ?? undefined }
                : {}),
              ...(nextApiKey.length > 0 ? { api_key: nextApiKey } : {}),
              ...(initialApi && nextModel !== initialApi.model ? { model: nextModel } : {}),
              ...(initialApi && parsedTemperature !== (initialApi.temperature ?? null)
                ? { temperature: parsedTemperature ?? undefined }
                : {}),
            })

      await onCompleted({
        api: result,
        message:
          mode === 'create'
            ? t('apis.feedback.created', { id: result.api_id })
            : t('apis.feedback.updated', { id: result.api_id }),
      })

      onOpenChange(false)
    } catch (error) {
      setSubmitError(getErrorMessage(error, t('apis.form.errors.submitFailed')))
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,42rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>
            {mode === 'create' ? t('apis.form.createTitle') : t('apis.form.editTitle')}
          </DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-5 pt-6">
          {submitError ? (
            <div className="rounded-[1.25rem] border border-[var(--color-state-error-line)] bg-[var(--color-state-error-soft)] px-4 py-3 text-sm text-[var(--color-text-primary)]">
              {submitError}
            </div>
          ) : null}

          {isLoading ? (
            <div className="space-y-4">
              {Array.from({ length: 4 }).map((_, index) => (
                <div className="space-y-2.5" key={index}>
                  <div className="h-3 w-24 animate-pulse rounded-full bg-[var(--color-bg-elevated)]" />
                  <div className="h-12 animate-pulse rounded-2xl bg-[var(--color-bg-elevated)]" />
                </div>
              ))}
            </div>
          ) : (
            <div className="grid gap-5">
              <Field htmlFor={fieldIds.apiId} label={t('apis.form.fields.apiId')}>
                <Input
                  disabled={mode === 'edit' || isSubmitting}
                  id={fieldIds.apiId}
                  onChange={(event) => {
                    setFormState((current) => ({ ...current, apiId: event.target.value }))
                  }}
                  placeholder={t('apis.form.placeholders.apiId')}
                  value={formState.apiId}
                />
              </Field>

              <Field htmlFor={fieldIds.provider} label={t('apis.form.fields.provider')}>
                <Select
                  disabled={isSubmitting}
                  items={providerOptions(openAiLabel)}
                  onValueChange={(value) => {
                    setFormState((current) => ({
                      ...current,
                      provider: value as LlmProvider,
                    }))
                  }}
                  textAlign="start"
                  triggerId={fieldIds.provider}
                  value={formState.provider}
                />
              </Field>

              <Field htmlFor={fieldIds.baseUrl} label={t('apis.form.fields.baseUrl')}>
                <Input
                  disabled={isSubmitting}
                  id={fieldIds.baseUrl}
                  onChange={(event) => {
                    setFormState((current) => ({ ...current, baseUrl: event.target.value }))
                  }}
                  placeholder={t('apis.form.placeholders.baseUrl')}
                  value={formState.baseUrl}
                />
              </Field>

              <Field
                description={
                  mode === 'edit' ? t('apis.form.fields.apiKeyHint') : undefined
                }
                htmlFor={fieldIds.apiKey}
                label={t('apis.form.fields.apiKey')}
              >
                <Input
                  autoComplete="off"
                  disabled={isSubmitting}
                  id={fieldIds.apiKey}
                  onChange={(event) => {
                    setFormState((current) => ({ ...current, apiKey: event.target.value }))
                  }}
                  placeholder={t('apis.form.placeholders.apiKey')}
                  type="password"
                  value={formState.apiKey}
                />
              </Field>

              <Field htmlFor={fieldIds.model} label={t('apis.form.fields.model')}>
                <Input
                  disabled={isSubmitting}
                  id={fieldIds.model}
                  onChange={(event) => {
                    setFormState((current) => ({ ...current, model: event.target.value }))
                  }}
                  placeholder={t('apis.form.placeholders.model')}
                  value={formState.model}
                />
              </Field>

              <div className="grid gap-5 md:grid-cols-2">
                <Field htmlFor={fieldIds.temperature} label={t('apis.form.fields.temperature')}>
                  <Input
                    disabled={isSubmitting}
                    id={fieldIds.temperature}
                    inputMode="decimal"
                    onChange={(event) => {
                      setFormState((current) => ({ ...current, temperature: event.target.value }))
                    }}
                    placeholder={t('apis.form.placeholders.temperature')}
                    value={formState.temperature}
                  />
                </Field>

                <Field htmlFor={fieldIds.maxTokens} label={t('apis.form.fields.maxTokens')}>
                  <Input
                    disabled={isSubmitting}
                    id={fieldIds.maxTokens}
                    inputMode="numeric"
                    onChange={(event) => {
                      setFormState((current) => ({ ...current, maxTokens: event.target.value }))
                    }}
                    placeholder={t('apis.form.placeholders.maxTokens')}
                    value={formState.maxTokens}
                  />
                </Field>
              </div>
            </div>
          )}
        </DialogBody>

        <DialogFooter>
          <DialogClose asChild>
            <Button disabled={isSubmitting} variant="ghost">
              {t('apis.actions.cancel')}
            </Button>
          </DialogClose>

          <Button disabled={isLoading || isSubmitting} onClick={() => void handleSubmit()}>
            {isSubmitting ? t('apis.actions.saving') : t('apis.actions.save')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
