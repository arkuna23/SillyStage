import { useMemo } from 'react'
import { useTranslation } from 'react-i18next'

import { Badge } from '../../components/ui/badge'
import { Button } from '../../components/ui/button'
import { Input } from '../../components/ui/input'
import { Select } from '../../components/ui/select'
import { Textarea } from '../../components/ui/textarea'
import { cn } from '../../lib/cn'
import type {
  ConditionOperator,
  ConditionScope,
  StoryGraph,
  StoryGraphCondition,
  StoryGraphStateOpType,
  StoryGraphTransition,
} from './types'
import { StoryGraphOnEnterUpdatesEditor } from './story-graph-on-enter-updates-editor'
import {
  buildConditionDraftKey,
  getGraphNodeLabel,
  type GraphConditionDrafts,
  type GraphOnEnterUpdateDrafts,
} from './story-graph-editor-utils'

type StoryGraphInspectorProps = {
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
  onUpdateCharacters: (nodeId: string, value: string) => void
  onUpdateNodeField: (nodeId: string, field: 'goal' | 'scene' | 'title', value: string) => void
  onUpdateNodeId: (nodeId: string, value: string) => void
  onUpdateOnEnterUpdate: (
    nodeId: string,
    operationIndex: number,
    patch: {
      character?: string
      key?: string
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
  readOnly?: boolean
  selectedNodeId: string | null
  selectedTransitionIndex: number | null
}

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

function SectionTitle({
  action,
  children,
}: {
  action?: React.ReactNode
  children: React.ReactNode
}) {
  return (
    <div className="flex items-center justify-between gap-3">
      <h4 className="text-sm font-medium text-[var(--color-text-primary)]">{children}</h4>
      {action}
    </div>
  )
}

function Field({
  children,
  label,
}: {
  children: React.ReactNode
  label: string
}) {
  return (
    <label className="block space-y-2">
      <span className="text-xs text-[var(--color-text-muted)]">{label}</span>
      {children}
    </label>
  )
}

export function StoryGraphInspector({
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
  readOnly = false,
  selectedNodeId,
  selectedTransitionIndex,
}: StoryGraphInspectorProps) {
  const { t } = useTranslation()

  const selectedNode = useMemo(
    () => graph.nodes.find((node) => node.id === selectedNodeId) ?? null,
    [graph.nodes, selectedNodeId],
  )

  const duplicateId =
    selectedNode && newNodeIds.has(selectedNode.id)
      ? graph.nodes.filter((node) => node.id === selectedNode.id).length > 1
      : false

  const targetItems = graph.nodes.map((node) => ({
    label: `${getGraphNodeLabel(node)} (${node.id})`,
    value: node.id,
  }))

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
    <div className="scrollbar-none flex h-full min-h-0 flex-col overflow-y-auto px-6 py-6">
      <div className="space-y-2">
        <div className="flex items-center gap-2">
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

      <div className="mt-6 space-y-6">
        <div className="space-y-4 rounded-[1.4rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] p-4">
          <SectionTitle
            action={
              !readOnly ? (
                <div className="flex items-center gap-2">
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
              ) : null
            }
          >
            {t('stories.graph.nodeSection')}
          </SectionTitle>

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
            <Input
              id={`story-graph-node-characters-${selectedNode.id}`}
              name={`story-graph-node-characters-${selectedNode.id}`}
              onChange={(event) => {
                onUpdateCharacters(selectedNode.id, event.target.value)
              }}
              placeholder={t('stories.graph.charactersPlaceholder')}
              readOnly={readOnly}
              value={selectedNode.characters.join(', ')}
            />
          </Field>
        </div>

        <div className="space-y-4 rounded-[1.4rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] p-4">
          <SectionTitle
            action={
              !readOnly ? (
                <Button
                  onClick={() => {
                    onAddTransition(selectedNode.id)
                  }}
                  size="sm"
                  variant="secondary"
                >
                  {t('stories.graph.addTransition' as const)}
                </Button>
              ) : null
            }
          >
            {t('stories.graph.transitions')}
          </SectionTitle>

          {selectedNode.transitions.length === 0 ? (
            <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
              {t('stories.graph.emptyTransitions')}
            </p>
          ) : null}

          <div className="space-y-4">
            {selectedNode.transitions.map((transition, transitionIndex) => {
              const transitionDraftKey = buildConditionDraftKey(selectedNode.id, transitionIndex)
              const conditionDraft =
                conditionDrafts[transitionDraftKey] ??
                JSON.stringify(transition.condition?.value ?? '', null, 2)
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

              return (
                <div
                  className={cn(
                    'space-y-4 rounded-[1.2rem] border bg-[color-mix(in_srgb,var(--color-bg-panel-strong)_75%,transparent)] p-4 transition',
                    selectedTransitionIndex === transitionIndex
                      ? 'border-[var(--color-accent-gold-line)] ring-1 ring-[var(--color-focus-ring)]'
                      : 'border-[var(--color-border-subtle)]',
                  )}
                  key={`${selectedNode.id}:${transitionIndex}`}
                  onClick={() => {
                    onSelectTransition(transitionIndex)
                  }}
                >
                  <div className="flex items-center justify-between gap-3">
                    <p className="text-sm font-medium text-[var(--color-text-primary)]">
                      {t('stories.graph.transitionLabel', { index: transitionIndex + 1 })}
                    </p>
                    {!readOnly ? (
                      <Button
                        onClick={() => {
                          onRemoveTransition(selectedNode.id, transitionIndex)
                        }}
                        size="sm"
                        variant="ghost"
                      >
                        {t('stories.graph.removeTransition')}
                      </Button>
                    ) : null}
                  </div>

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
                          onToggleCondition(selectedNode.id, transitionIndex, !transition.condition)
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
                              onUpdateTransitionCondition(selectedNode.id, transitionIndex, {
                                scope: nextValue as ConditionScope,
                              })
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
                              onUpdateTransitionCondition(selectedNode.id, transitionIndex, {
                                op: nextValue as ConditionOperator,
                              })
                            }}
                            textAlign="start"
                            value={transition.condition.op}
                          />
                        </Field>
                      </div>

                      {transition.condition.scope === 'character' ? (
                        <Field label={t('stories.graph.conditionCharacter')}>
                          <Input
                            id={`story-graph-condition-character-${selectedNode.id}-${transitionIndex}`}
                            name={`story-graph-condition-character-${selectedNode.id}-${transitionIndex}`}
                            onChange={(event) => {
                              onUpdateTransitionCondition(selectedNode.id, transitionIndex, {
                                character: event.target.value,
                              })
                            }}
                            placeholder={t('stories.graph.conditionCharacterPlaceholder')}
                            readOnly={readOnly}
                            value={transition.condition.character ?? ''}
                          />
                        </Field>
                      ) : null}

                      <Field label={t('stories.graph.conditionKey')}>
                        <Input
                          id={`story-graph-condition-key-${selectedNode.id}-${transitionIndex}`}
                          name={`story-graph-condition-key-${selectedNode.id}-${transitionIndex}`}
                          onChange={(event) => {
                            onUpdateTransitionCondition(selectedNode.id, transitionIndex, {
                              key: event.target.value,
                            })
                          }}
                          readOnly={readOnly}
                          value={transition.condition.key}
                        />
                      </Field>

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
                            onConditionDraftChange(selectedNode.id, transitionIndex, event.target.value)
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
                </div>
              )
            })}
          </div>
        </div>

        <StoryGraphOnEnterUpdatesEditor
          drafts={onEnterUpdateDrafts}
          node={selectedNode}
          onAddOperation={onAddOnEnterUpdate}
          onDraftChange={onOnEnterUpdateDraftChange}
          onRemoveOperation={onRemoveOnEnterUpdate}
          onUpdateOperation={onUpdateOnEnterUpdate}
          readOnly={readOnly}
        />
      </div>
    </div>
  )
}
