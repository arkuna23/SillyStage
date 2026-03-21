import type { CommonVariableDefinition, StoryDetail } from '../stories/types'
import type { StageCommonVariable } from './stage-ui-types'
import { serializeVariableValue } from './stage-variable-utils'
import type { RuntimeSnapshot } from './types'

function getCommonVariableValue(
  definition: CommonVariableDefinition,
  snapshot: RuntimeSnapshot | null,
) {
  if (!snapshot) {
    return undefined
  }

  if (definition.scope === 'character') {
    if (!definition.character_id) {
      return undefined
    }

    return snapshot.world_state.character_state[definition.character_id]?.[definition.key]
  }

  if (definition.scope === 'player') {
    return snapshot.world_state.player_state[definition.key]
  }

  return snapshot.world_state.custom[definition.key]
}

function formatCommonVariableValue(value: unknown) {
  if (value === undefined) {
    return '—'
  }

  if (typeof value === 'string') {
    return value.length > 0 ? value : '""'
  }

  if (typeof value === 'number' || typeof value === 'boolean' || typeof value === 'bigint') {
    return String(value)
  }

  if (value === null) {
    return 'null'
  }

  return serializeVariableValue(value)
}

export function buildStageCommonVariables(
  story: StoryDetail | null,
  snapshot: RuntimeSnapshot | null,
): StageCommonVariable[] {
  if (!story) {
    return []
  }

  return story.common_variables.map((definition) => ({
    id: `${definition.scope}:${definition.character_id ?? ''}:${definition.key}`,
    label: definition.display_name.trim() || definition.key,
    value: formatCommonVariableValue(getCommonVariableValue(definition, snapshot)),
  }))
}
