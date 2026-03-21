import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'

import { Button } from '../../components/ui/button'
import { Input } from '../../components/ui/input'
import { Select } from '../../components/ui/select'
import { Textarea } from '../../components/ui/textarea'
import { cn } from '../../lib/cn'
import { StoryGraphCollapsibleCard } from './story-graph-collapsible-card'
import {
  buildOnEnterUpdateDraftKey,
  editableGraphStateOpTypes,
  type GraphOnEnterUpdateDrafts,
  isEditableGraphStateOpType,
  isGraphStateValueOpType,
} from './story-graph-editor-utils'
import type { StoryGraphNode, StoryGraphStateOpType } from './types'

type StoryGraphOnEnterUpdatesEditorProps = {
  drafts: GraphOnEnterUpdateDrafts
  expandedOperationKeys: Set<string>
  node: StoryGraphNode
  onDraftChange: (nodeId: string, operationIndex: number, value: string) => void
  onRemoveOperation: (nodeId: string, operationIndex: number) => void
  onToggleOperation: (operationIndex: number) => void
  onUpdateOperation: (
    nodeId: string,
    operationIndex: number,
    patch: {
      character?: string
      key?: string
      type?: StoryGraphStateOpType
      value?: unknown
    },
  ) => void
  readOnly?: boolean
}

const operationTypeItems = editableGraphStateOpTypes.map((value) => ({ value }))

function buildOperationExpansionKey(nodeId: string, operationIndex: number) {
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

export function StoryGraphOnEnterUpdatesEditor({
  drafts,
  expandedOperationKeys,
  node,
  onDraftChange,
  onRemoveOperation,
  onToggleOperation,
  onUpdateOperation,
  readOnly = false,
}: StoryGraphOnEnterUpdatesEditorProps) {
  const { t } = useTranslation()
  const operationTypeLabels: Partial<Record<StoryGraphStateOpType, string>> = {
    RemoveCharacterState: t('stories.graph.onEnterUpdateTypes.RemoveCharacterState'),
    RemovePlayerState: t('stories.graph.onEnterUpdateTypes.RemovePlayerState'),
    RemoveState: t('stories.graph.onEnterUpdateTypes.RemoveState'),
    SetCharacterState: t('stories.graph.onEnterUpdateTypes.SetCharacterState'),
    SetPlayerState: t('stories.graph.onEnterUpdateTypes.SetPlayerState'),
    SetState: t('stories.graph.onEnterUpdateTypes.SetState'),
  }

  return (
    <>
      {node.on_enter_updates?.length ? (
        <div className="space-y-4">
          {node.on_enter_updates.map((operation, operationIndex) => {
            const isEditable = isEditableGraphStateOpType(operation.type)
            const needsValue = isGraphStateValueOpType(operation.type)
            const needsCharacter =
              operation.type === 'SetCharacterState' || operation.type === 'RemoveCharacterState'
            const needsKey = 'key' in operation

            const draftKey = buildOnEnterUpdateDraftKey(node.id, operationIndex)
            const draftValue = drafts[draftKey] ?? 'null'
            const expansionKey = buildOperationExpansionKey(node.id, operationIndex)
            const valueError =
              needsValue && draftValue
                ? (() => {
                    try {
                      JSON.parse(draftValue)
                      return null
                    } catch {
                      return t('stories.graph.errors.invalidOnEnterUpdateValue')
                    }
                  })()
                : null
            const operationTypeLabel = operationTypeLabels[operation.type] ?? operation.type
            const operationSummaryParts = [
              operationTypeLabel,
              'character' in operation ? operation.character : null,
              'key' in operation ? operation.key : null,
            ].filter((part): part is string => typeof part === 'string' && part.trim().length > 0)

            return (
              <StoryGraphCollapsibleCard
                action={
                  !readOnly && isEditable ? (
                    <Button
                      onClick={() => {
                        onRemoveOperation(node.id, operationIndex)
                      }}
                      size="sm"
                      variant="ghost"
                    >
                      {t('stories.graph.removeOnEnterUpdate')}
                    </Button>
                  ) : null
                }
                className="rounded-[1.2rem] bg-[color-mix(in_srgb,var(--color-bg-panel-strong)_75%,transparent)]"
                contentClassName="space-y-4"
                key={`${node.id}:op:${operationIndex}:${operation.type}`}
                onToggle={() => {
                  onToggleOperation(operationIndex)
                }}
                open={expandedOperationKeys.has(expansionKey)}
                subtitle={operationSummaryParts.join(' · ') || operationTypeLabel}
                title={t('stories.graph.onEnterUpdateLabel', { index: operationIndex + 1 })}
              >
                <Field label={t('stories.graph.onEnterUpdateType')}>
                  {isEditable ? (
                    <Select
                      disabled={readOnly}
                      items={operationTypeItems.map((item) => ({
                        label: operationTypeLabels[item.value] ?? item.value,
                        value: item.value,
                      }))}
                      onValueChange={(nextValue) => {
                        const nextType = nextValue as StoryGraphStateOpType
                        const basePatch: {
                          character?: string
                          key?: string
                          type: StoryGraphStateOpType
                          value?: unknown
                        } = { type: nextType }

                        if (
                          nextType === 'SetCharacterState' ||
                          nextType === 'RemoveCharacterState'
                        ) {
                          basePatch.character = ''
                        }

                        if (
                          nextType === 'SetState' ||
                          nextType === 'RemoveState' ||
                          nextType === 'SetPlayerState' ||
                          nextType === 'RemovePlayerState' ||
                          nextType === 'SetCharacterState' ||
                          nextType === 'RemoveCharacterState'
                        ) {
                          basePatch.key = ''
                        }

                        if (
                          nextType === 'SetState' ||
                          nextType === 'SetPlayerState' ||
                          nextType === 'SetCharacterState'
                        ) {
                          basePatch.value = ''
                          onDraftChange(node.id, operationIndex, '""')
                        }

                        onUpdateOperation(node.id, operationIndex, basePatch)
                      }}
                      textAlign="start"
                      value={operation.type}
                    />
                  ) : (
                    <Input
                      id={`story-graph-on-enter-type-${node.id}-${operationIndex}`}
                      name={`story-graph-on-enter-type-${node.id}-${operationIndex}`}
                      readOnly
                      value={operation.type}
                    />
                  )}
                </Field>

                {!isEditable ? (
                  <div className="rounded-[1.1rem] border border-[var(--color-state-error-line)] bg-[color-mix(in_srgb,var(--color-state-error)_14%,transparent)] px-3.5 py-3 text-sm leading-7 text-[var(--color-text-primary)]">
                    {t('stories.graph.onEnterUpdatesUnsupported', { type: operation.type })}
                  </div>
                ) : (
                  <div className="grid gap-3">
                    {needsCharacter ? (
                      <Field label={t('stories.graph.onEnterUpdateCharacter')}>
                        <Input
                          id={`story-graph-on-enter-character-${node.id}-${operationIndex}`}
                          name={`story-graph-on-enter-character-${node.id}-${operationIndex}`}
                          onChange={(event) => {
                            onUpdateOperation(node.id, operationIndex, {
                              character: event.target.value,
                            })
                          }}
                          placeholder={t('stories.graph.onEnterUpdateCharacterPlaceholder')}
                          readOnly={readOnly}
                          value={'character' in operation ? operation.character : ''}
                        />
                      </Field>
                    ) : null}

                    {needsKey ? (
                      <Field label={t('stories.graph.onEnterUpdateKey')}>
                        <Input
                          id={`story-graph-on-enter-key-${node.id}-${operationIndex}`}
                          name={`story-graph-on-enter-key-${node.id}-${operationIndex}`}
                          onChange={(event) => {
                            onUpdateOperation(node.id, operationIndex, {
                              key: event.target.value,
                            })
                          }}
                          readOnly={readOnly}
                          value={'key' in operation ? operation.key : ''}
                        />
                      </Field>
                    ) : null}

                    {needsValue ? (
                      <Field label={t('stories.graph.onEnterUpdateValue')}>
                        <Textarea
                          className={cn(
                            valueError
                              ? 'border-[var(--color-state-error-line)] focus:border-[var(--color-state-error-line)]'
                              : undefined,
                          )}
                          id={`story-graph-on-enter-value-${node.id}-${operationIndex}`}
                          name={`story-graph-on-enter-value-${node.id}-${operationIndex}`}
                          onChange={(event) => {
                            onDraftChange(node.id, operationIndex, event.target.value)
                          }}
                          placeholder={t('stories.graph.onEnterUpdateValuePlaceholder')}
                          readOnly={readOnly}
                          rows={4}
                          value={draftValue}
                        />
                      </Field>
                    ) : null}
                    {valueError ? (
                      <p className="text-xs text-[var(--color-state-error-text)]">{valueError}</p>
                    ) : null}
                  </div>
                )}
              </StoryGraphCollapsibleCard>
            )
          })}
        </div>
      ) : (
        <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
          {t('stories.graph.emptyOnEnterUpdates')}
        </p>
      )}
    </>
  )
}
