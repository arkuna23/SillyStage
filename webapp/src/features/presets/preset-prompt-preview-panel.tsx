import { useEffect, useMemo, useRef, useState } from 'react'
import { faCopy } from '@fortawesome/free-solid-svg-icons/faCopy'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'

import { PromptViewer } from '../../components/prompt-viewer'
import { buildPromptViewerCopyText } from '../../components/prompt-viewer-copy'
import { Badge } from '../../components/ui/badge'
import { Button } from '../../components/ui/button'
import { Select } from '../../components/ui/select'
import { Switch } from '../../components/ui/switch'
import { useToastNotice } from '../../components/ui/toast-context'
import { previewPresetTemplate } from '../apis/api'
import type {
  AgentRoleKey,
  ArchitectPromptMode,
  PresetDetail,
  PresetPromptPreview,
} from '../apis/types'
import {
  buildPromptPreviewViewerMessages,
  getPromptPreviewArchitectModeLabel,
  getPromptPreviewRoleLabel,
  type PromptPreviewTranslateFn,
} from './prompt-preview-shared'
import {
  createPresetRoleLabels,
  getOrderedAgentRoleKeys,
  getPromptModuleLabel,
} from './preset-labels'

type Notice = {
  message: string
  tone: 'error' | 'success' | 'warning'
}

type PresetPromptPreviewPanelProps = {
  initialAgent?: AgentRoleKey
  initialModuleId?: string | null
  preset: PresetDetail
  t: PromptPreviewTranslateFn
}

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

export function PresetPromptPreviewPanel({
  initialAgent,
  initialModuleId,
  preset,
  t,
}: PresetPromptPreviewPanelProps) {
  const previewAbortRef = useRef<AbortController | null>(null)
  const [notice, setNotice] = useState<Notice | null>(null)
  useToastNotice(notice)

  const roleLabels = useMemo(() => createPresetRoleLabels(t), [t])
  const [agent, setAgent] = useState<AgentRoleKey>(initialAgent ?? 'planner')
  const [selectedModuleId, setSelectedModuleId] = useState(initialModuleId?.trim() ?? '')
  const [architectMode, setArchitectMode] = useState<ArchitectPromptMode>('graph')
  const [showEntryMarkers, setShowEntryMarkers] = useState(false)
  const [preview, setPreview] = useState<PresetPromptPreview | null>(null)
  const [isPreviewLoading, setIsPreviewLoading] = useState(false)

  const agentModules = useMemo(
    () => sortModules(preset.agents[agent].modules),
    [agent, preset.agents],
  )
  const selectedModule = useMemo(
    () =>
      selectedModuleId
        ? agentModules.find((module) => module.module_id === selectedModuleId) ?? null
        : null,
    [agentModules, selectedModuleId],
  )
  const agentOptions = useMemo(
    () =>
      getOrderedAgentRoleKeys().map((roleKey) => ({
        label: roleLabels[roleKey],
        value: roleKey,
      })),
    [roleLabels],
  )
  const moduleOptions = useMemo(
    () =>
      agentModules.map((module) => ({
        label: getPromptModuleLabel(t, module.module_id, module.display_name),
        value: module.module_id,
      })),
    [agentModules, t],
  )
  const previewRoleLabel = useMemo(
    () => (preview ? getPromptPreviewRoleLabel(t, preview.message_role) : null),
    [preview, t],
  )
  const viewerMessages = useMemo(
    () =>
      buildPromptPreviewViewerMessages({
        preview,
        t,
      }),
    [preview, t],
  )
  const visibleViewerMessages = useMemo(
    () => viewerMessages.filter((message) => message.modules.length > 0),
    [viewerMessages],
  )
  const copyContent = useMemo(
    () =>
      buildPromptViewerCopyText({
        entryLabel: t('presetsPage.preview.viewer.entry'),
        messages: visibleViewerMessages,
        moduleLabel: t('presetsPage.preview.viewer.module'),
        noEntryContentLabel: t('presetsPage.preview.viewer.noEntryContent'),
        showEntryMarkers,
      }),
    [showEntryMarkers, t, visibleViewerMessages],
  )

  useEffect(() => {
    return () => {
      previewAbortRef.current?.abort()
    }
  }, [])

  useEffect(() => {
    setPreview(null)
  }, [agent, architectMode, selectedModuleId])

  useEffect(() => {
    if (selectedModuleId && !agentModules.some((module) => module.module_id === selectedModuleId)) {
      setSelectedModuleId('')
    }
  }, [agentModules, selectedModuleId])

  async function handleGeneratePreview() {
    previewAbortRef.current?.abort()
    const controller = new AbortController()
    previewAbortRef.current = controller
    setIsPreviewLoading(true)

    try {
      const nextPreview = await previewPresetTemplate(
        {
          agent,
          architect_mode: agent === 'architect' ? architectMode : undefined,
          module_id: selectedModuleId || undefined,
          preset_id: preset.preset_id,
        },
        controller.signal,
      )

      if (!controller.signal.aborted) {
        setPreview(nextPreview)
      }
    } catch (error) {
      if (!controller.signal.aborted) {
        setNotice({
          message: getErrorMessage(error, t('presetsPage.preview.feedback.generateFailed')),
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
    <div className="space-y-5">
      <div className="space-y-4">
        <div className={`grid gap-4 ${agent === 'architect' ? 'grid-cols-3' : 'grid-cols-2'}`}>
          <div className="min-w-0 space-y-2">
            <label className="text-xs text-[var(--color-text-muted)]" htmlFor="preset-preview-agent">
              {t('presetsPage.preview.fields.agent')}
            </label>
            <Select
              items={agentOptions}
              onValueChange={(value) => {
                setAgent(value as AgentRoleKey)
              }}
              triggerId="preset-preview-agent"
              value={agent}
            />
          </div>

          <div className="min-w-0 space-y-2">
            <label className="text-xs text-[var(--color-text-muted)]" htmlFor="preset-preview-module">
              {t('presetsPage.preview.fields.module')}
            </label>
            <Select
              allowClear
              clearLabel={t('presetsPage.preview.allModules')}
              items={moduleOptions}
              onValueChange={setSelectedModuleId}
              placeholder={t('presetsPage.preview.allModules')}
              triggerId="preset-preview-module"
              value={selectedModuleId}
            />
          </div>

          {agent === 'architect' ? (
            <div className="min-w-0 space-y-2">
              <label className="text-xs text-[var(--color-text-muted)]" htmlFor="preset-preview-architect-mode">
                {t('presetsPage.preview.fields.architectMode')}
              </label>
              <Select
                items={[
                  {
                    label: getPromptPreviewArchitectModeLabel(t, 'graph'),
                    value: 'graph',
                  },
                  {
                    label: getPromptPreviewArchitectModeLabel(t, 'draft_init'),
                    value: 'draft_init',
                  },
                  {
                    label: getPromptPreviewArchitectModeLabel(t, 'draft_continue'),
                    value: 'draft_continue',
                  },
                ]}
                onValueChange={(value) => {
                  if (value === 'graph' || value === 'draft_init' || value === 'draft_continue') {
                    setArchitectMode(value)
                  }
                }}
                triggerId="preset-preview-architect-mode"
                value={architectMode}
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
              ? t('presetsPage.preview.actions.generating')
              : t('presetsPage.preview.actions.generate')}
          </Button>
        </div>
      </div>

      <div className="space-y-4">
        <div className="flex flex-wrap items-center gap-2">
          <h4 className="text-sm font-medium text-[var(--color-text-primary)]">
            {t('presetsPage.preview.resultTitle')}
          </h4>
          {previewRoleLabel ? (
            <Badge className="normal-case" variant="subtle">
              {previewRoleLabel}
            </Badge>
          ) : null}
          {selectedModule ? (
            <Badge className="normal-case" variant="gold">
              {getPromptModuleLabel(t, selectedModule.module_id, selectedModule.display_name)}
            </Badge>
          ) : null}
        </div>

        {preview?.unresolved_context_keys.length ? (
          <div className="rounded-[1.2rem] border border-[var(--color-state-info-line)] bg-[var(--color-state-info-soft)] px-4 py-3">
            <p className="text-sm text-[var(--color-text-primary)]">
              {t('presetsPage.preview.unresolvedTitle')}
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
              aria-label={t('presetsPage.preview.viewer.showEntryMarkers')}
              checked={showEntryMarkers}
              onCheckedChange={setShowEntryMarkers}
              size="sm"
            />
            <span>{t('presetsPage.preview.viewer.showEntryMarkers')}</span>
          </label>

          <Button
            className="gap-2"
            disabled={copyContent.length === 0}
            onClick={() => {
              void copyText(copyContent)
                .then(() => {
                  setNotice({
                    message: t('presetsPage.preview.feedback.copied'),
                    tone: 'success',
                  })
                })
                .catch(() => {
                  setNotice({
                    message: t('presetsPage.preview.feedback.copyFailed'),
                    tone: 'error',
                  })
                })
            }}
            size="sm"
            variant="ghost"
          >
            <FontAwesomeIcon icon={faCopy} />
            {t('presetsPage.preview.actions.copy')}
          </Button>
        </div>

        {isPreviewLoading ? (
          <div className="space-y-4">
            <div className="h-12 animate-pulse rounded-[1.2rem] bg-[var(--color-bg-panel)]" />
            <div className="h-40 animate-pulse rounded-[1.2rem] bg-[var(--color-bg-panel)]" />
          </div>
        ) : (
          <PromptViewer
            emptyLabel={t('presetsPage.preview.empty')}
            messages={viewerMessages}
            noEntryContentLabel={t('presetsPage.preview.viewer.noEntryContent')}
            showEntryMarkers={showEntryMarkers}
            syntheticEntryLabel={t('presetsPage.preview.viewer.synthetic')}
          />
        )}
      </div>
    </div>
  )
}
