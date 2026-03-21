import { faCopy } from '@fortawesome/free-solid-svg-icons/faCopy'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { PromptViewer } from '../../components/prompt-viewer'
import { buildPromptViewerCopyText } from '../../components/prompt-viewer-copy'
import { Badge } from '../../components/ui/badge'
import { Button } from '../../components/ui/button'
import {
  Dialog,
  DialogBody,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../../components/ui/dialog'
import { Select } from '../../components/ui/select'
import { Switch } from '../../components/ui/switch'
import { useToastNotice } from '../../components/ui/toast-context'
import { getPreset, previewPresetRuntime } from '../apis/api'
import type { AgentRoleKey, PresetDetail, PresetPromptPreview } from '../apis/types'
import { createPresetRoleLabels, getPromptModuleLabel } from '../presets/preset-labels'
import {
  buildPromptPreviewViewerMessages,
  getPromptPreviewRoleLabel,
} from '../presets/prompt-preview-shared'
import type { StageCopy } from './copy'

type Notice = {
  message: string
  tone: 'error' | 'success' | 'warning'
}

type PromptPreviewCharacterOption = {
  label: string
  value: string
}

type StagePromptPreviewDialogProps = {
  actorCharacterOptions: ReadonlyArray<PromptPreviewCharacterOption>
  copy: StageCopy
  onOpenChange: (open: boolean) => void
  open: boolean
  presetId: string
  sessionId: string
}

const stageRuntimePreviewAgents = [
  'director',
  'actor',
  'narrator',
  'keeper',
  'replyer',
] as const satisfies readonly AgentRoleKey[]

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

async function copyText(text: string) {
  if (typeof navigator !== 'undefined' && navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text)
    return
  }

  throw new Error('Clipboard API is unavailable')
}

function sortModules(modules: PresetDetail['agents'][AgentRoleKey]['modules']) {
  return [...modules].sort((left, right) => {
    if (left.order !== right.order) {
      return left.order - right.order
    }

    return left.module_id.localeCompare(right.module_id, 'zh-Hans-CN-u-co-pinyin')
  })
}

export function StagePromptPreviewDialog({
  actorCharacterOptions,
  copy,
  onOpenChange,
  open,
  presetId,
  sessionId,
}: StagePromptPreviewDialogProps) {
  const { t } = useTranslation()
  const translate = useCallback(
    (key: string, options?: Record<string, unknown>) => String(t(key as never, options as never)),
    [t],
  )
  const previewAbortRef = useRef<AbortController | null>(null)
  const presetAbortRef = useRef<AbortController | null>(null)
  const [notice, setNotice] = useState<Notice | null>(null)
  useToastNotice(notice)

  const roleLabels = useMemo(() => createPresetRoleLabels(translate), [translate])
  const [preset, setPreset] = useState<PresetDetail | null>(null)
  const [isPresetLoading, setIsPresetLoading] = useState(false)
  const [agent, setAgent] = useState<AgentRoleKey>('director')
  const [selectedModuleId, setSelectedModuleId] = useState('')
  const [selectedCharacterId, setSelectedCharacterId] = useState('')
  const [showEntryMarkers, setShowEntryMarkers] = useState(false)
  const [preview, setPreview] = useState<PresetPromptPreview | null>(null)
  const [isPreviewLoading, setIsPreviewLoading] = useState(false)

  const agentOptions = useMemo(
    () =>
      stageRuntimePreviewAgents.map((roleKey) => ({
        label: roleLabels[roleKey],
        value: roleKey,
      })),
    [roleLabels],
  )
  const agentModules = useMemo(
    () => (preset ? sortModules(preset.agents[agent].modules) : []),
    [agent, preset],
  )
  const moduleOptions = useMemo(
    () =>
      agentModules.map((module) => ({
        label: getPromptModuleLabel(translate, module.module_id, module.display_name),
        value: module.module_id,
      })),
    [agentModules, translate],
  )
  const characterOptions = useMemo(() => [...actorCharacterOptions], [actorCharacterOptions])
  const selectedModule = useMemo(
    () =>
      selectedModuleId
        ? (agentModules.find((module) => module.module_id === selectedModuleId) ?? null)
        : null,
    [agentModules, selectedModuleId],
  )
  const previewRoleLabel = useMemo(
    () => (preview ? getPromptPreviewRoleLabel(translate, preview.message_role) : null),
    [preview, translate],
  )
  const viewerMessages = useMemo(
    () =>
      buildPromptPreviewViewerMessages({
        preview,
        t: translate,
      }),
    [preview, translate],
  )
  const visibleViewerMessages = useMemo(
    () => viewerMessages.filter((message) => message.modules.length > 0),
    [viewerMessages],
  )
  const copyContent = useMemo(
    () =>
      buildPromptViewerCopyText({
        entryLabel: translate('presetsPage.preview.viewer.entry'),
        messages: visibleViewerMessages,
        moduleLabel: translate('presetsPage.preview.viewer.module'),
        noEntryContentLabel: translate('presetsPage.preview.viewer.noEntryContent'),
        showEntryMarkers,
      }),
    [showEntryMarkers, translate, visibleViewerMessages],
  )

  useEffect(() => {
    return () => {
      previewAbortRef.current?.abort()
      presetAbortRef.current?.abort()
    }
  }, [])

  useEffect(() => {
    if (!open) {
      setPreset(null)
      setAgent('director')
      setSelectedModuleId('')
      setSelectedCharacterId('')
      setShowEntryMarkers(false)
      setPreview(null)
      setIsPresetLoading(false)
      setIsPreviewLoading(false)
      previewAbortRef.current?.abort()
      presetAbortRef.current?.abort()
      return
    }

    const controller = new AbortController()
    presetAbortRef.current?.abort()
    presetAbortRef.current = controller
    setIsPresetLoading(true)
    setPreview(null)

    void getPreset(presetId, controller.signal)
      .then((result) => {
        if (!controller.signal.aborted) {
          setPreset(result)
        }
      })
      .catch((error) => {
        if (!controller.signal.aborted) {
          setNotice({
            message: getErrorMessage(error, copy.settings.bindings.preview.loadPresetFailed),
            tone: 'error',
          })
        }
      })
      .finally(() => {
        if (!controller.signal.aborted) {
          setIsPresetLoading(false)
        }
      })

    return () => {
      controller.abort()
    }
  }, [copy.settings.bindings.preview.loadPresetFailed, open, presetId])

  useEffect(() => {
    setPreview(null)
  }, [agent, selectedCharacterId, selectedModuleId])

  useEffect(() => {
    if (selectedModuleId && !agentModules.some((module) => module.module_id === selectedModuleId)) {
      setSelectedModuleId('')
    }
  }, [agentModules, selectedModuleId])

  useEffect(() => {
    if (agent !== 'actor') {
      setSelectedCharacterId('')
    }
  }, [agent])

  useEffect(() => {
    if (
      selectedCharacterId &&
      !characterOptions.some((character) => character.value === selectedCharacterId)
    ) {
      setSelectedCharacterId('')
    }
  }, [characterOptions, selectedCharacterId])

  async function handleGeneratePreview() {
    if (!preset) {
      setNotice({
        message: copy.settings.bindings.preview.loadPresetFailed,
        tone: 'error',
      })
      return
    }

    if (agent === 'actor' && !selectedCharacterId.trim()) {
      setNotice({
        message: copy.settings.bindings.preview.characterRequired,
        tone: 'warning',
      })
      return
    }

    previewAbortRef.current?.abort()
    const controller = new AbortController()
    previewAbortRef.current = controller
    setIsPreviewLoading(true)

    try {
      const nextPreview = await previewPresetRuntime(
        {
          agent,
          character_id: agent === 'actor' ? selectedCharacterId || undefined : undefined,
          module_id: selectedModuleId || undefined,
          preset_id: preset.preset_id,
        },
        {
          sessionId,
          signal: controller.signal,
        },
      )

      if (!controller.signal.aborted) {
        setPreview(nextPreview)
      }
    } catch (error) {
      if (!controller.signal.aborted) {
        setNotice({
          message: getErrorMessage(error, translate('presetsPage.preview.feedback.generateFailed')),
          tone: 'error',
        })
      }
    } finally {
      if (!controller.signal.aborted) {
        setIsPreviewLoading(false)
      }
    }
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent contentClassName="w-[min(92vw,46rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <div className="w-full space-y-3">
            <DialogTitle>{copy.settings.bindings.preview.title}</DialogTitle>
            {preset ? (
              <div className="flex flex-wrap items-center gap-2">
                <Badge className="normal-case" variant="subtle">
                  {preset.display_name}
                </Badge>
              </div>
            ) : null}
            <DialogDescription>{copy.settings.bindings.preview.description}</DialogDescription>
          </div>
        </DialogHeader>

        <DialogBody className="pt-6">
          {isPresetLoading ? (
            <div className="space-y-4">
              <div className="h-12 animate-pulse rounded-[1.2rem] bg-[var(--color-bg-panel)]" />
              <div className="h-12 animate-pulse rounded-[1.2rem] bg-[var(--color-bg-panel)]" />
              <div className="h-40 animate-pulse rounded-[1.2rem] bg-[var(--color-bg-panel)]" />
            </div>
          ) : preset ? (
            <div className="space-y-5">
              <div className="space-y-4">
                <div className={`grid gap-4 ${agent === 'actor' ? 'grid-cols-3' : 'grid-cols-2'}`}>
                  <div className="min-w-0 space-y-2">
                    <label
                      className="text-xs text-[var(--color-text-muted)]"
                      htmlFor="stage-preview-agent"
                    >
                      {translate('presetsPage.preview.fields.agent')}
                    </label>
                    <Select
                      items={agentOptions}
                      onValueChange={(value) => {
                        setAgent(value as AgentRoleKey)
                      }}
                      triggerId="stage-preview-agent"
                      value={agent}
                    />
                  </div>

                  <div className="min-w-0 space-y-2">
                    <label
                      className="text-xs text-[var(--color-text-muted)]"
                      htmlFor="stage-preview-module"
                    >
                      {translate('presetsPage.preview.fields.module')}
                    </label>
                    <Select
                      allowClear
                      clearLabel={translate('presetsPage.preview.allModules')}
                      items={moduleOptions}
                      onValueChange={setSelectedModuleId}
                      placeholder={translate('presetsPage.preview.allModules')}
                      triggerId="stage-preview-module"
                      value={selectedModuleId}
                    />
                  </div>

                  {agent === 'actor' ? (
                    <div className="min-w-0 space-y-2">
                      <label
                        className="text-xs text-[var(--color-text-muted)]"
                        htmlFor="stage-preview-character"
                      >
                        {translate('presetsPage.preview.fields.character')}
                      </label>
                      <Select
                        items={characterOptions}
                        onValueChange={setSelectedCharacterId}
                        placeholder={translate('presetsPage.preview.placeholders.character')}
                        triggerId="stage-preview-character"
                        value={selectedCharacterId}
                      />
                    </div>
                  ) : null}
                </div>

                <div className="flex justify-end">
                  <Button
                    disabled={isPreviewLoading}
                    onClick={() => {
                      void handleGeneratePreview()
                    }}
                    size="sm"
                    variant="secondary"
                  >
                    {isPreviewLoading
                      ? translate('presetsPage.preview.actions.generating')
                      : translate('presetsPage.preview.actions.generate')}
                  </Button>
                </div>
              </div>

              <div className="space-y-4">
                <div className="flex flex-wrap items-center gap-2">
                  <h4 className="text-sm font-medium text-[var(--color-text-primary)]">
                    {translate('presetsPage.preview.resultTitle')}
                  </h4>
                  {previewRoleLabel ? (
                    <Badge className="normal-case" variant="subtle">
                      {previewRoleLabel}
                    </Badge>
                  ) : null}
                  {selectedModule ? (
                    <Badge className="normal-case" variant="gold">
                      {getPromptModuleLabel(
                        translate,
                        selectedModule.module_id,
                        selectedModule.display_name,
                      )}
                    </Badge>
                  ) : null}
                </div>

                {preview?.unresolved_context_keys.length ? (
                  <div className="rounded-[1.2rem] border border-[var(--color-state-info-line)] bg-[var(--color-state-info-soft)] px-4 py-3">
                    <p className="text-sm text-[var(--color-text-primary)]">
                      {translate('presetsPage.preview.unresolvedTitle')}
                    </p>
                    <div className="mt-3 flex flex-wrap gap-2">
                      {preview.unresolved_context_keys.map((key) => (
                        <Badge className="normal-case" key={key} variant="info">
                          {key}
                        </Badge>
                      ))}
                    </div>
                  </div>
                ) : null}

                <div className="flex flex-wrap items-center justify-between gap-3 rounded-[1.2rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-3">
                  <label className="flex items-center gap-3 text-sm text-[var(--color-text-secondary)]">
                    <Switch
                      aria-label={translate('presetsPage.preview.viewer.showEntryMarkers')}
                      checked={showEntryMarkers}
                      onCheckedChange={setShowEntryMarkers}
                      size="sm"
                    />
                    <span>{translate('presetsPage.preview.viewer.showEntryMarkers')}</span>
                  </label>

                  <Button
                    className="gap-2"
                    disabled={copyContent.length === 0}
                    onClick={() => {
                      void copyText(copyContent)
                        .then(() => {
                          setNotice({
                            message: translate('presetsPage.preview.feedback.copied'),
                            tone: 'success',
                          })
                        })
                        .catch(() => {
                          setNotice({
                            message: translate('presetsPage.preview.feedback.copyFailed'),
                            tone: 'error',
                          })
                        })
                    }}
                    size="sm"
                    variant="ghost"
                  >
                    <FontAwesomeIcon icon={faCopy} />
                    {translate('presetsPage.preview.actions.copy')}
                  </Button>
                </div>

                {isPreviewLoading ? (
                  <div className="space-y-4">
                    <div className="h-12 animate-pulse rounded-[1.2rem] bg-[var(--color-bg-panel)]" />
                    <div className="h-40 animate-pulse rounded-[1.2rem] bg-[var(--color-bg-panel)]" />
                  </div>
                ) : (
                  <PromptViewer
                    emptyLabel={translate('presetsPage.preview.empty')}
                    messages={viewerMessages}
                    noEntryContentLabel={translate('presetsPage.preview.viewer.noEntryContent')}
                    showEntryMarkers={showEntryMarkers}
                    syntheticEntryLabel={translate('presetsPage.preview.viewer.synthetic')}
                  />
                )}
              </div>
            </div>
          ) : (
            <div className="rounded-[1.35rem] border border-dashed border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-5 py-8 text-sm text-[var(--color-text-muted)]">
              {copy.settings.bindings.preview.loadPresetFailed}
            </div>
          )}
        </DialogBody>

        <DialogFooter className="justify-start">
          <DialogClose asChild>
            <Button variant="secondary">{translate('presetsPage.preview.actions.close')}</Button>
          </DialogClose>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
