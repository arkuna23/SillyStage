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
import { Textarea } from '../../components/ui/textarea'
import { useToastMessage } from '../../components/ui/toast-context'
import { createPreset, getPreset, updatePreset } from '../apis/api'
import {
  agentRoleKeys,
  type AgentPresetConfig,
  type AgentRoleKey,
  type Preset,
  type PresetAgentConfigs,
} from '../apis/types'

type PresetFormDialogProps = {
  existingPresetIds: ReadonlyArray<string>
  mode: 'create' | 'edit'
  onCompleted: (result: { message: string; preset: Preset }) => Promise<void> | void
  onOpenChange: (open: boolean) => void
  open: boolean
  presetId?: string | null
}

type AgentFormState = {
  extra: string
  maxTokens: string
  temperature: string
}

type FormState = {
  agents: Record<AgentRoleKey, AgentFormState>
  displayName: string
  presetId: string
}

function createEmptyAgentState(): AgentFormState {
  return {
    extra: '',
    maxTokens: '',
    temperature: '',
  }
}

function createInitialState(): FormState {
  return {
    agents: {
      actor: createEmptyAgentState(),
      architect: createEmptyAgentState(),
      director: createEmptyAgentState(),
      keeper: createEmptyAgentState(),
      narrator: createEmptyAgentState(),
      planner: createEmptyAgentState(),
      replyer: createEmptyAgentState(),
    },
    displayName: '',
    presetId: '',
  }
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function parseAgentPresetConfig(
  roleKey: AgentRoleKey,
  agent: AgentFormState,
  t: (key: string, options?: Record<string, unknown>) => string,
  roleLabels: Record<AgentRoleKey, string>,
): AgentPresetConfig {
  let extra: unknown | null | undefined

  if (agent.extra.trim()) {
    try {
      extra = JSON.parse(agent.extra)
    } catch {
      throw new Error(
        t('presetsPage.form.errors.extraInvalid', {
          role: roleLabels[roleKey],
        }),
      )
    }
  }

  let temperature: number | undefined
  if (agent.temperature.trim()) {
    const parsed = Number(agent.temperature)
    if (!Number.isFinite(parsed)) {
      throw new Error(
        t('presetsPage.form.errors.temperatureInvalid', {
          role: roleLabels[roleKey],
        }),
      )
    }
    temperature = parsed
  }

  let maxTokens: number | undefined
  if (agent.maxTokens.trim()) {
    const parsed = Number(agent.maxTokens)
    if (!Number.isInteger(parsed) || parsed <= 0) {
      throw new Error(
        t('presetsPage.form.errors.maxTokensInvalid', {
          role: roleLabels[roleKey],
        }),
      )
    }
    maxTokens = parsed
  }

  return {
    ...(temperature !== undefined ? { temperature } : {}),
    ...(maxTokens !== undefined ? { max_tokens: maxTokens } : {}),
    ...(extra !== undefined ? { extra } : {}),
  }
}

function toPresetAgents(
  agents: FormState['agents'],
  t: (key: string, options?: Record<string, unknown>) => string,
  roleLabels: Record<AgentRoleKey, string>,
) {
  return Object.fromEntries(
    agentRoleKeys.map((roleKey) => [
      roleKey,
      parseAgentPresetConfig(roleKey, agents[roleKey], t, roleLabels),
    ]),
  ) as PresetAgentConfigs
}

export function PresetFormDialog({
  existingPresetIds,
  mode,
  onCompleted,
  onOpenChange,
  open,
  presetId,
}: PresetFormDialogProps) {
  const { t } = useTranslation()
  const translate = (key: string, options?: Record<string, unknown>) =>
    String(t(key as never, options as never))
  const [formState, setFormState] = useState<FormState>(createInitialState)
  const [isLoading, setIsLoading] = useState(false)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)
  useToastMessage(submitError)

  const roleLabels: Record<AgentRoleKey, string> = useMemo(
    () => ({
      actor: t('presetsPage.roles.actor'),
      architect: t('presetsPage.roles.architect'),
      director: t('presetsPage.roles.director'),
      keeper: t('presetsPage.roles.keeper'),
      narrator: t('presetsPage.roles.narrator'),
      planner: t('presetsPage.roles.planner'),
      replyer: t('presetsPage.roles.replyer'),
    }),
    [t],
  )

  useEffect(() => {
    if (!open) {
      setFormState(createInitialState())
      setIsLoading(false)
      setIsSubmitting(false)
      setSubmitError(null)
      return
    }

    if (mode !== 'edit' || !presetId) {
      setFormState(createInitialState())
      setSubmitError(null)
      return
    }

    const controller = new AbortController()
    setIsLoading(true)
    setSubmitError(null)

    void getPreset(presetId, controller.signal)
      .then((result) => {
        if (controller.signal.aborted) {
          return
        }

        setFormState({
          agents: Object.fromEntries(
            agentRoleKeys.map((roleKey) => [
              roleKey,
              {
                extra:
                  result.agents[roleKey].extra !== undefined && result.agents[roleKey].extra !== null
                    ? JSON.stringify(result.agents[roleKey].extra, null, 2)
                    : '',
                maxTokens: result.agents[roleKey].max_tokens?.toString() ?? '',
                temperature: result.agents[roleKey].temperature?.toString() ?? '',
              },
            ]),
          ) as FormState['agents'],
          displayName: result.display_name,
          presetId: result.preset_id,
        })
      })
      .catch((error) => {
        if (!controller.signal.aborted) {
          setSubmitError(getErrorMessage(error, t('presetsPage.feedback.loadPresetFailed')))
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
  }, [mode, open, presetId, t])

  function updateAgent(roleKey: AgentRoleKey, key: keyof AgentFormState, value: string) {
    setFormState((current) => ({
      ...current,
      agents: {
        ...current.agents,
        [roleKey]: {
          ...current.agents[roleKey],
          [key]: value,
        },
      },
    }))
  }

  async function handleSubmit() {
    if (!formState.presetId.trim()) {
      setSubmitError(t('presetsPage.form.errors.presetIdRequired'))
      return
    }

    if (mode === 'create' && existingPresetIds.includes(formState.presetId.trim())) {
      setSubmitError(t('presetsPage.form.errors.presetIdDuplicate'))
      return
    }

    if (!formState.displayName.trim()) {
      setSubmitError(t('presetsPage.form.errors.displayNameRequired'))
      return
    }

    setSubmitError(null)
    setIsSubmitting(true)

    try {
      const agents = toPresetAgents(formState.agents, translate, roleLabels)
      const preset =
        mode === 'create'
          ? await createPreset({
              agents,
              display_name: formState.displayName.trim(),
              preset_id: formState.presetId.trim(),
            })
          : await updatePreset({
              agents,
              display_name: formState.displayName.trim(),
              preset_id: formState.presetId.trim(),
            })

      await onCompleted({
        message:
          mode === 'create'
            ? t('presetsPage.feedback.created', { id: preset.display_name })
            : t('presetsPage.feedback.updated', { id: preset.display_name }),
        preset,
      })

      onOpenChange(false)
    } catch (error) {
      setSubmitError(getErrorMessage(error, t('presetsPage.form.errors.submitFailed')))
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,56rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>
            {mode === 'create' ? t('presetsPage.form.createTitle') : t('presetsPage.form.editTitle')}
          </DialogTitle>
        </DialogHeader>

        <DialogBody className="space-y-5 pt-6">
          {isLoading ? (
            <div className="space-y-4">
              <div className="h-12 animate-pulse rounded-[1.35rem] bg-[var(--color-bg-elevated)]" />
              <div className="grid gap-4 md:grid-cols-2">
                {Array.from({ length: agentRoleKeys.length }).map((_, index) => (
                  <div
                    className="h-40 animate-pulse rounded-[1.35rem] bg-[var(--color-bg-elevated)]"
                    key={index}
                  />
                ))}
              </div>
            </div>
          ) : (
            <>
              <div className="grid gap-4 md:grid-cols-2">
                <label className="space-y-2.5">
                  <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                    {t('presetsPage.form.fields.presetId')}
                  </span>
                  <Input
                    disabled={mode === 'edit'}
                    onChange={(event) => {
                      setFormState((current) => ({
                        ...current,
                        presetId: event.target.value,
                      }))
                    }}
                    placeholder={t('presetsPage.form.placeholders.presetId')}
                    value={formState.presetId}
                  />
                </label>

                <label className="space-y-2.5">
                  <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                    {t('presetsPage.form.fields.displayName')}
                  </span>
                  <Input
                    onChange={(event) => {
                      setFormState((current) => ({
                        ...current,
                        displayName: event.target.value,
                      }))
                    }}
                    placeholder={t('presetsPage.form.placeholders.displayName')}
                    value={formState.displayName}
                  />
                </label>
              </div>

              <div className="grid gap-4 md:grid-cols-2">
                {agentRoleKeys.map((roleKey) => (
                  <div
                    className="space-y-4 rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4"
                    key={roleKey}
                  >
                    <p className="text-sm font-medium text-[var(--color-text-primary)]">
                      {roleLabels[roleKey]}
                    </p>

                    <div className="grid gap-4 sm:grid-cols-2">
                      <label className="space-y-2">
                        <span className="block text-xs text-[var(--color-text-muted)]">
                          {t('presetsPage.form.fields.temperature')}
                        </span>
                        <Input
                          onChange={(event) => {
                            updateAgent(roleKey, 'temperature', event.target.value)
                          }}
                          placeholder={t('presetsPage.form.placeholders.temperature')}
                          value={formState.agents[roleKey].temperature}
                        />
                      </label>

                      <label className="space-y-2">
                        <span className="block text-xs text-[var(--color-text-muted)]">
                          {t('presetsPage.form.fields.maxTokens')}
                        </span>
                        <Input
                          onChange={(event) => {
                            updateAgent(roleKey, 'maxTokens', event.target.value)
                          }}
                          placeholder={t('presetsPage.form.placeholders.maxTokens')}
                          value={formState.agents[roleKey].maxTokens}
                        />
                      </label>
                    </div>

                    <label className="space-y-2">
                      <span className="block text-xs text-[var(--color-text-muted)]">
                        {t('presetsPage.form.fields.extra')}
                      </span>
                      <Textarea
                        className="min-h-[8rem]"
                        onChange={(event) => {
                          updateAgent(roleKey, 'extra', event.target.value)
                        }}
                        placeholder={t('presetsPage.form.placeholders.extra')}
                        value={formState.agents[roleKey].extra}
                      />
                    </label>
                  </div>
                ))}
              </div>
            </>
          )}
        </DialogBody>

        <DialogFooter className="justify-end gap-2">
          <Button onClick={() => onOpenChange(false)} variant="ghost">
            {t('presetsPage.actions.cancel')}
          </Button>
          <Button disabled={isLoading || isSubmitting} onClick={() => void handleSubmit()}>
            {isSubmitting ? t('presetsPage.actions.saving') : t('presetsPage.actions.save')}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
