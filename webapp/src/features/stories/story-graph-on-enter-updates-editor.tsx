import { useTranslation } from 'react-i18next'

import { Button } from '../../components/ui/button'
import { Input } from '../../components/ui/input'
import { Select } from '../../components/ui/select'
import { Textarea } from '../../components/ui/textarea'
import { cn } from '../../lib/cn'
import type { StoryGraphNode, StoryGraphStateOpType } from './types'
import {
  buildOnEnterUpdateDraftKey,
  editableGraphStateOpTypes,
  isEditableGraphStateOpType,
  isGraphStateValueOpType,
  type GraphOnEnterUpdateDrafts,
} from './story-graph-editor-utils'

type StoryGraphOnEnterUpdatesEditorProps = {
  drafts: GraphOnEnterUpdateDrafts
  node: StoryGraphNode
  onAddOperation: (nodeId: string) => void
  onDraftChange: (nodeId: string, operationIndex: number, value: string) => void
  onRemoveOperation: (nodeId: string, operationIndex: number) => void
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

export function StoryGraphOnEnterUpdatesEditor({
  drafts,
  node,
  onAddOperation,
  onDraftChange,
  onRemoveOperation,
  onUpdateOperation,
  readOnly = false,
}: StoryGraphOnEnterUpdatesEditorProps) {
  const { t } = useTranslation()

  return (
    <div className="space-y-4 rounded-[1.4rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_75%,transparent)] p-4">
      <div className="flex items-center justify-between gap-3">
        <div className="space-y-1">
          <p className="text-sm font-medium text-[var(--color-text-primary)]">
            {t('stories.graph.onEnterUpdatesTitle')}
          </p>
          <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
            {t('stories.graph.onEnterUpdatesHint')}
          </p>
        </div>
        {!readOnly ? (
          <Button
            onClick={() => {
              onAddOperation(node.id)
            }}
            size="sm"
            variant="secondary"
          >
            {t('stories.graph.addOnEnterUpdate')}
          </Button>
        ) : null}
      </div>

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

            return (
              <div
                className="space-y-4 rounded-[1.2rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel-strong)_75%,transparent)] p-4"
                key={`${node.id}:op:${operationIndex}:${operation.type}`}
              >
                <div className="flex items-center justify-between gap-3">
                  <p className="text-sm font-medium text-[var(--color-text-primary)]">
                    {t('stories.graph.onEnterUpdateLabel', { index: operationIndex + 1 })}
                  </p>
                  {!readOnly && isEditable ? (
                    <Button
                      onClick={() => {
                        onRemoveOperation(node.id, operationIndex)
                      }}
                      size="sm"
                      variant="ghost"
                    >
                      {t('stories.graph.removeOnEnterUpdate')}
                    </Button>
                  ) : null}
                </div>

                <Field label={t('stories.graph.onEnterUpdateType')}>
                  {isEditable ? (
                    <Select
                      disabled={readOnly}
                      items={operationTypeItems.map((item) => ({
                        label: t(`stories.graph.onEnterUpdateTypes.${item.value}` as const),
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
                      <p className="text-xs text-[var(--color-state-error-text)]">
                        {valueError}
                      </p>
                    ) : null}
                  </div>
                )}
              </div>
            )
          })}
        </div>
      ) : (
        <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
          {t('stories.graph.emptyOnEnterUpdates')}
        </p>
      )}
    </div>
  )
}
