import { faBookOpen } from '@fortawesome/free-solid-svg-icons/faBookOpen'
import { faCheckDouble } from '@fortawesome/free-solid-svg-icons/faCheckDouble'
import { faDiagramProject } from '@fortawesome/free-solid-svg-icons/faDiagramProject'
import { faPlus } from '@fortawesome/free-solid-svg-icons/faPlus'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { AnimatePresence, motion, useReducedMotion } from 'framer-motion'
import { type ReactNode, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { Badge } from '../../components/ui/badge'
import { Button } from '../../components/ui/button'
import { Input } from '../../components/ui/input'
import { Select, type SelectOption } from '../../components/ui/select'
import { Textarea } from '../../components/ui/textarea'
import { cn } from '../../lib/cn'
import type { CharacterSummary } from '../characters/types'
import {
  type StoryCommonVariableKeySource,
  useStoryCommonVariableSchemaCatalog,
} from './story-common-variable-schema-catalog'
import {
  StoryGraphCharacterMultiSelect,
  StoryGraphCharacterSelect,
} from './story-graph-character-select'
import { StoryGraphCollapsibleCard } from './story-graph-collapsible-card'
import {
  buildConditionDraftKey,
  type GraphConditionDrafts,
  type GraphOnEnterUpdateDrafts,
  getGraphNodeLabel,
} from './story-graph-editor-utils'
import { StoryGraphOnEnterUpdatesEditor } from './story-graph-on-enter-updates-editor'
import type {
  ConditionOperator,
  ConditionScope,
  StoryGraph,
  StoryGraphCondition,
  StoryGraphNode,
  StoryGraphStateOpType,
  StoryGraphTransition,
} from './types'

type StoryGraphInspectorProps = {
  availableCharacters: ReadonlyArray<CharacterSummary>
  conditionDrafts: GraphConditionDrafts
  graph: StoryGraph
  newNodeIds: Set<string>
  onAddOnEnterUpdate: (nodeId: string) => void
  onAddTransition: (nodeId: string) => void
  onConditionDraftChange: (nodeId: string, transitionIndex: number, value: string) => void
  onDeleteNode: (nodeId: string) => void
  onOnEnterUpdateDraftChange: (nodeId: string, operationIndex: number, value: string) => void
  onRemoveTransition: (nodeId: string, transitionIndex: number) => void
  onRemoveOnEnterUpdate: (nodeId: string, operationIndex: number) => void
  onSelectTransition: (transitionIndex: number | null) => void
  onSetStartNode: (nodeId: string) => void
  onToggleCondition: (nodeId: string, transitionIndex: number, enabled: boolean) => void
  onUpdateCharacters: (nodeId: string, value: string[]) => void
  onUpdateNodeField: (nodeId: string, field: 'goal' | 'scene' | 'title', value: string) => void
  onUpdateNodeId: (nodeId: string, value: string) => void
  onUpdateOnEnterUpdate: (
    nodeId: string,
    operationIndex: number,
    patch: {
      character?: string
      characters?: string[]
      key?: string
      node_id?: string
      type?: StoryGraphStateOpType
      value?: unknown
    },
  ) => void
  onUpdateTransition: (
    nodeId: string,
    transitionIndex: number,
    patch: Partial<StoryGraphTransition>,
  ) => void
  onEnterUpdateDrafts: GraphOnEnterUpdateDrafts
  onUpdateTransitionCondition: (
    nodeId: string,
    transitionIndex: number,
    patch: Partial<StoryGraphCondition>,
  ) => void
  playerSchemaId?: string | null
  readOnly?: boolean
  selectedNodeId: string | null
  selectedTransitionIndex: number | null
  worldSchemaId?: string | null
}

type SelectedNodeInspectorContentProps = Omit<StoryGraphInspectorProps, 'selectedNodeId'> & {
  activeTab: InspectorTab
  onActiveTabChange: (nextValue: InspectorTab) => void
  selectedNode: StoryGraphNode
}

type InspectorTab = 'node' | 'transitions' | 'updates'

const conditionScopeItems: Array<{ label: string; value: ConditionScope }> = [
  { label: 'Global', value: 'global' },
  { label: 'Player', value: 'player' },
  { label: 'Character', value: 'character' },
]

const conditionOperatorItems: Array<{ label: string; value: ConditionOperator }> = [
  { label: 'eq', value: 'eq' },
  { label: 'ne', value: 'ne' },
  { label: 'gt', value: 'gt' },
  { label: 'gte', value: 'gte' },
  { label: 'lt', value: 'lt' },
  { label: 'lte', value: 'lte' },
  { label: 'contains', value: 'contains' },
]

function buildTransitionExpansionKey(nodeId: string, transitionIndex: number) {
  return `${nodeId}:${transitionIndex}`
}

function buildOnEnterUpdateExpansionKey(nodeId: string, operationIndex: number) {
  return `${nodeId}:op:${operationIndex}`
}

function Field({ children, label }: { children: ReactNode; label: string }) {
  return (
    <label className="block space-y-2">
      <span className="text-xs text-[var(--color-text-muted)]">{label}</span>
      {children}
    </label>
  )
}

function ensureCurrentValueOption(
  items: ReadonlyArray<SelectOption>,
  value: string,
  fallbackLabel: string,
) {
  if (!value.trim() || items.some((item) => item.value === value)) {
    return [...items]
  }

  return [
    {
      label: `${value} · ${fallbackLabel}`,
      value,
    },
    ...items,
  ]
}

function getKeySourceHint(args: {
  needsCharacterSelection?: boolean
  source: StoryCommonVariableKeySource | null
  translate: (key: string) => string
}) {
  if (args.needsCharacterSelection) {
    return args.translate('stories.graph.keySourceSelectCharacterFirst')
  }

  if (!args.source) {
    return args.translate('stories.graph.keySourceMissing')
  }

  if (args.source.status === 'loading') {
    return args.translate('stories.graph.keySourceLoading')
  }

  if (args.source.status === 'error') {
    return args.translate('stories.graph.keySourceLoadFailed')
  }

  if (args.source.status === 'missing') {
    return args.translate('stories.graph.keySourceMissing')
  }

  if (args.source.items.length === 0) {
    return args.translate('stories.graph.keySourceEmpty')
  }

  return null
}

function InspectorTabButton({
  active,
  icon,
  id,
  label,
  onClick,
  panelId,
}: {
  active: boolean
  icon: ReactNode
  id: string
  label: string
  onClick: () => void
  panelId: string
}) {
  return (
    <button
      aria-controls={panelId}
      aria-label={label}
      aria-selected={active}
      className={cn(
        'inline-flex size-10 items-center justify-center rounded-[0.95rem] border transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-focus-ring)]',
        active
          ? 'border-[var(--color-accent-gold-line)] bg-[linear-gradient(135deg,color-mix(in_srgb,var(--color-accent-gold)_88%,var(--color-bg-curtain)),color-mix(in_srgb,var(--color-accent-gold-strong)_82%,var(--color-bg-curtain)))] text-[color:var(--color-accent-ink)] shadow-[0_10px_24px_var(--color-accent-glow-soft)]'
          : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-panel-strong)] text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]',
      )}
      id={id}
      onClick={onClick}
      role="tab"
      title={label}
      type="button"
    >
      <span aria-hidden="true" className="inline-flex items-center justify-center text-sm">
        {icon}
      </span>
    </button>
  )
}

function SelectedNodeInspectorContent({
  activeTab,
  availableCharacters,
  conditionDrafts,
  graph,
  newNodeIds,
  onActiveTabChange,
  onAddOnEnterUpdate,
  onAddTransition,
  onConditionDraftChange,
  onDeleteNode,
  onOnEnterUpdateDraftChange,
  onRemoveTransition,
  onRemoveOnEnterUpdate,
  onSelectTransition,
  onSetStartNode,
  onToggleCondition,
  onUpdateCharacters,
  onUpdateNodeField,
  onUpdateNodeId,
  onUpdateOnEnterUpdate,
  onUpdateTransition,
  onEnterUpdateDrafts,
  onUpdateTransitionCondition,
  playerSchemaId,
  readOnly = false,
  selectedNode,
  selectedTransitionIndex,
  worldSchemaId,
}: SelectedNodeInspectorContentProps) {
  const { t } = useTranslation()
  const prefersReducedMotion = useReducedMotion()
  const [expandedTransitionKeys, setExpandedTransitionKeys] = useState<Set<string>>(new Set())
  const [expandedOnEnterUpdateKeys, setExpandedOnEnterUpdateKeys] = useState<Set<string>>(new Set())

  const duplicateId = newNodeIds.has(selectedNode.id)
    ? graph.nodes.filter((node) => node.id === selectedNode.id).length > 1
    : false

  const targetItems = graph.nodes.map((node) => ({
    label: `${getGraphNodeLabel(node)} (${node.id})`,
    value: node.id,
  }))
  const referencedCharacterIds = useMemo(
    () =>
      Array.from(
        new Set(
          [
            ...selectedNode.characters,
            ...selectedNode.transitions
              .map((transition) => transition.condition?.character?.trim() ?? '')
              .filter((characterId) => characterId.length > 0),
            ...(selectedNode.on_enter_updates ?? [])
              .flatMap((operation) =>
                'character' in operation ? [operation.character.trim()] : [],
              )
              .filter((characterId) => characterId.length > 0),
          ].filter((characterId) => characterId.length > 0),
        ),
      ),
    [selectedNode],
  )
  const schemaCatalog = useStoryCommonVariableSchemaCatalog({
    characterIds: referencedCharacterIds,
    enabled: true,
    playerSchemaId,
    worldSchemaId,
  })
  const onEnterUpdatesCount = selectedNode.on_enter_updates?.length ?? 0
  const tabItems = [
    {
      icon: <FontAwesomeIcon fixedWidth icon={faBookOpen} />,
      label: t('stories.graph.nodeSection'),
      value: 'node' as const,
    },
    {
      icon: <FontAwesomeIcon fixedWidth icon={faDiagramProject} />,
      label: t('stories.graph.transitions'),
      value: 'transitions' as const,
    },
    {
      icon: <FontAwesomeIcon fixedWidth icon={faCheckDouble} />,
      label: t('stories.graph.onEnterUpdatesTitle'),
      value: 'updates' as const,
    },
  ]

  const activeTabTitle =
    activeTab === 'node'
      ? t('stories.graph.nodeSection')
      : activeTab === 'transitions'
        ? t('stories.graph.transitions')
        : t('stories.graph.onEnterUpdatesTitle')
  const activeTabSubtitle =
    activeTab === 'transitions'
      ? t('stories.graph.transitionsCount', { count: selectedNode.transitions.length })
      : activeTab === 'updates'
        ? t('stories.graph.onEnterUpdatesCount', { count: onEnterUpdatesCount })
        : selectedNode.id
  const activeTabAction =
    activeTab === 'transitions' && !readOnly ? (
      <Button
        onClick={() => {
          const nextTransitionIndex = selectedNode.transitions.length
          const transitionKey = buildTransitionExpansionKey(selectedNode.id, nextTransitionIndex)

          onAddTransition(selectedNode.id)
          setExpandedTransitionKeys((currentKeys) => new Set(currentKeys).add(transitionKey))
        }}
        size="sm"
        variant="secondary"
      >
        <FontAwesomeIcon icon={faPlus} />
        {t('stories.graph.addTransition' as const)}
      </Button>
    ) : activeTab === 'updates' && !readOnly ? (
      <Button
        onClick={() => {
          const nextOperationIndex = onEnterUpdatesCount
          const nextOperationKey = buildOnEnterUpdateExpansionKey(
            selectedNode.id,
            nextOperationIndex,
          )

          onAddOnEnterUpdate(selectedNode.id)
          setExpandedOnEnterUpdateKeys((currentKeys) => new Set(currentKeys).add(nextOperationKey))
        }}
        size="sm"
        variant="secondary"
      >
        <FontAwesomeIcon icon={faPlus} />
        {t('stories.graph.addOnEnterUpdate')}
      </Button>
    ) : null

  const activePanelId = `story-graph-inspector-panel-${selectedNode.id}-${activeTab}`
  const activeTabId = `story-graph-inspector-tab-${selectedNode.id}-${activeTab}`

  return (
    <div className="scrollbar-none flex h-full min-h-0 flex-col overflow-y-auto px-6 py-6">
      <div className="rounded-[1.4rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] p-4">
        <div className="flex items-start justify-between gap-4">
          <div className="min-w-0 space-y-2">
            <div className="flex flex-wrap items-center gap-2">
              <h3 className="text-lg font-semibold text-[var(--color-text-primary)]">
                {getGraphNodeLabel(selectedNode)}
              </h3>
              {graph.start_node === selectedNode.id ? (
                <Badge className="px-2.5 py-1" variant="info">
                  {t('stories.graph.start')}
                </Badge>
              ) : null}
              {readOnly ? (
                <Badge className="px-2.5 py-1" variant="gold">
                  {t('stories.graph.readOnly')}
                </Badge>
              ) : null}
            </div>
            <p className="text-sm text-[var(--color-text-secondary)]">{selectedNode.id}</p>
          </div>

          {!readOnly ? (
            <div className="flex shrink-0 items-center gap-2">
              {graph.start_node !== selectedNode.id ? (
                <Button
                  onClick={() => {
                    onSetStartNode(selectedNode.id)
                  }}
                  size="sm"
                  variant="ghost"
                >
                  {t('stories.graph.setStart')}
                </Button>
              ) : null}
              <Button
                onClick={() => {
                  onDeleteNode(selectedNode.id)
                }}
                size="sm"
                variant="danger"
              >
                {t('stories.graph.deleteNode')}
              </Button>
            </div>
          ) : null}
        </div>
      </div>

      <div className="mt-4 rounded-[1.4rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] p-4">
        <div className="flex items-center justify-between gap-4">
          <div className="min-w-0 flex-1 self-center">
            <h4 className="text-sm font-medium text-[var(--color-text-primary)]">
              {activeTabTitle}
            </h4>
            <p className="mt-1 text-xs leading-6 text-[var(--color-text-muted)]">
              {activeTabSubtitle}
            </p>
          </div>

          <div
            aria-label={t('stories.graph.title')}
            className="inline-flex shrink-0 items-center gap-2 rounded-[1.2rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel-strong)] p-1 self-center"
            role="tablist"
          >
            {tabItems.map((item) => {
              const tabId = `story-graph-inspector-tab-${selectedNode.id}-${item.value}`
              const panelId = `story-graph-inspector-panel-${selectedNode.id}-${item.value}`

              return (
                <InspectorTabButton
                  active={item.value === activeTab}
                  icon={item.icon}
                  id={tabId}
                  key={item.value}
                  label={item.label}
                  onClick={() => {
                    onActiveTabChange(item.value)
                  }}
                  panelId={panelId}
                />
              )
            })}
          </div>
        </div>

        <AnimatePresence initial={false} mode="wait">
          <motion.div
            animate={prefersReducedMotion ? { opacity: 1 } : { opacity: 1, y: 0 }}
            aria-labelledby={activeTabId}
            className="mt-5"
            exit={prefersReducedMotion ? { opacity: 0 } : { opacity: 0, y: -6 }}
            id={activePanelId}
            initial={prefersReducedMotion ? { opacity: 0 } : { opacity: 0, y: 8 }}
            key={activeTab}
            role="tabpanel"
            transition={{ duration: prefersReducedMotion ? 0 : 0.18, ease: [0.22, 1, 0.36, 1] }}
          >
            <div className="space-y-4">
              {activeTabAction ? <div className="flex justify-end">{activeTabAction}</div> : null}

              {activeTab === 'node' ? (
                <div className="space-y-4">
                  <Field label={t('stories.graph.nodeId')}>
                    <Input
                      id={`story-graph-node-id-${selectedNode.id}`}
                      name={`story-graph-node-id-${selectedNode.id}`}
                      onChange={(event) => {
                        onUpdateNodeId(selectedNode.id, event.target.value)
                      }}
                      readOnly={readOnly || !newNodeIds.has(selectedNode.id)}
                      value={selectedNode.id}
                    />
                  </Field>
                  {duplicateId ? (
                    <p className="text-xs text-[var(--color-state-error-text)]">
                      {t('stories.graph.errors.duplicateNodeId')}
                    </p>
                  ) : null}

                  <Field label={t('stories.graph.nodeTitle')}>
                    <Input
                      id={`story-graph-node-title-${selectedNode.id}`}
                      name={`story-graph-node-title-${selectedNode.id}`}
                      onChange={(event) => {
                        onUpdateNodeField(selectedNode.id, 'title', event.target.value)
                      }}
                      readOnly={readOnly}
                      value={selectedNode.title}
                    />
                  </Field>

                  <Field label={t('stories.graph.scene')}>
                    <Textarea
                      id={`story-graph-node-scene-${selectedNode.id}`}
                      name={`story-graph-node-scene-${selectedNode.id}`}
                      onChange={(event) => {
                        onUpdateNodeField(selectedNode.id, 'scene', event.target.value)
                      }}
                      readOnly={readOnly}
                      rows={4}
                      value={selectedNode.scene}
                    />
                  </Field>

                  <Field label={t('stories.graph.goal')}>
                    <Textarea
                      id={`story-graph-node-goal-${selectedNode.id}`}
                      name={`story-graph-node-goal-${selectedNode.id}`}
                      onChange={(event) => {
                        onUpdateNodeField(selectedNode.id, 'goal', event.target.value)
                      }}
                      readOnly={readOnly}
                      rows={4}
                      value={selectedNode.goal}
                    />
                  </Field>

                  <Field label={t('stories.graph.characters')}>
                    <StoryGraphCharacterMultiSelect
                      characters={availableCharacters}
                      disabled={readOnly}
                      onChange={(nextCharacterIds) => {
                        onUpdateCharacters(selectedNode.id, nextCharacterIds)
                      }}
                      selectedCharacterIds={selectedNode.characters}
                    />
                  </Field>
                </div>
              ) : null}

              {activeTab === 'transitions' ? (
                <div className="space-y-4">
                  {selectedNode.transitions.length === 0 ? (
                    <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
                      {t('stories.graph.emptyTransitions')}
                    </p>
                  ) : null}

                  <div className="space-y-4">
                    {selectedNode.transitions.map((transition, transitionIndex) => {
                      const transitionDraftKey = buildConditionDraftKey(
                        selectedNode.id,
                        transitionIndex,
                      )
                      const transitionCardKey = buildTransitionExpansionKey(
                        selectedNode.id,
                        transitionIndex,
                      )
                      const conditionDraft =
                        conditionDrafts[transitionDraftKey] ??
                        JSON.stringify(transition.condition?.value ?? '', null, 2)
                      const targetNode =
                        graph.nodes.find((node) => node.id === transition.to) ?? null
                      const conditionValueError =
                        transition.condition && conditionDraft
                          ? (() => {
                              try {
                                JSON.parse(conditionDraft)
                                return null
                              } catch {
                                return t('stories.graph.errors.invalidConditionValue')
                              }
                            })()
                          : null
                      const conditionCharacterId = transition.condition?.character?.trim() ?? ''
                      const conditionKeySource =
                        transition.condition?.scope === 'character'
                          ? conditionCharacterId
                            ? (schemaCatalog.characterByCharacterId[conditionCharacterId] ?? null)
                            : null
                          : transition.condition?.scope === 'player'
                            ? schemaCatalog.player
                            : schemaCatalog.world
                      const conditionKeyHint = getKeySourceHint({
                        needsCharacterSelection:
                          transition.condition?.scope === 'character' && !conditionCharacterId,
                        source: conditionKeySource,
                        translate: (key) => t(key as never),
                      })
                      const conditionKeyItems = ensureCurrentValueOption(
                        conditionKeySource?.items ?? [],
                        transition.condition?.key ?? '',
                        t('stories.graph.currentKey'),
                      )
                      const isConditionKeyDisabled =
                        readOnly ||
                        (transition.condition?.scope === 'character' && !conditionCharacterId) ||
                        conditionKeySource?.status !== 'ready'

                      return (
                        <StoryGraphCollapsibleCard
                          action={
                            !readOnly ? (
                              <Button
                                onClick={() => {
                                  onRemoveTransition(selectedNode.id, transitionIndex)
                                }}
                                size="sm"
                                variant="ghost"
                              >
                                {t('stories.graph.removeTransition')}
                              </Button>
                            ) : null
                          }
                          className={cn(
                            'rounded-[1.2rem] bg-[color-mix(in_srgb,var(--color-bg-panel-strong)_75%,transparent)]',
                            selectedTransitionIndex === transitionIndex
                              ? 'border-[var(--color-accent-gold-line)] ring-1 ring-[var(--color-focus-ring)]'
                              : 'border-[var(--color-border-subtle)]',
                          )}
                          contentClassName="space-y-4"
                          key={`${selectedNode.id}:${transitionIndex}`}
                          onToggle={() => {
                            onSelectTransition(transitionIndex)
                            setExpandedTransitionKeys((currentKeys) => {
                              const nextKeys = new Set(currentKeys)

                              if (nextKeys.has(transitionCardKey)) {
                                nextKeys.delete(transitionCardKey)
                              } else {
                                nextKeys.add(transitionCardKey)
                              }

                              return nextKeys
                            })
                          }}
                          open={expandedTransitionKeys.has(transitionCardKey)}
                          subtitle={
                            <span className="block truncate">
                              {targetNode
                                ? `${getGraphNodeLabel(targetNode)} (${targetNode.id})`
                                : transition.to || '—'}{' '}
                              ·{' '}
                              {transition.condition
                                ? t('stories.graph.conditionEnabled')
                                : t('stories.graph.conditionDisabled')}
                            </span>
                          }
                          title={t('stories.graph.transitionLabel', { index: transitionIndex + 1 })}
                        >
                          <Field label={t('stories.graph.transitionTarget')}>
                            <Select
                              disabled={readOnly}
                              items={targetItems}
                              onValueChange={(nextValue) => {
                                onUpdateTransition(selectedNode.id, transitionIndex, {
                                  to: nextValue,
                                })
                              }}
                              textAlign="start"
                              value={transition.to}
                            />
                          </Field>

                          <div className="flex items-center justify-between gap-3 rounded-[1.1rem] border border-[var(--color-border-subtle)] px-3 py-3">
                            <div>
                              <p className="text-sm font-medium text-[var(--color-text-primary)]">
                                {t('stories.graph.condition')}
                              </p>
                              <p className="text-xs text-[var(--color-text-muted)]">
                                {transition.condition
                                  ? t('stories.graph.conditionEnabled')
                                  : t('stories.graph.conditionDisabled')}
                              </p>
                            </div>
                            {!readOnly ? (
                              <Button
                                onClick={() => {
                                  onToggleCondition(
                                    selectedNode.id,
                                    transitionIndex,
                                    !transition.condition,
                                  )
                                }}
                                size="sm"
                                variant={transition.condition ? 'secondary' : 'ghost'}
                              >
                                {transition.condition
                                  ? t('stories.graph.disableCondition')
                                  : t('stories.graph.enableCondition')}
                              </Button>
                            ) : null}
                          </div>

                          {transition.condition ? (
                            <div className="grid gap-3">
                              <div className="grid gap-3 sm:grid-cols-2">
                                <Field label={t('stories.graph.conditionScope')}>
                                  <Select
                                    disabled={readOnly}
                                    items={conditionScopeItems.map((item) => ({
                                      label: t(`stories.graph.scope.${item.value}` as const),
                                      value: item.value,
                                    }))}
                                    onValueChange={(nextValue) => {
                                      onUpdateTransitionCondition(
                                        selectedNode.id,
                                        transitionIndex,
                                        {
                                          scope: nextValue as ConditionScope,
                                        },
                                      )
                                    }}
                                    textAlign="start"
                                    value={transition.condition.scope ?? 'global'}
                                  />
                                </Field>
                                <Field label={t('stories.graph.conditionOperator')}>
                                  <Select
                                    disabled={readOnly}
                                    items={conditionOperatorItems}
                                    onValueChange={(nextValue) => {
                                      onUpdateTransitionCondition(
                                        selectedNode.id,
                                        transitionIndex,
                                        {
                                          op: nextValue as ConditionOperator,
                                        },
                                      )
                                    }}
                                    textAlign="start"
                                    value={transition.condition.op}
                                  />
                                </Field>
                              </div>

                              {transition.condition.scope === 'character' ? (
                                <Field label={t('stories.graph.conditionCharacter')}>
                                  <StoryGraphCharacterSelect
                                    characters={availableCharacters}
                                    disabled={readOnly}
                                    onValueChange={(nextValue) => {
                                      onUpdateTransitionCondition(
                                        selectedNode.id,
                                        transitionIndex,
                                        {
                                          character: nextValue,
                                        },
                                      )
                                    }}
                                    value={transition.condition.character ?? ''}
                                  />
                                </Field>
                              ) : null}

                              <Field label={t('stories.graph.conditionKey')}>
                                <Select
                                  allowClear
                                  clearLabel={t('stories.graph.clearSelection')}
                                  disabled={isConditionKeyDisabled}
                                  items={conditionKeyItems}
                                  onValueChange={(nextValue) => {
                                    onUpdateTransitionCondition(selectedNode.id, transitionIndex, {
                                      key: nextValue,
                                    })
                                  }}
                                  placeholder={t('stories.graph.keySelectPlaceholder')}
                                  textAlign="start"
                                  value={transition.condition.key || undefined}
                                />
                              </Field>
                              {conditionKeyHint ? (
                                <p className="text-xs text-[var(--color-text-muted)]">
                                  {conditionKeyHint}
                                </p>
                              ) : null}

                              <Field label={t('stories.graph.conditionValue')}>
                                <Textarea
                                  className={cn(
                                    conditionValueError
                                      ? 'border-[var(--color-state-error-line)] focus:border-[var(--color-state-error-line)]'
                                      : undefined,
                                  )}
                                  id={`story-graph-condition-value-${selectedNode.id}-${transitionIndex}`}
                                  name={`story-graph-condition-value-${selectedNode.id}-${transitionIndex}`}
                                  onChange={(event) => {
                                    onConditionDraftChange(
                                      selectedNode.id,
                                      transitionIndex,
                                      event.target.value,
                                    )
                                  }}
                                  placeholder={t('stories.graph.conditionValuePlaceholder')}
                                  readOnly={readOnly}
                                  rows={4}
                                  value={conditionDraft}
                                />
                              </Field>
                              {conditionValueError ? (
                                <p className="text-xs text-[var(--color-state-error-text)]">
                                  {conditionValueError}
                                </p>
                              ) : null}
                            </div>
                          ) : null}
                        </StoryGraphCollapsibleCard>
                      )
                    })}
                  </div>
                </div>
              ) : null}

              {activeTab === 'updates' ? (
                <StoryGraphOnEnterUpdatesEditor
                  availableCharacters={availableCharacters}
                  drafts={onEnterUpdateDrafts}
                  expandedOperationKeys={expandedOnEnterUpdateKeys}
                  key={selectedNode.id}
                  node={selectedNode}
                  onDraftChange={onOnEnterUpdateDraftChange}
                  onRemoveOperation={onRemoveOnEnterUpdate}
                  onToggleOperation={(operationIndex) => {
                    const expansionKey = buildOnEnterUpdateExpansionKey(
                      selectedNode.id,
                      operationIndex,
                    )

                    setExpandedOnEnterUpdateKeys((currentKeys) => {
                      const nextKeys = new Set(currentKeys)

                      if (nextKeys.has(expansionKey)) {
                        nextKeys.delete(expansionKey)
                      } else {
                        nextKeys.add(expansionKey)
                      }

                      return nextKeys
                    })
                  }}
                  onUpdateOperation={onUpdateOnEnterUpdate}
                  readOnly={readOnly}
                  schemaCatalog={schemaCatalog}
                />
              ) : null}
            </div>
          </motion.div>
        </AnimatePresence>
      </div>
    </div>
  )
}

export function StoryGraphInspector({
  availableCharacters,
  conditionDrafts,
  graph,
  newNodeIds,
  onAddOnEnterUpdate,
  onAddTransition,
  onConditionDraftChange,
  onDeleteNode,
  onOnEnterUpdateDraftChange,
  onRemoveTransition,
  onRemoveOnEnterUpdate,
  onSelectTransition,
  onSetStartNode,
  onToggleCondition,
  onUpdateCharacters,
  onUpdateNodeField,
  onUpdateNodeId,
  onUpdateOnEnterUpdate,
  onUpdateTransition,
  onEnterUpdateDrafts,
  onUpdateTransitionCondition,
  playerSchemaId,
  readOnly = false,
  selectedNodeId,
  selectedTransitionIndex,
  worldSchemaId,
}: StoryGraphInspectorProps) {
  const { t } = useTranslation()
  const [activeTab, setActiveTab] = useState<InspectorTab>('node')

  const selectedNode = useMemo(
    () => graph.nodes.find((node) => node.id === selectedNodeId) ?? null,
    [graph.nodes, selectedNodeId],
  )

  if (!selectedNode) {
    return (
      <div className="flex h-full min-h-0 flex-col justify-center px-6 py-6 text-center">
        <p className="text-base font-medium text-[var(--color-text-primary)]">
          {t('stories.graph.inspectorEmptyTitle')}
        </p>
        <p className="mt-3 text-sm leading-7 text-[var(--color-text-secondary)]">
          {t('stories.graph.inspectorEmptyDescription')}
        </p>
      </div>
    )
  }

  return (
    <SelectedNodeInspectorContent
      activeTab={activeTab}
      availableCharacters={availableCharacters}
      conditionDrafts={conditionDrafts}
      graph={graph}
      newNodeIds={newNodeIds}
      onActiveTabChange={setActiveTab}
      onAddOnEnterUpdate={onAddOnEnterUpdate}
      onAddTransition={onAddTransition}
      onConditionDraftChange={onConditionDraftChange}
      onDeleteNode={onDeleteNode}
      onEnterUpdateDrafts={onEnterUpdateDrafts}
      onOnEnterUpdateDraftChange={onOnEnterUpdateDraftChange}
      onRemoveOnEnterUpdate={onRemoveOnEnterUpdate}
      onRemoveTransition={onRemoveTransition}
      onSelectTransition={onSelectTransition}
      onSetStartNode={onSetStartNode}
      onToggleCondition={onToggleCondition}
      onUpdateCharacters={onUpdateCharacters}
      onUpdateNodeField={onUpdateNodeField}
      onUpdateNodeId={onUpdateNodeId}
      onUpdateOnEnterUpdate={onUpdateOnEnterUpdate}
      onUpdateTransition={onUpdateTransition}
      onUpdateTransitionCondition={onUpdateTransitionCondition}
      playerSchemaId={playerSchemaId}
      readOnly={readOnly}
      selectedNode={selectedNode}
      selectedTransitionIndex={selectedTransitionIndex}
      worldSchemaId={worldSchemaId}
    />
  )
}
