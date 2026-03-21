import { faRotateRight } from '@fortawesome/free-solid-svg-icons/faRotateRight'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'

import { Button } from '../../components/ui/button'
import { useToastNotice } from '../../components/ui/toast-context'
import type { CharacterSummary } from '../characters/types'
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
  type VariableRowDraft,
} from './stage-variable-utils'
import type { RuntimeSnapshot, SessionVariables, StateUpdate, VariableStateOp } from './types'

type Notice = {
  message: string
  tone: 'error' | 'success'
}

type StageCharacterVariablesPanelProps = {
  character: CharacterSummary
  copy: StageCopy
  onVariablesApplied: (variables: SessionVariables) => void
  runtimeSnapshot: RuntimeSnapshot | null
  sessionId: string
}

function cloneCharacterState(values: Record<string, unknown>) {
  return { ...values }
}

function buildCharacterRows(values: Record<string, unknown>) {
  return rowsFromVariableMap(values)
}

function buildCharacterStateUpdate(
  characterId: string,
  original: Record<string, unknown>,
  next: Record<string, unknown>,
): StateUpdate {
  const ops: VariableStateOp[] = []

  Object.entries(next).forEach(([key, value]) => {
    if (!areVariableValuesEqual(original[key], value)) {
      ops.push({
        character: characterId,
        key,
        type: 'SetCharacterState',
        value,
      })
    }
  })

  Object.keys(original).forEach((key) => {
    if (!(key in next)) {
      ops.push({
        character: characterId,
        key,
        type: 'RemoveCharacterState',
      })
    }
  })

  return { ops }
}

export function StageCharacterVariablesPanel({
  character,
  copy,
  onVariablesApplied,
  runtimeSnapshot,
  sessionId,
}: StageCharacterVariablesPanelProps) {
  const [baselineRows, setBaselineRows] = useState<VariableRowDraft[] | null>(null)
  const [draftRows, setDraftRows] = useState<VariableRowDraft[] | null>(null)
  const [originalValues, setOriginalValues] = useState<Record<string, unknown> | null>(null)
  const [hasExternalUpdates, setHasExternalUpdates] = useState(false)
  const [isLoading, setIsLoading] = useState(true)
  const [isRefreshing, setIsRefreshing] = useState(false)
  const [isSaving, setIsSaving] = useState(false)
  const [notice, setNotice] = useState<Notice | null>(null)
  const hasHydratedRef = useRef(false)
  useToastNotice(notice)

  const snapshotCharacterState = useMemo(
    () =>
      cloneCharacterState(
        runtimeSnapshot?.world_state.character_state[character.character_id] ?? {},
      ),
    [character.character_id, runtimeSnapshot],
  )
  const snapshotSignature = useMemo(
    () => JSON.stringify(snapshotCharacterState),
    [snapshotCharacterState],
  )
  const originalSignature = useMemo(
    () => (originalValues ? JSON.stringify(originalValues) : null),
    [originalValues],
  )
  const normalizedBaseline = useMemo(
    () => (baselineRows ? normalizeVariableRows(baselineRows) : null),
    [baselineRows],
  )
  const normalizedDraft = useMemo(
    () => (draftRows ? normalizeVariableRows(draftRows) : null),
    [draftRows],
  )
  const isDirty = useMemo(() => {
    if (!normalizedBaseline || !normalizedDraft) {
      return false
    }

    return JSON.stringify(normalizedBaseline) !== JSON.stringify(normalizedDraft)
  }, [normalizedBaseline, normalizedDraft])

  const parsedRows = useMemo(() => {
    if (!draftRows) {
      return null
    }

    return parseVariableRows(draftRows, copy.variables.errors)
  }, [copy.variables.errors, draftRows])

  const loadVariables = useCallback(
    async (loadingState: 'initial' | 'refresh') => {
      if (loadingState === 'initial') {
        setIsLoading(true)
      } else {
        setIsRefreshing(true)
      }

      try {
        const result = await getSessionVariables(sessionId)
        const nextValues = cloneCharacterState(result.character_state[character.character_id] ?? {})
        const nextRows = buildCharacterRows(nextValues)

        setOriginalValues(nextValues)
        setBaselineRows(nextRows)
        setDraftRows(nextRows)
        setHasExternalUpdates(false)
        hasHydratedRef.current = true
        onVariablesApplied(cloneSessionVariables(result))
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
    [character.character_id, copy.variables.variableLoadFailed, onVariablesApplied, sessionId],
  )

  useEffect(() => {
    hasHydratedRef.current = false
    void loadVariables('initial')
  }, [loadVariables])

  useEffect(() => {
    if (!hasHydratedRef.current || !originalValues) {
      return
    }

    if (snapshotSignature === originalSignature) {
      return
    }

    if (isDirty) {
      setHasExternalUpdates(true)
      return
    }

    const nextValues = cloneCharacterState(snapshotCharacterState)
    const nextRows = buildCharacterRows(nextValues)
    setOriginalValues(nextValues)
    setBaselineRows(nextRows)
    setDraftRows(nextRows)
    setHasExternalUpdates(false)
  }, [isDirty, originalSignature, originalValues, snapshotCharacterState, snapshotSignature])

  async function handleSave() {
    if (!originalValues || !parsedRows) {
      return
    }

    if (Object.keys(parsedRows.errors).length > 0) {
      return
    }

    const update = buildCharacterStateUpdate(
      character.character_id,
      originalValues,
      parsedRows.values,
    )

    if (update.ops.length === 0) {
      return
    }

    setIsSaving(true)

    try {
      const result = await updateSessionVariables(sessionId, { update })
      const nextValues = cloneCharacterState(result.character_state[character.character_id] ?? {})
      const nextRows = buildCharacterRows(nextValues)
      setOriginalValues(nextValues)
      setBaselineRows(nextRows)
      setDraftRows(nextRows)
      setHasExternalUpdates(false)
      onVariablesApplied(cloneSessionVariables(result))
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

  if (isLoading || !draftRows) {
    return (
      <div className="space-y-4">
        <p className="text-sm leading-6 text-[var(--color-text-secondary)]">
          {copy.characterDialog.variablesDescription}
        </p>
        <div className="space-y-3">
          {Array.from({ length: 3 }).map((_, index) => (
            <div
              className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4"
              key={index}
            >
              <div className="h-12 animate-pulse rounded-[1rem] bg-[var(--color-bg-panel)]" />
            </div>
          ))}
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-5">
      <div className="flex flex-wrap items-start justify-between gap-4">
        <p className="max-w-[42rem] text-sm leading-6 text-[var(--color-text-secondary)]">
          {copy.characterDialog.variablesDescription}
        </p>
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
              if (!baselineRows) {
                return
              }
              setDraftRows(baselineRows)
              setHasExternalUpdates(false)
            }}
            size="sm"
            variant="ghost"
          >
            {copy.variables.reset}
          </Button>
          <Button
            disabled={
              !isDirty || isSaving || !parsedRows || Object.keys(parsedRows.errors).length > 0
            }
            onClick={() => void handleSave()}
            size="sm"
          >
            {isSaving ? copy.variables.saving : copy.variables.save}
          </Button>
        </div>
      </div>

      {hasExternalUpdates ? (
        <div className="rounded-[1.15rem] border border-[var(--color-state-warning-line)] bg-[var(--color-state-warning-soft)] px-4 py-3 text-sm leading-6 text-[var(--color-text-primary)]">
          {copy.variables.hasExternalUpdates}
        </div>
      ) : null}

      <section className="space-y-4 rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
        <div className="space-y-1">
          <p className="text-sm font-medium text-[var(--color-text-primary)]">{character.name}</p>
          <p className="text-xs text-[var(--color-text-muted)]">{character.character_id}</p>
        </div>
        <VariableRows
          addLabel={copy.variables.addRow}
          errors={parsedRows?.errors ?? {}}
          fieldIdPrefix={`stage-character-variable-${character.character_id}`}
          jsonLabel={copy.variables.json}
          keyLabel={copy.variables.key}
          onAddRow={() => {
            setDraftRows((current) => [...(current ?? []), createVariableRow()])
          }}
          onChangeKey={(rowId, value) => {
            setDraftRows(
              (current) =>
                current?.map((row) => (row.id === rowId ? { ...row, key: value } : row)) ?? current,
            )
          }}
          onChangeValue={(rowId, value) => {
            setDraftRows(
              (current) =>
                current?.map((row) => (row.id === rowId ? { ...row, valueText: value } : row)) ??
                current,
            )
          }}
          onRemoveRow={(rowId) => {
            setDraftRows((current) => current?.filter((row) => row.id !== rowId) ?? current)
          }}
          rows={draftRows}
        />
      </section>
    </div>
  )
}
