import { useEffect, useMemo, useState } from 'react'
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
import { createApiGroup, getApiGroup, updateApiGroup } from './api'
import {
  type AgentRoleKey,
  type ApiConfig,
  type ApiGroup,
  type ApiGroupBindings,
  agentRoleKeys,
  getAgentBindingKey,
} from './types'

type ApiGroupFormDialogProps = {
  apiGroupId?: string | null
  apis: ReadonlyArray<ApiConfig>
  existingApiGroupIds: ReadonlyArray<string>
  mode: 'create' | 'edit'
  onCompleted: (result: { apiGroup: ApiGroup; message: string }) => Promise<void> | void
  onOpenChange: (open: boolean) => void
  open: boolean
}

type FormState = {
  apiGroupId: string
  bindings: Record<AgentRoleKey, string>
  displayName: string
}

function createInitialBindings(defaultApiId = ''): FormState['bindings'] {
  return {
    actor: defaultApiId,
    architect: defaultApiId,
    director: defaultApiId,
    keeper: defaultApiId,
    narrator: defaultApiId,
    planner: defaultApiId,
    replyer: defaultApiId,
  }
}

function createInitialState(defaultApiId = ''): FormState {
  return {
    apiGroupId: '',
    bindings: createInitialBindings(defaultApiId),
    displayName: '',
  }
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function toBindings(bindings: FormState['bindings']): ApiGroupBindings {
  return Object.fromEntries(
    agentRoleKeys.map((roleKey) => [getAgentBindingKey(roleKey), bindings[roleKey].trim()]),
  ) as ApiGroupBindings
}

export function ApiGroupFormDialog({
  apiGroupId,
  apis,
  existingApiGroupIds,
  mode,
  onCompleted,
  onOpenChange,
  open,
}: ApiGroupFormDialogProps) {
  const { t } = useTranslation()
  const [formState, setFormState] = useState<FormState>(() =>
    createInitialState(apis[0]?.api_id ?? ''),
  )
  const [isLoading, setIsLoading] = useState(false)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)
  useToastMessage(submitError)

  const roleLabels: Record<AgentRoleKey, string> = useMemo(
    () => ({
      actor: t('apis.roles.actor'),
      architect: t('apis.roles.architect'),
      director: t('apis.roles.director'),
      keeper: t('apis.roles.keeper'),
      narrator: t('apis.roles.narrator'),
      planner: t('apis.roles.planner'),
      replyer: t('apis.roles.replyer'),
    }),
    [t],
  )

  const baseApiItems = useMemo(
    () =>
      apis.map((apiConfig) => ({
        label: `${apiConfig.display_name} · ${apiConfig.api_id}`,
        value: apiConfig.api_id,
      })),
    [apis],
  )

  useEffect(() => {
    if (!open) {
      setFormState(createInitialState(apis[0]?.api_id ?? ''))
      setIsLoading(false)
      setIsSubmitting(false)
      setSubmitError(null)
      return
    }

    if (mode !== 'edit' || !apiGroupId) {
      setFormState(createInitialState(apis[0]?.api_id ?? ''))
      setSubmitError(null)
      return
    }

    const controller = new AbortController()
    setIsLoading(true)
    setSubmitError(null)

    void getApiGroup(apiGroupId, controller.signal)
      .then((result) => {
        if (controller.signal.aborted) {
          return
        }

        setFormState({
          apiGroupId: result.api_group_id,
          bindings: Object.fromEntries(
            agentRoleKeys.map((roleKey) => [
              roleKey,
              result.bindings[getAgentBindingKey(roleKey)] ?? '',
            ]),
          ) as FormState['bindings'],
          displayName: result.display_name,
        })
      })
      .catch((error) => {
        if (!controller.signal.aborted) {
          setSubmitError(getErrorMessage(error, t('apis.groupForm.errors.loadFailed')))
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
  }, [apiGroupId, apis, mode, open, t])

  function getApiItemsForRole(roleKey: AgentRoleKey) {
    const selectedApiId = formState.bindings[roleKey]

    if (!selectedApiId.trim() || baseApiItems.some((item) => item.value === selectedApiId)) {
      return baseApiItems
    }

    return [
      {
        label: t('apis.groupForm.missingApi', { id: selectedApiId }),
        value: selectedApiId,
      },
      ...baseApiItems,
    ]
  }

  async function handleSubmit() {
    if (!formState.apiGroupId.trim()) {
      setSubmitError(t('apis.groupForm.errors.apiGroupIdRequired'))
      return
    }

    if (mode === 'create' && existingApiGroupIds.includes(formState.apiGroupId.trim())) {
      setSubmitError(t('apis.groupForm.errors.apiGroupIdDuplicate'))
      return
    }

    if (!formState.displayName.trim()) {
      setSubmitError(t('apis.groupForm.errors.displayNameRequired'))
      return
    }

    if (apis.length === 0) {
      setSubmitError(t('apis.groupForm.errors.noApis'))
      return
    }

    for (const roleKey of agentRoleKeys) {
      if (!formState.bindings[roleKey].trim()) {
        setSubmitError(
          t('apis.groupForm.errors.bindingRequired', {
            role: roleLabels[roleKey],
          }),
        )
        return
      }
    }

    setSubmitError(null)
    setIsSubmitting(true)

    try {
      const bindings = toBindings(formState.bindings)
      const apiGroup =
        mode === 'create'
          ? await createApiGroup({
              api_group_id: formState.apiGroupId.trim(),
              bindings,
              display_name: formState.displayName.trim(),
            })
          : await updateApiGroup({
              api_group_id: formState.apiGroupId.trim(),
              bindings,
              display_name: formState.displayName.trim(),
            })

      await onCompleted({
        apiGroup,
        message:
          mode === 'create'
            ? t('apis.groupFeedback.created', { id: apiGroup.display_name })
            : t('apis.groupFeedback.updated', { id: apiGroup.display_name }),
      })

      onOpenChange(false)
    } catch (error) {
      setSubmitError(getErrorMessage(error, t('apis.groupForm.errors.submitFailed')))
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,48rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>
            {mode === 'create' ? t('apis.groupForm.createTitle') : t('apis.groupForm.editTitle')}
          </DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-5 pt-6">
          {isLoading ? (
            <div className="space-y-4">
              {Array.from({ length: 4 }).map((_, index) => (
                <div
                  className="h-16 animate-pulse rounded-[1.35rem] bg-[var(--color-bg-elevated)]"
                  key={index}
                />
              ))}
            </div>
          ) : (
            <>
              <div className="grid gap-4 md:grid-cols-2">
                <label className="space-y-2.5">
                  <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                    {t('apis.groupForm.fields.apiGroupId')}
                  </span>
                  <Input
                    disabled={mode === 'edit'}
                    placeholder={t('apis.groupForm.placeholders.apiGroupId')}
                    value={formState.apiGroupId}
                    onChange={(event) => {
                      setFormState((current) => ({ ...current, apiGroupId: event.target.value }))
                    }}
                  />
                </label>

                <label className="space-y-2.5">
                  <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                    {t('apis.groupForm.fields.displayName')}
                  </span>
                  <Input
                    placeholder={t('apis.groupForm.placeholders.displayName')}
                    value={formState.displayName}
                    onChange={(event) => {
                      setFormState((current) => ({ ...current, displayName: event.target.value }))
                    }}
                  />
                </label>
              </div>

              <div className="grid gap-4 md:grid-cols-2">
                {agentRoleKeys.map((roleKey) => (
                  <label className="space-y-2.5" key={roleKey}>
                    <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                      {roleLabels[roleKey]}
                    </span>
                    <Select
                      items={getApiItemsForRole(roleKey)}
                      placeholder={t('apis.groupForm.placeholders.apiBinding')}
                      textAlign="start"
                      value={formState.bindings[roleKey]}
                      onValueChange={(value) => {
                        setFormState((current) => ({
                          ...current,
                          bindings: {
                            ...current.bindings,
                            [roleKey]: value,
                          },
                        }))
                      }}
                    />
                  </label>
                ))}
              </div>
            </>
          )}
        </DialogBody>

        <DialogFooter className="justify-end gap-2">
          <Button onClick={() => onOpenChange(false)} variant="ghost">
            {t('apis.actions.cancel')}
          </Button>
          <Button
            disabled={isLoading || isSubmitting || apis.length === 0}
            onClick={() => void handleSubmit()}
          >
            {isSubmitting ? t('apis.actions.saving') : t('apis.actions.save')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
