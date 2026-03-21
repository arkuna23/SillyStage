import { faRotateRight } from '@fortawesome/free-solid-svg-icons/faRotateRight'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'

import { Badge } from '../../components/ui/badge'
import { Button } from '../../components/ui/button'
import { useToastNotice } from '../../components/ui/toast-context'
import { cn } from '../../lib/cn'
import type { CharacterSummary } from '../characters/types'
import type {
  ConditionOperator,
  ConditionScope,
  StoryDetail,
  StoryGraphCondition,
  StoryGraphNode,
} from '../stories/types'
import { getSessionVariables, updateSessionVariables } from './api'
import type { StageCopy } from './copy'
import { VariableRows } from './stage-variable-editor'
import {
  areVariableValuesEqual,
  cloneSessionVariables,
  createVariableRow,
  normalizeVariableRows,
  parseVariableRows,
  rowsFromVariableMap,
  serializeVariableValue,
  type VariableRowDraft,
} from './stage-variable-utils'
import type { RuntimeSnapshot, SessionVariables, StateUpdate, VariableStateOp } from './types'

type StageSessionVariablesPanelProps = {
  characterMap: Map<string, CharacterSummary>
  copy: StageCopy
  onVariablesApplied: (variables: SessionVariables) => void
  runtimeSnapshot: RuntimeSnapshot | null
  sessionId: string
  story: StoryDetail | null
}

type Notice = {
  message: string
  tone: 'error' | 'success'
}

type VariablesDraftState = {
  customRows: VariableRowDraft[]
  playerRows: VariableRowDraft[]
}

type ParsedVariablesDraft = {
  errors: Record<string, string>
  values: {
    custom: Record<string, unknown>
    player_state: Record<string, unknown>
  } | null
}

type ConditionStatus = 'missing' | 'satisfied' | 'unsatisfied'

function extractVariablesFromSnapshot(snapshot: RuntimeSnapshot | null): SessionVariables | null {
  if (!snapshot) {
    return null
  }

  return {
    character_state: Object.fromEntries(
      Object.entries(snapshot.world_state.character_state).map(([characterId, state]) => [
        characterId,
        { ...state },
      ]),
    ),
    custom: { ...snapshot.world_state.custom },
    player_state: { ...snapshot.world_state.player_state },
  }
}

function buildDraftFromVariables(variables: SessionVariables): VariablesDraftState {
  return {
    customRows: rowsFromVariableMap(variables.custom),
    playerRows: rowsFromVariableMap(variables.player_state),
  }
}

function normalizeDraftState(draft: VariablesDraftState) {
  return {
    customRows: normalizeVariableRows(draft.customRows),
    playerRows: normalizeVariableRows(draft.playerRows),
  }
}

function parseDraftState(
  draft: VariablesDraftState,
  errorCopy: StageCopy['variables']['errors'],
): ParsedVariablesDraft {
  const customParsed = parseVariableRows(draft.customRows, errorCopy)
  const playerParsed = parseVariableRows(draft.playerRows, errorCopy)
  const errors = {
    ...customParsed.errors,
    ...playerParsed.errors,
  }

  if (Object.keys(errors).length > 0) {
    return {
      errors,
      values: null,
    }
  }

  return {
    errors: {},
    values: {
      custom: customParsed.values,
      player_state: playerParsed.values,
    },
  }
}

function buildVariableUpdate(
  original: SessionVariables,
  next: {
    custom: Record<string, unknown>
    player_state: Record<string, unknown>
  },
): StateUpdate {
  const ops: VariableStateOp[] = []

  Object.entries(next.custom).forEach(([key, value]) => {
    if (!areVariableValuesEqual(original.custom[key], value)) {
      ops.push({
        key,
        type: 'SetState',
        value,
      })
    }
  })

  Object.keys(original.custom).forEach((key) => {
    if (!(key in next.custom)) {
      ops.push({
        key,
        type: 'RemoveState',
      })
    }
  })

  Object.entries(next.player_state).forEach(([key, value]) => {
    if (!areVariableValuesEqual(original.player_state[key], value)) {
      ops.push({
        key,
        type: 'SetPlayerState',
        value,
      })
    }
  })

  Object.keys(original.player_state).forEach((key) => {
    if (!(key in next.player_state)) {
      ops.push({
        key,
        type: 'RemovePlayerState',
      })
    }
  })

  return { ops }
}

function resolveConditionScope(scope?: ConditionScope) {
  return scope ?? 'global'
}

function compareNumbers(left: unknown, right: unknown) {
  if (typeof left !== 'number' || typeof right !== 'number') {
    return null
  }

  return left < right ? -1 : left > right ? 1 : 0
}

function includesValue(actual: unknown, expected: unknown) {
  if (Array.isArray(actual)) {
    return actual.some((entry) => areVariableValuesEqual(entry, expected))
  }

  if (typeof actual === 'string' && typeof expected === 'string') {
    return actual.includes(expected)
  }

  if (
    actual &&
    typeof actual === 'object' &&
    !Array.isArray(actual) &&
    typeof expected === 'string'
  ) {
    return Object.hasOwn(actual, expected)
  }

  return false
}

function matchesCondition(actual: unknown, condition: StoryGraphCondition) {
  const operator = condition.op as ConditionOperator

  switch (operator) {
    case 'eq':
      return areVariableValuesEqual(actual, condition.value)
    case 'ne':
      return !areVariableValuesEqual(actual, condition.value)
    case 'gt':
      return compareNumbers(actual, condition.value) === 1
    case 'gte': {
      const comparison = compareNumbers(actual, condition.value)
      return comparison === 1 || comparison === 0
    }
    case 'lt':
      return compareNumbers(actual, condition.value) === -1
    case 'lte': {
      const comparison = compareNumbers(actual, condition.value)
      return comparison === -1 || comparison === 0
    }
    case 'contains':
      return includesValue(actual, condition.value)
    default:
      return false
  }
}

function getConditionActualValue(condition: StoryGraphCondition, variables: SessionVariables) {
  const scope = resolveConditionScope(condition.scope)

  if (scope === 'player') {
    return variables.player_state[condition.key]
  }

  if (scope === 'character') {
    const characterId = condition.character ?? ''
    return variables.character_state[characterId]?.[condition.key]
  }

  return variables.custom[condition.key]
}

function getConditionScopeLabel(
  condition: StoryGraphCondition,
  copy: StageCopy,
  characterMap: Map<string, CharacterSummary>,
) {
  const scope = resolveConditionScope(condition.scope)

  if (scope === 'player') {
    return copy.variables.conditions.scopePlayer
  }

  if (scope === 'character') {
    const characterName = condition.character
      ? (characterMap.get(condition.character)?.name ?? condition.character)
      : null

    return characterName
      ? `${copy.variables.conditions.scopeCharacter} / ${characterName}`
      : copy.variables.conditions.scopeCharacter
  }

  return copy.variables.conditions.scopeCustom
}

function getConditionStatusClassName(status: ConditionStatus) {
  if (status === 'satisfied') {
    return 'border-[var(--color-state-success-line)] bg-[var(--color-state-success-soft)] text-[var(--color-text-primary)]'
  }

  if (status === 'missing') {
    return 'border-[var(--color-state-warning-line)] bg-[var(--color-state-warning-soft)] text-[var(--color-text-primary)]'
  }

  return 'border-[var(--color-state-error-line)] bg-[var(--color-state-error-soft)] text-[var(--color-text-primary)]'
}

function getConditionStatusLabel(status: ConditionStatus, copy: StageCopy) {
  if (status === 'satisfied') {
    return copy.variables.conditions.satisfied
  }

  if (status === 'missing') {
    return copy.variables.conditions.missing
  }

  return copy.variables.conditions.unsatisfied
}

export function StageSessionVariablesPanel({
  characterMap,
  copy,
  onVariablesApplied,
  runtimeSnapshot,
  sessionId,
  story,
}: StageSessionVariablesPanelProps) {
  const [baselineDraft, setBaselineDraft] = useState<VariablesDraftState | null>(null)
  const [draft, setDraft] = useState<VariablesDraftState | null>(null)
  const [originalVariables, setOriginalVariables] = useState<SessionVariables | null>(null)
  const [hasExternalUpdates, setHasExternalUpdates] = useState(false)
  const [isLoading, setIsLoading] = useState(true)
  const [isRefreshing, setIsRefreshing] = useState(false)
  const [isSaving, setIsSaving] = useState(false)
  const [notice, setNotice] = useState<Notice | null>(null)
  const hasHydratedRef = useRef(false)
  useToastNotice(notice)

  const snapshotVariables = useMemo(
    () => extractVariablesFromSnapshot(runtimeSnapshot),
    [runtimeSnapshot],
  )

  const snapshotSignature = useMemo(
    () => (snapshotVariables ? JSON.stringify(snapshotVariables) : null),
    [snapshotVariables],
  )
  const originalSignature = useMemo(
    () => (originalVariables ? JSON.stringify(originalVariables) : null),
    [originalVariables],
  )
  const normalizedBaselineDraft = useMemo(
    () => (baselineDraft ? normalizeDraftState(baselineDraft) : null),
    [baselineDraft],
  )
  const normalizedDraft = useMemo(() => (draft ? normalizeDraftState(draft) : null), [draft])
  const isDirty = useMemo(() => {
    if (!normalizedBaselineDraft || !normalizedDraft) {
      return false
    }

    return JSON.stringify(normalizedBaselineDraft) !== JSON.stringify(normalizedDraft)
  }, [normalizedBaselineDraft, normalizedDraft])

  const parsedDraft = useMemo<ParsedVariablesDraft | null>(() => {
    if (!draft) {
      return null
    }

    return parseDraftState(draft, copy.variables.errors)
  }, [copy.variables.errors, draft])

  const previewVariables = useMemo(() => {
    if (!originalVariables) {
      return null
    }

    if (!parsedDraft?.values) {
      return originalVariables
    }

    return {
      character_state: originalVariables.character_state,
      custom: parsedDraft.values.custom,
      player_state: parsedDraft.values.player_state,
    }
  }, [originalVariables, parsedDraft])

  const currentNode = useMemo<StoryGraphNode | null>(() => {
    if (!story || !runtimeSnapshot) {
      return null
    }

    return (
      story.graph.nodes.find((node) => node.id === runtimeSnapshot.world_state.current_node) ?? null
    )
  }, [runtimeSnapshot, story])

  const storyNodeMap = useMemo(
    () => new Map((story?.graph.nodes ?? []).map((node) => [node.id, node])),
    [story],
  )

  const loadVariables = useCallback(
    async (loadingState: 'initial' | 'refresh') => {
      if (loadingState === 'initial') {
        setIsLoading(true)
      } else {
        setIsRefreshing(true)
      }

      try {
        const result = await getSessionVariables(sessionId)
        const nextVariables = cloneSessionVariables(result)
        const nextDraft = buildDraftFromVariables(nextVariables)

        setOriginalVariables(nextVariables)
        setBaselineDraft(nextDraft)
        setDraft(nextDraft)
        setHasExternalUpdates(false)
        hasHydratedRef.current = true
        onVariablesApplied(nextVariables)
      } catch (error) {
        setNotice({
          message: error instanceof Error ? error.message : copy.variables.variableLoadFailed,
          tone: 'error',
        })
      } finally {
        if (loadingState === 'initial') {
          setIsLoading(false)
        } else {
          setIsRefreshing(false)
        }
      }
    },
    [copy.variables.variableLoadFailed, onVariablesApplied, sessionId],
  )

  useEffect(() => {
    hasHydratedRef.current = false
    void loadVariables('initial')
  }, [loadVariables])

  useEffect(() => {
    if (!hasHydratedRef.current || !snapshotVariables || !originalVariables) {
      return
    }

    if (snapshotSignature === originalSignature) {
      return
    }

    if (isDirty) {
      setHasExternalUpdates(true)
      return
    }

    const nextVariables = cloneSessionVariables(snapshotVariables)
    const nextDraft = buildDraftFromVariables(nextVariables)
    setOriginalVariables(nextVariables)
    setBaselineDraft(nextDraft)
    setDraft(nextDraft)
    setHasExternalUpdates(false)
  }, [isDirty, originalSignature, snapshotSignature, snapshotVariables, originalVariables])

  function updateRows(
    section: 'customRows' | 'playerRows',
    updater: (rows: VariableRowDraft[]) => VariableRowDraft[],
  ) {
    setDraft((current) =>
      current
        ? {
            ...current,
            [section]: updater(current[section]),
          }
        : current,
    )
  }

  async function handleSave() {
    if (!originalVariables || !parsedDraft?.values) {
      return
    }

    const update = buildVariableUpdate(originalVariables, parsedDraft.values)

    if (update.ops.length === 0) {
      return
    }

    setIsSaving(true)

    try {
      const result = await updateSessionVariables(sessionId, { update })
      const nextVariables = cloneSessionVariables(result)
      const nextDraft = buildDraftFromVariables(nextVariables)
      setOriginalVariables(nextVariables)
      setBaselineDraft(nextDraft)
      setDraft(nextDraft)
      setHasExternalUpdates(false)
      onVariablesApplied(nextVariables)
      setNotice({
        message: copy.variables.saved,
        tone: 'success',
      })
    } catch (error) {
      setNotice({
        message: error instanceof Error ? error.message : copy.variables.saveFailed,
        tone: 'error',
      })
    } finally {
      setIsSaving(false)
    }
  }

  if (isLoading || !draft) {
    return (
      <div className="flex h-full min-h-0 flex-col">
        <div className="border-b border-[var(--color-border-subtle)] px-6 py-5 md:px-7">
          <div className="space-y-2">
            <h3 className="font-display text-[1.45rem] leading-tight text-[var(--color-text-primary)]">
              {copy.variables.title}
            </h3>
            <p className="text-sm leading-6 text-[var(--color-text-secondary)]">
              {copy.variables.sectionDescription}
            </p>
          </div>
        </div>
        <div className="scrollbar-none min-h-0 flex-1 overflow-y-auto px-6 pb-6 pt-6 md:px-7 md:pb-7">
          <div className="space-y-4">
            {Array.from({ length: 3 }).map((_, index) => (
              <div
                className="rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-5 py-5"
                key={index}
              >
                <div className="h-5 w-32 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
                <div className="mt-4 h-12 animate-pulse rounded-[1rem] bg-[var(--color-bg-panel)]" />
                <div className="mt-3 h-12 animate-pulse rounded-[1rem] bg-[var(--color-bg-panel)]" />
              </div>
            ))}
          </div>
        </div>
      </div>
    )
  }

  return (
    <div className="flex h-full min-h-0 flex-col">
      <div className="border-b border-[var(--color-border-subtle)] px-6 py-5 md:px-7">
        <div className="flex flex-wrap items-start justify-between gap-4">
          <div className="space-y-2">
            <h3 className="font-display text-[1.45rem] leading-tight text-[var(--color-text-primary)]">
              {copy.variables.title}
            </h3>
            <p className="text-sm leading-6 text-[var(--color-text-secondary)]">
              {copy.variables.sectionDescription}
            </p>
          </div>
          <div className="flex flex-wrap items-center gap-2">
            <Button
              disabled={isRefreshing || isSaving}
              onClick={() => void loadVariables('refresh')}
              size="sm"
              variant="ghost"
            >
              <FontAwesomeIcon className="mr-2 text-xs" icon={faRotateRight} />
              {copy.variables.refresh}
            </Button>
            <Button
              disabled={!isDirty || isSaving}
              onClick={() => {
                if (!baselineDraft) {
                  return
                }
                setDraft(baselineDraft)
                setHasExternalUpdates(false)
              }}
              size="sm"
              variant="ghost"
            >
              {copy.variables.reset}
            </Button>
            <Button
              disabled={!isDirty || isSaving || !parsedDraft?.values}
              onClick={() => void handleSave()}
              size="sm"
            >
              {isSaving ? copy.variables.saving : copy.variables.save}
            </Button>
          </div>
        </div>
      </div>

      <div className="scrollbar-none min-h-0 flex-1 overflow-y-auto px-6 pb-6 pt-6 md:px-7 md:pb-7">
        <div className="space-y-6">
          {hasExternalUpdates ? (
            <div className="rounded-[1.15rem] border border-[var(--color-state-warning-line)] bg-[var(--color-state-warning-soft)] px-4 py-3 text-sm leading-6 text-[var(--color-text-primary)]">
              {copy.variables.hasExternalUpdates}
            </div>
          ) : null}

          <section className="space-y-4 rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
            <div className="space-y-1">
              <p className="text-sm font-medium text-[var(--color-text-primary)]">
                {copy.variables.custom}
              </p>
            </div>
            <VariableRows
              addLabel={copy.variables.addRow}
              errors={parsedDraft?.errors ?? {}}
              fieldIdPrefix="stage-session-custom-variable"
              jsonLabel={copy.variables.json}
              keyLabel={copy.variables.key}
              onAddRow={() => {
                updateRows('customRows', (rows) => [...rows, createVariableRow()])
              }}
              onChangeKey={(rowId, value) => {
                updateRows('customRows', (rows) =>
                  rows.map((row) => (row.id === rowId ? { ...row, key: value } : row)),
                )
              }}
              onChangeValue={(rowId, value) => {
                updateRows('customRows', (rows) =>
                  rows.map((row) => (row.id === rowId ? { ...row, valueText: value } : row)),
                )
              }}
              onRemoveRow={(rowId) => {
                updateRows('customRows', (rows) => rows.filter((row) => row.id !== rowId))
              }}
              rows={draft.customRows}
            />
          </section>

          <section className="space-y-4 rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
            <div className="space-y-1">
              <p className="text-sm font-medium text-[var(--color-text-primary)]">
                {copy.variables.playerState}
              </p>
            </div>
            <VariableRows
              addLabel={copy.variables.addRow}
              errors={parsedDraft?.errors ?? {}}
              fieldIdPrefix="stage-session-player-variable"
              jsonLabel={copy.variables.json}
              keyLabel={copy.variables.key}
              onAddRow={() => {
                updateRows('playerRows', (rows) => [...rows, createVariableRow()])
              }}
              onChangeKey={(rowId, value) => {
                updateRows('playerRows', (rows) =>
                  rows.map((row) => (row.id === rowId ? { ...row, key: value } : row)),
                )
              }}
              onChangeValue={(rowId, value) => {
                updateRows('playerRows', (rows) =>
                  rows.map((row) => (row.id === rowId ? { ...row, valueText: value } : row)),
                )
              }}
              onRemoveRow={(rowId) => {
                updateRows('playerRows', (rows) => rows.filter((row) => row.id !== rowId))
              }}
              rows={draft.playerRows}
            />
          </section>

          <section className="space-y-4 rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
            <div className="space-y-1">
              <p className="text-sm font-medium text-[var(--color-text-primary)]">
                {copy.variables.conditions.section}
              </p>
              {currentNode ? (
                <p className="text-sm leading-6 text-[var(--color-text-secondary)]">
                  {currentNode.title}
                </p>
              ) : null}
            </div>

            {currentNode && currentNode.transitions.length > 0 ? (
              <div className="space-y-3">
                {currentNode.transitions.map((transition, index) => {
                  const targetNode = storyNodeMap.get(transition.to)
                  const condition = transition.condition

                  if (!condition) {
                    return (
                      <div
                        className="rounded-[1.2rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-4"
                        key={`${transition.to}-${index}`}
                      >
                        <div className="flex flex-wrap items-center justify-between gap-3">
                          <div className="space-y-1">
                            <p className="text-xs uppercase tracking-[0.12em] text-[var(--color-text-muted)]">
                              {copy.variables.conditions.target}
                            </p>
                            <p className="text-sm text-[var(--color-text-primary)]">
                              {targetNode?.title ?? transition.to}
                            </p>
                            <p className="text-xs text-[var(--color-text-muted)]">
                              {transition.to}
                            </p>
                          </div>
                          <Badge variant="info">{copy.variables.conditions.noCondition}</Badge>
                        </div>
                      </div>
                    )
                  }

                  const actualValue = previewVariables
                    ? getConditionActualValue(condition, previewVariables)
                    : undefined
                  const status: ConditionStatus =
                    typeof actualValue === 'undefined'
                      ? 'missing'
                      : matchesCondition(actualValue, condition)
                        ? 'satisfied'
                        : 'unsatisfied'

                  return (
                    <div
                      className="rounded-[1.2rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-4"
                      key={`${transition.to}-${index}`}
                    >
                      <div className="flex flex-wrap items-start justify-between gap-3">
                        <div className="space-y-1">
                          <p className="text-xs uppercase tracking-[0.12em] text-[var(--color-text-muted)]">
                            {copy.variables.conditions.target}
                          </p>
                          <p className="text-sm text-[var(--color-text-primary)]">
                            {targetNode?.title ?? transition.to}
                          </p>
                          <p className="text-xs text-[var(--color-text-muted)]">{transition.to}</p>
                        </div>
                        <span
                          className={cn(
                            'inline-flex items-center rounded-full border px-3 py-1 text-xs font-medium uppercase',
                            getConditionStatusClassName(status),
                          )}
                        >
                          {getConditionStatusLabel(status, copy)}
                        </span>
                      </div>

                      <div className="mt-4 grid gap-3 md:grid-cols-2">
                        <div className="space-y-1">
                          <p className="text-xs text-[var(--color-text-muted)]">
                            {getConditionScopeLabel(condition, copy, characterMap)}
                          </p>
                          <p className="font-mono text-sm text-[var(--color-text-primary)]">
                            {condition.key}
                          </p>
                        </div>
                        <div className="space-y-1">
                          <p className="text-xs text-[var(--color-text-muted)]">
                            {copy.variables.operator}
                          </p>
                          <p className="font-mono text-sm text-[var(--color-text-primary)]">
                            {condition.op}
                          </p>
                        </div>
                        <div className="space-y-1">
                          <p className="text-xs text-[var(--color-text-muted)]">
                            {copy.variables.value}
                          </p>
                          <p className="font-mono text-sm leading-6 text-[var(--color-text-primary)]">
                            {serializeVariableValue(condition.value)}
                          </p>
                        </div>
                        <div className="space-y-1">
                          <p className="text-xs text-[var(--color-text-muted)]">
                            {copy.variables.conditions.currentValue}
                          </p>
                          <p className="font-mono text-sm leading-6 text-[var(--color-text-primary)]">
                            {typeof actualValue === 'undefined'
                              ? '—'
                              : serializeVariableValue(actualValue)}
                          </p>
                        </div>
                      </div>
                    </div>
                  )
                })}
              </div>
            ) : (
              <div className="rounded-[1.2rem] border border-dashed border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-4 text-sm leading-6 text-[var(--color-text-secondary)]">
                {copy.variables.conditions.empty}
              </div>
            )}
          </section>
        </div>
      </div>
    </div>
  )
}
