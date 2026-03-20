import type {
  ConditionOperator,
  ConditionScope,
  StoryGraph,
  StoryGraphCondition,
  StoryGraphNode,
  StoryGraphStateOp,
  StoryGraphStateOpType,
} from './types'

export const GRAPH_NODE_HEIGHT = 176
export const GRAPH_NODE_WIDTH = 248
export const GRAPH_COLUMN_GAP = 136
export const GRAPH_ROW_GAP = 28
export const GRAPH_STAGE_PADDING = 64
export const GRAPH_MAX_ZOOM = 1.8
export const GRAPH_MIN_ZOOM = 0.55
export const GRAPH_ZOOM_STEP = 0.12

export type GraphNodePosition = {
  x: number
  y: number
}

export type GraphLayoutNode = GraphNodePosition & {
  level: number
  node: StoryGraphNode
}

export type GraphLayoutEdge = {
  from: string
  path: string
  to: string
}

export type GraphLayoutResult = {
  edges: GraphLayoutEdge[]
  height: number
  nodes: GraphLayoutNode[]
  width: number
}

export type GraphViewport = {
  x: number
  y: number
  zoom: number
}

export type GraphConditionDrafts = Record<string, string>
export type GraphOnEnterUpdateDrafts = Record<string, string>

export const defaultConditionScope: ConditionScope = 'global'
export const defaultConditionOperator: ConditionOperator = 'eq'
export const editableGraphStateOpTypes = [
  'SetState',
  'RemoveState',
  'SetPlayerState',
  'RemovePlayerState',
  'SetCharacterState',
  'RemoveCharacterState',
] as const
export const graphStateValueOpTypes = ['SetState', 'SetPlayerState', 'SetCharacterState'] as const

export function cloneGraph(graph: StoryGraph): StoryGraph {
  return JSON.parse(JSON.stringify(graph)) as StoryGraph
}

export function buildConditionDraftKey(nodeId: string, transitionIndex: number) {
  return `${nodeId}:${transitionIndex}`
}

export function buildOnEnterUpdateDraftKey(nodeId: string, operationIndex: number) {
  return `${nodeId}:${operationIndex}`
}

export function serializeConditionValue(value: unknown) {
  try {
    return JSON.stringify(value, null, 2)
  } catch {
    return 'null'
  }
}

export function createConditionDrafts(graph: StoryGraph): GraphConditionDrafts {
  const drafts: GraphConditionDrafts = {}

  graph.nodes.forEach((node) => {
    node.transitions.forEach((transition, transitionIndex) => {
      drafts[buildConditionDraftKey(node.id, transitionIndex)] = serializeConditionValue(
        transition.condition?.value ?? '',
      )
    })
  })

  return drafts
}

export function isEditableGraphStateOpType(type: StoryGraphStateOpType) {
  return editableGraphStateOpTypes.includes(type as (typeof editableGraphStateOpTypes)[number])
}

export function isGraphStateValueOpType(type: StoryGraphStateOpType) {
  return graphStateValueOpTypes.includes(type as (typeof graphStateValueOpTypes)[number])
}

export function serializeStateOpValue(operation: StoryGraphStateOp) {
  if (!('value' in operation)) {
    return 'null'
  }

  return serializeConditionValue(operation.value)
}

export function createOnEnterUpdateDrafts(graph: StoryGraph): GraphOnEnterUpdateDrafts {
  const drafts: GraphOnEnterUpdateDrafts = {}

  graph.nodes.forEach((node) => {
    ;(node.on_enter_updates ?? []).forEach((operation, operationIndex) => {
      if (!isGraphStateValueOpType(operation.type)) {
        return
      }

      drafts[buildOnEnterUpdateDraftKey(node.id, operationIndex)] = serializeStateOpValue(operation)
    })
  })

  return drafts
}

export function getGraphNodeLabel(node: StoryGraphNode) {
  return node.title.trim() || node.id
}

export function buildGraphLayout(graph: StoryGraph | null): GraphLayoutResult {
  if (!graph || graph.nodes.length === 0) {
    return {
      edges: [],
      height: GRAPH_NODE_HEIGHT + GRAPH_STAGE_PADDING * 2,
      nodes: [],
      width: GRAPH_NODE_WIDTH + GRAPH_STAGE_PADDING * 2,
    }
  }

  const nodesById = new Map(graph.nodes.map((node) => [node.id, node]))
  const levels = new Map<string, number>()
  const queue: string[] = []

  const startNodeId =
    nodesById.has(graph.start_node) ? graph.start_node : graph.nodes[0]?.id

  if (startNodeId) {
    levels.set(startNodeId, 0)
    queue.push(startNodeId)
  }

  while (queue.length > 0) {
    const nodeId = queue.shift()

    if (!nodeId) {
      continue
    }

    const node = nodesById.get(nodeId)
    const level = levels.get(nodeId) ?? 0

    if (!node) {
      continue
    }

    node.transitions.forEach((transition) => {
      if (!nodesById.has(transition.to)) {
        return
      }

      const nextLevel = level + 1
      const existingLevel = levels.get(transition.to)

      if (existingLevel === undefined || nextLevel < existingLevel) {
        levels.set(transition.to, nextLevel)
        queue.push(transition.to)
      }
    })
  }

  let maxLevel = Math.max(...levels.values(), 0)

  graph.nodes.forEach((node) => {
    if (!levels.has(node.id)) {
      maxLevel += 1
      levels.set(node.id, maxLevel)
    }
  })

  const nodesByLevel = new Map<number, StoryGraphNode[]>()

  graph.nodes.forEach((node) => {
    const level = levels.get(node.id) ?? 0
    const collection = nodesByLevel.get(level) ?? []
    collection.push(node)
    nodesByLevel.set(level, collection)
  })

  const orderedLevels = Array.from(nodesByLevel.keys()).sort((left, right) => left - right)
  const positionedNodes: GraphLayoutNode[] = []

  orderedLevels.forEach((level, columnIndex) => {
    const layerNodes = (nodesByLevel.get(level) ?? []).sort((left, right) =>
      getGraphNodeLabel(left).localeCompare(getGraphNodeLabel(right)),
    )

    layerNodes.forEach((node, rowIndex) => {
      positionedNodes.push({
        level,
        node,
        x: GRAPH_STAGE_PADDING + columnIndex * (GRAPH_NODE_WIDTH + GRAPH_COLUMN_GAP),
        y: GRAPH_STAGE_PADDING + rowIndex * (GRAPH_NODE_HEIGHT + GRAPH_ROW_GAP),
      })
    })
  })

  const nodePositionMap = new Map(
    positionedNodes.map((positionedNode) => [positionedNode.node.id, positionedNode]),
  )

  const edges: GraphLayoutEdge[] = []

  graph.nodes.forEach((node) => {
    const fromNode = nodePositionMap.get(node.id)

    if (!fromNode) {
      return
    }

    node.transitions.forEach((transition) => {
      const toNode = nodePositionMap.get(transition.to)

      if (!toNode) {
        return
      }

      const startX = fromNode.x + GRAPH_NODE_WIDTH
      const startY = fromNode.y + GRAPH_NODE_HEIGHT / 2
      const endX = toNode.x
      const endY = toNode.y + GRAPH_NODE_HEIGHT / 2
      const controlDistance = Math.max(44, (endX - startX) * 0.45)
      const controlX = startX + controlDistance

      edges.push({
        from: node.id,
        path: `M ${startX} ${startY} C ${controlX} ${startY}, ${endX - controlDistance} ${endY}, ${endX} ${endY}`,
        to: transition.to,
      })
    })
  })

  const width =
    Math.max(
      ...positionedNodes.map((positionedNode) => positionedNode.x + GRAPH_NODE_WIDTH),
      GRAPH_NODE_WIDTH + GRAPH_STAGE_PADDING,
    ) + GRAPH_STAGE_PADDING
  const height =
    Math.max(
      ...positionedNodes.map((positionedNode) => positionedNode.y + GRAPH_NODE_HEIGHT),
      GRAPH_NODE_HEIGHT + GRAPH_STAGE_PADDING,
    ) + GRAPH_STAGE_PADDING

  return {
    edges,
    height,
    nodes: positionedNodes,
    width,
  }
}

export function buildInitialNodePositions(graph: StoryGraph) {
  const layout = buildGraphLayout(graph)
  const positions: Record<string, GraphNodePosition> = {}

  layout.nodes.forEach((positionedNode) => {
    positions[positionedNode.node.id] = {
      x: positionedNode.x,
      y: positionedNode.y,
    }
  })

  return positions
}

export function mergeNodePositions(
  graph: StoryGraph,
  existing: Record<string, GraphNodePosition>,
) {
  const nextPositions = { ...existing }
  const layoutPositions = buildInitialNodePositions(graph)
  const validIds = new Set(graph.nodes.map((node) => node.id))

  Object.keys(nextPositions).forEach((nodeId) => {
    if (!validIds.has(nodeId)) {
      delete nextPositions[nodeId]
    }
  })

  graph.nodes.forEach((node) => {
    if (!nextPositions[node.id]) {
      nextPositions[node.id] = layoutPositions[node.id] ?? { x: GRAPH_STAGE_PADDING, y: GRAPH_STAGE_PADDING }
    }
  })

  return nextPositions
}

export function createEmptyNode(id: string): StoryGraphNode {
  return {
    characters: [],
    goal: '',
    id,
    on_enter_updates: [],
    scene: '',
    title: '',
    transitions: [],
  }
}

export function createDefaultStoryGraph(nodeId = 'node-1'): StoryGraph {
  return {
    nodes: [createEmptyNode(nodeId)],
    start_node: nodeId,
  }
}

export function createUniqueNodeId(graph: StoryGraph) {
  const existingIds = new Set(graph.nodes.map((node) => node.id))
  let index = graph.nodes.length + 1

  while (existingIds.has(`node-${index}`)) {
    index += 1
  }

  return `node-${index}`
}

export function createDefaultCondition(): StoryGraphCondition {
  return {
    key: '',
    op: defaultConditionOperator,
    scope: defaultConditionScope,
    value: '',
  }
}

export function createDefaultStateOp(): StoryGraphStateOp {
  return {
    key: '',
    type: 'SetState',
    value: '',
  }
}

export function normalizeGraphForSave(
  graph: StoryGraph,
  conditionDrafts: GraphConditionDrafts,
  onEnterUpdateDrafts: GraphOnEnterUpdateDrafts,
): { errors: string[]; graph: StoryGraph | null } {
  const normalizedGraph = cloneGraph(graph)
  const errors: string[] = []
  const nodeIds = new Set(normalizedGraph.nodes.map((node) => node.id))
  const seenIds = new Set<string>()

  if (normalizedGraph.nodes.length === 0) {
    errors.push('empty_graph')
  }

  normalizedGraph.nodes.forEach((node) => {
    if (!node.id.trim()) {
      errors.push('empty_node_id')
      return
    }

    if (seenIds.has(node.id)) {
      errors.push(`duplicate:${node.id}`)
      return
    }

    seenIds.add(node.id)

    node.transitions.forEach((transition, transitionIndex) => {
      if (!transition.to || !nodeIds.has(transition.to)) {
        errors.push(`missing_target:${node.id}:${transitionIndex}`)
      }

      if (!transition.condition) {
        return
      }

      if (!transition.condition.key.trim()) {
        errors.push(`missing_condition_key:${node.id}:${transitionIndex}`)
      }

      if (transition.condition.scope === 'character' && !transition.condition.character?.trim()) {
        errors.push(`missing_condition_character:${node.id}:${transitionIndex}`)
      }

      const draftKey = buildConditionDraftKey(node.id, transitionIndex)
      const valueDraft = conditionDrafts[draftKey] ?? serializeConditionValue(transition.condition.value)

      try {
        transition.condition.value = JSON.parse(valueDraft)
      } catch {
        errors.push(`invalid_condition_value:${node.id}:${transitionIndex}`)
      }
    })

    ;(node.on_enter_updates ?? []).forEach((operation, operationIndex) => {
      if (!isEditableGraphStateOpType(operation.type)) {
        errors.push(`unsupported_on_enter_update:${node.id}:${operationIndex}:${operation.type}`)
        return
      }

      if ('key' in operation && !operation.key.trim()) {
        errors.push(`missing_on_enter_update_key:${node.id}:${operationIndex}`)
      }

      if ('character' in operation && !operation.character.trim()) {
        errors.push(`missing_on_enter_update_character:${node.id}:${operationIndex}`)
      }

      if (!isGraphStateValueOpType(operation.type)) {
        return
      }

      const draftKey = buildOnEnterUpdateDraftKey(node.id, operationIndex)
      const valueDraft = onEnterUpdateDrafts[draftKey] ?? serializeStateOpValue(operation)

      try {
        ;(operation as Extract<StoryGraphStateOp, { value: unknown }>).value = JSON.parse(valueDraft)
      } catch {
        errors.push(`invalid_on_enter_update_value:${node.id}:${operationIndex}`)
      }
    })
  })

  if (!normalizedGraph.start_node || !nodeIds.has(normalizedGraph.start_node)) {
    errors.push('missing_start_node')
  }

  return {
    errors,
    graph: errors.length === 0 ? normalizedGraph : null,
  }
}

export function getStoryGraphValidationMessage(
  translate: (key: string, options?: Record<string, unknown>) => string,
  errorKey: string,
) {
  if (errorKey.startsWith('duplicate:') || errorKey === 'empty_node_id') {
    return translate('stories.graph.errors.duplicateNodeId')
  }
  if (errorKey.startsWith('missing_target:')) {
    return translate('stories.graph.errors.missingTarget')
  }
  if (errorKey.startsWith('missing_condition_key:')) {
    return translate('stories.graph.errors.missingConditionKey')
  }
  if (errorKey.startsWith('missing_condition_character:')) {
    return translate('stories.graph.errors.missingConditionCharacter')
  }
  if (errorKey.startsWith('invalid_condition_value:')) {
    return translate('stories.graph.errors.invalidConditionValue')
  }
  if (errorKey.startsWith('missing_on_enter_update_key:')) {
    return translate('stories.graph.errors.missingOnEnterUpdateKey')
  }
  if (errorKey.startsWith('missing_on_enter_update_character:')) {
    return translate('stories.graph.errors.missingOnEnterUpdateCharacter')
  }
  if (errorKey.startsWith('invalid_on_enter_update_value:')) {
    return translate('stories.graph.errors.invalidOnEnterUpdateValue')
  }
  if (errorKey.startsWith('unsupported_on_enter_update:')) {
    return translate('stories.graph.errors.unsupportedOnEnterUpdate')
  }
  if (errorKey === 'missing_start_node') {
    return translate('stories.graph.errors.missingStartNode')
  }
  if (errorKey === 'empty_graph') {
    return translate('stories.graph.errors.emptyGraph')
  }

  return translate('stories.graph.errors.invalidGraph')
}

export function getConditionTargetValue(
  condition: StoryGraphCondition | null | undefined,
  snapshotVariables: {
    character_state?: Record<string, Record<string, unknown>>
    custom?: Record<string, unknown>
    player_state?: Record<string, unknown>
  },
) {
  if (!condition) {
    return undefined
  }

  const scope = condition.scope ?? 'global'

  if (scope === 'character') {
    if (!condition.character) {
      return undefined
    }

    return snapshotVariables.character_state?.[condition.character]?.[condition.key]
  }

  if (scope === 'player') {
    return snapshotVariables.player_state?.[condition.key]
  }

  return snapshotVariables.custom?.[condition.key]
}
