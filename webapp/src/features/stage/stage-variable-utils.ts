import type { SessionVariables } from './types'

export type VariableRowDraft = {
  id: string
  key: string
  valueText: string
}

export type VariableEditorErrorCopy = {
  duplicateKey: string
  invalidJson: string
  keyRequired: string
}

let rowCounter = 0

export function createVariableRow(key = '', valueText = 'null'): VariableRowDraft {
  rowCounter += 1

  return {
    id: `variable-row-${Date.now()}-${rowCounter}`,
    key,
    valueText,
  }
}

export function serializeVariableValue(value: unknown) {
  try {
    return JSON.stringify(value)
  } catch {
    return 'null'
  }
}

export function cloneSessionVariables(variables: SessionVariables): SessionVariables {
  return {
    character_state: Object.fromEntries(
      Object.entries(variables.character_state).map(([characterId, state]) => [
        characterId,
        { ...state },
      ]),
    ),
    custom: { ...variables.custom },
    player_state: { ...variables.player_state },
  }
}

export function rowsFromVariableMap(values: Record<string, unknown>) {
  return Object.entries(values)
    .sort(([left], [right]) => left.localeCompare(right))
    .map(([key, value]) => createVariableRow(key, serializeVariableValue(value)))
}

export function normalizeVariableRows(rows: VariableRowDraft[]) {
  return rows.map((row) => ({
    key: row.key,
    valueText: row.valueText,
  }))
}

export function areVariableValuesEqual(left: unknown, right: unknown) {
  return JSON.stringify(left) === JSON.stringify(right)
}

export function parseVariableRows(rows: VariableRowDraft[], errorCopy: VariableEditorErrorCopy) {
  const errors: Record<string, string> = {}
  const values: Record<string, unknown> = {}
  const duplicateIds = new Map<string, string[]>()

  rows.forEach((row) => {
    const key = row.key.trim()

    if (!key) {
      errors[row.id] = errorCopy.keyRequired
      return
    }

    duplicateIds.set(key, [...(duplicateIds.get(key) ?? []), row.id])

    try {
      values[key] = JSON.parse(row.valueText)
    } catch {
      errors[row.id] = errorCopy.invalidJson
    }
  })

  duplicateIds.forEach((rowIds) => {
    if (rowIds.length < 2) {
      return
    }

    rowIds.forEach((rowId) => {
      errors[rowId] = errorCopy.duplicateKey
    })
  })

  return {
    errors,
    values,
  }
}
