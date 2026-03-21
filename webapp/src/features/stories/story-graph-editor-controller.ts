import { type Dispatch, type SetStateAction, useEffect, useMemo, useState } from 'react'
import {
  buildConditionDraftKey,
  buildInitialNodePositions,
  buildOnEnterUpdateDraftKey,
  cloneGraph,
  createConditionDrafts,
  createDefaultCondition,
  createDefaultStateOp,
  createOnEnterUpdateDrafts,
  createUniqueNodeId,
  GRAPH_STAGE_PADDING,
  type GraphConditionDrafts,
  type GraphNodePosition,
  type GraphOnEnterUpdateDrafts,
  type GraphViewport,
  mergeNodePositions,
  normalizeGraphForSave,
} from './story-graph-editor-utils'
import type { StoryGraph, StoryGraphStateOp, StoryGraphStateOpType } from './types'

export const INITIAL_STORY_GRAPH_VIEWPORT: GraphViewport = {
  x: 36,
  y: 28,
  zoom: 1,
}

function arePositionsEqual(
  left: Record<string, GraphNodePosition>,
  right: Record<string, GraphNodePosition>,
) {
  const leftKeys = Object.keys(left)
  const rightKeys = Object.keys(right)

  if (leftKeys.length !== rightKeys.length) {
    return false
  }

  return leftKeys.every((key) => {
    const leftPosition = left[key]
    const rightPosition = right[key]

    return (
      rightPosition !== undefined &&
      leftPosition.x === rightPosition.x &&
      leftPosition.y === rightPosition.y
    )
  })
}

export function isEditableTarget(target: EventTarget | null) {
  return (
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement ||
    (target instanceof HTMLElement && target.isContentEditable)
  )
}

export type StoryGraphEditorController = {
  conditionDrafts: GraphConditionDrafts
  graphDraft: StoryGraph | null
  handleAddNode: () => void
  handleAddOnEnterUpdate: (nodeId: string) => void
  handleAddTransition: (nodeId: string) => void
  handleDeleteNode: (nodeId: string) => void
  handleResetChanges: () => void
  handleResetView: () => void
  handleRemoveOnEnterUpdate: (nodeId: string, operationIndex: number) => void
  handleRemoveTransition: (nodeId: string, transitionIndex: number) => void
  handleSelectNode: (nodeId: string | null) => void
  handleToggleCondition: (nodeId: string, transitionIndex: number, enabled: boolean) => void
  handleUpdateCharacters: (nodeId: string, value: string) => void
  handleUpdateNodeField: (nodeId: string, field: 'goal' | 'scene' | 'title', value: string) => void
  handleUpdateNodeId: (nodeId: string, value: string) => void
  handleUpdateOnEnterUpdate: (
    nodeId: string,
    operationIndex: number,
    patch: {
      character?: string
      key?: string
      type?: StoryGraphStateOpType
      value?: unknown
    },
  ) => void
  handleUpdateTransition: (
    nodeId: string,
    transitionIndex: number,
    patch: Record<string, unknown>,
  ) => void
  handleUpdateTransitionCondition: (
    nodeId: string,
    transitionIndex: number,
    patch: Record<string, unknown>,
  ) => void
  isDirty: boolean
  newNodeIds: Set<string>
  nodePositions: Record<string, GraphNodePosition>
  onEnterUpdateDrafts: GraphOnEnterUpdateDrafts
  patchGraph: (mutator: (currentGraph: StoryGraph) => StoryGraph) => void
  selectedEdgeId: string | null
  selectedNodeId: string | null
  selectedTransitionIndex: number | null
  setConditionDraftValue: (nodeId: string, transitionIndex: number, value: string) => void
  setNodePosition: (nodeId: string, position: GraphNodePosition) => void
  setSelectedTransitionIndex: (transitionIndex: number | null) => void
  setViewport: Dispatch<SetStateAction<GraphViewport>>
  setOnEnterUpdateDraftValue: (nodeId: string, operationIndex: number, value: string) => void
  syncBaselineGraph: (nextGraph: StoryGraph) => void
  viewport: GraphViewport
}

export function useStoryGraphEditorController({
  graph,
  onLastNodeWarning,
  open,
  readOnly = false,
}: {
  graph: StoryGraph | null
  onLastNodeWarning?: () => void
  open: boolean
  readOnly?: boolean
}): StoryGraphEditorController {
  const [graphDraft, setGraphDraft] = useState<StoryGraph | null>(null)
  const [baselineGraph, setBaselineGraph] = useState<StoryGraph | null>(null)
  const [conditionDrafts, setConditionDrafts] = useState<GraphConditionDrafts>({})
  const [baselineConditionDrafts, setBaselineConditionDrafts] = useState<GraphConditionDrafts>({})
  const [onEnterUpdateDrafts, setOnEnterUpdateDrafts] = useState<GraphOnEnterUpdateDrafts>({})
  const [baselineOnEnterUpdateDrafts, setBaselineOnEnterUpdateDrafts] =
    useState<GraphOnEnterUpdateDrafts>({})
  const [nodePositions, setNodePositions] = useState<Record<string, GraphNodePosition>>({})
  const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null)
  const [selectedTransitionIndex, setSelectedTransitionIndex] = useState<number | null>(null)
  const [viewport, setViewport] = useState<GraphViewport>(INITIAL_STORY_GRAPH_VIEWPORT)
  const [newNodeIds, setNewNodeIds] = useState<Set<string>>(new Set())

  useEffect(() => {
    if (!open || !graph) {
      return
    }

    const nextGraph = cloneGraph(graph)
    const nextConditionDrafts = createConditionDrafts(nextGraph)
    const nextOnEnterUpdateDrafts = createOnEnterUpdateDrafts(nextGraph)
    let cancelled = false

    queueMicrotask(() => {
      if (cancelled) {
        return
      }

      setGraphDraft(nextGraph)
      setBaselineGraph(cloneGraph(graph))
      setConditionDrafts(nextConditionDrafts)
      setBaselineConditionDrafts(nextConditionDrafts)
      setOnEnterUpdateDrafts(nextOnEnterUpdateDrafts)
      setBaselineOnEnterUpdateDrafts(nextOnEnterUpdateDrafts)
      setNodePositions(buildInitialNodePositions(nextGraph))
      setSelectedNodeId(nextGraph.start_node || (nextGraph.nodes[0]?.id ?? null))
      setSelectedTransitionIndex(null)
      setViewport(INITIAL_STORY_GRAPH_VIEWPORT)
      setNewNodeIds(new Set())
    })

    return () => {
      cancelled = true
    }
  }, [graph, open])

  useEffect(() => {
    if (!graphDraft) {
      return
    }
    let cancelled = false

    queueMicrotask(() => {
      if (cancelled) {
        return
      }

      setNodePositions((currentPositions) => {
        const nextPositions = mergeNodePositions(graphDraft, currentPositions)

        return arePositionsEqual(currentPositions, nextPositions) ? currentPositions : nextPositions
      })
    })

    return () => {
      cancelled = true
    }
  }, [graphDraft])

  useEffect(() => {
    if (!graphDraft || selectedNodeId === null || selectedTransitionIndex === null) {
      return
    }

    const selectedNode = graphDraft.nodes.find((node) => node.id === selectedNodeId)

    if (!selectedNode || selectedTransitionIndex >= selectedNode.transitions.length) {
      let cancelled = false

      queueMicrotask(() => {
        if (!cancelled) {
          setSelectedTransitionIndex(null)
        }
      })

      return () => {
        cancelled = true
      }
    }
  }, [graphDraft, selectedNodeId, selectedTransitionIndex])

  const isDirty = useMemo(() => {
    if (!graphDraft || !baselineGraph) {
      return false
    }

    return (
      JSON.stringify(graphDraft) !== JSON.stringify(baselineGraph) ||
      JSON.stringify(conditionDrafts) !== JSON.stringify(baselineConditionDrafts) ||
      JSON.stringify(onEnterUpdateDrafts) !== JSON.stringify(baselineOnEnterUpdateDrafts)
    )
  }, [
    baselineConditionDrafts,
    baselineGraph,
    baselineOnEnterUpdateDrafts,
    conditionDrafts,
    graphDraft,
    onEnterUpdateDrafts,
  ])

  const selectedEdgeId =
    selectedNodeId !== null && selectedTransitionIndex !== null && graphDraft
      ? (() => {
          const selectedNode = graphDraft.nodes.find((node) => node.id === selectedNodeId)
          const selectedTransition = selectedNode?.transitions[selectedTransitionIndex]

          return selectedTransition
            ? `${selectedNodeId}:${selectedTransitionIndex}:${selectedTransition.to}`
            : null
        })()
      : null

  function patchGraph(mutator: (currentGraph: StoryGraph) => StoryGraph) {
    setGraphDraft((currentGraph) => {
      if (!currentGraph) {
        return currentGraph
      }

      return mutator(cloneGraph(currentGraph))
    })
  }

  function handleSelectNode(nodeId: string | null) {
    setSelectedNodeId(nodeId)
    setSelectedTransitionIndex(null)
  }

  function handleAddNode() {
    if (!graphDraft || readOnly) {
      return
    }

    const nextNodeId = createUniqueNodeId(graphDraft)
    const visibleCenterX = (-viewport.x + 520) / viewport.zoom
    const visibleCenterY = (-viewport.y + 280) / viewport.zoom

    patchGraph((currentGraph) => {
      currentGraph.nodes.push({
        characters: [],
        goal: '',
        id: nextNodeId,
        on_enter_updates: [],
        scene: '',
        title: '',
        transitions: [],
      })

      if (!currentGraph.start_node) {
        currentGraph.start_node = nextNodeId
      }

      return currentGraph
    })

    setConditionDrafts((currentDrafts) => ({ ...currentDrafts }))
    setSelectedNodeId(nextNodeId)
    setNewNodeIds((currentIds) => new Set(currentIds).add(nextNodeId))
    setNodePositions((currentPositions) => ({
      ...currentPositions,
      [nextNodeId]: {
        x: Math.max(GRAPH_STAGE_PADDING, visibleCenterX - 124),
        y: Math.max(GRAPH_STAGE_PADDING, visibleCenterY - 88),
      },
    }))
  }

  function handleDeleteNode(nodeId: string) {
    if (!graphDraft || readOnly) {
      return
    }

    if (graphDraft.nodes.length <= 1) {
      onLastNodeWarning?.()
      return
    }

    patchGraph((currentGraph) => {
      currentGraph.nodes = currentGraph.nodes
        .filter((node) => node.id !== nodeId)
        .map((node) => ({
          ...node,
          transitions: node.transitions.filter((transition) => transition.to !== nodeId),
        }))

      if (currentGraph.start_node === nodeId) {
        currentGraph.start_node = currentGraph.nodes[0]?.id ?? ''
      }

      return currentGraph
    })

    setSelectedNodeId((currentSelectedNodeId) =>
      currentSelectedNodeId === nodeId
        ? (graphDraft.nodes.find((node) => node.id !== nodeId)?.id ?? null)
        : currentSelectedNodeId,
    )
    setSelectedTransitionIndex(null)
    setNodePositions((currentPositions) => {
      const nextPositions = { ...currentPositions }
      delete nextPositions[nodeId]
      return nextPositions
    })
    setNewNodeIds((currentIds) => {
      const nextIds = new Set(currentIds)
      nextIds.delete(nodeId)
      return nextIds
    })
  }

  function handleUpdateNodeField(nodeId: string, field: 'goal' | 'scene' | 'title', value: string) {
    patchGraph((currentGraph) => {
      currentGraph.nodes = currentGraph.nodes.map((node) =>
        node.id === nodeId ? { ...node, [field]: value } : node,
      )

      return currentGraph
    })
  }

  function handleUpdateCharacters(nodeId: string, value: string) {
    patchGraph((currentGraph) => {
      currentGraph.nodes = currentGraph.nodes.map((node) =>
        node.id === nodeId
          ? {
              ...node,
              characters: value
                .split(',')
                .map((entry) => entry.trim())
                .filter(Boolean),
            }
          : node,
      )

      return currentGraph
    })
  }

  function handleUpdateNodeId(nodeId: string, value: string) {
    if (readOnly) {
      return
    }

    const trimmedValue = value.trim()

    patchGraph((currentGraph) => {
      currentGraph.nodes = currentGraph.nodes.map((node) => {
        if (node.id === nodeId) {
          return {
            ...node,
            id: trimmedValue,
          }
        }

        return {
          ...node,
          transitions: node.transitions.map((transition) =>
            transition.to === nodeId ? { ...transition, to: trimmedValue } : transition,
          ),
        }
      })

      if (currentGraph.start_node === nodeId) {
        currentGraph.start_node = trimmedValue
      }

      return currentGraph
    })

    setNodePositions((currentPositions) => {
      const currentPosition = currentPositions[nodeId]

      if (!currentPosition) {
        return currentPositions
      }

      const nextPositions = { ...currentPositions }
      delete nextPositions[nodeId]
      nextPositions[trimmedValue] = currentPosition

      return nextPositions
    })
    setSelectedNodeId((currentSelectedNodeId) =>
      currentSelectedNodeId === nodeId ? trimmedValue : currentSelectedNodeId,
    )
    setConditionDrafts((currentDrafts) => {
      const nextDrafts: GraphConditionDrafts = {}

      Object.entries(currentDrafts).forEach(([draftKey, draftValue]) => {
        const [draftNodeId, transitionIndex] = draftKey.split(':')
        const nextNodeId = draftNodeId === nodeId ? trimmedValue : draftNodeId
        nextDrafts[`${nextNodeId}:${transitionIndex}`] = draftValue
      })

      return nextDrafts
    })
    setOnEnterUpdateDrafts((currentDrafts) => {
      const nextDrafts: GraphOnEnterUpdateDrafts = {}

      Object.entries(currentDrafts).forEach(([draftKey, draftValue]) => {
        const [draftNodeId, operationIndex] = draftKey.split(':')
        const nextNodeId = draftNodeId === nodeId ? trimmedValue : draftNodeId
        nextDrafts[`${nextNodeId}:${operationIndex}`] = draftValue
      })

      return nextDrafts
    })
    setNewNodeIds((currentIds) => {
      if (!currentIds.has(nodeId)) {
        return currentIds
      }

      const nextIds = new Set(currentIds)
      nextIds.delete(nodeId)
      nextIds.add(trimmedValue)
      return nextIds
    })
  }

  function handleAddTransition(nodeId: string) {
    if (!graphDraft || readOnly) {
      return
    }

    const defaultTarget = graphDraft.nodes.find((node) => node.id !== nodeId)?.id ?? nodeId

    patchGraph((currentGraph) => {
      currentGraph.nodes = currentGraph.nodes.map((node) =>
        node.id === nodeId
          ? {
              ...node,
              transitions: [...node.transitions, { to: defaultTarget }],
            }
          : node,
      )

      return currentGraph
    })

    const transitionIndex =
      graphDraft.nodes.find((node) => node.id === nodeId)?.transitions.length ?? 0
    setConditionDrafts((currentDrafts) => ({
      ...currentDrafts,
      [buildConditionDraftKey(nodeId, transitionIndex)]: '""',
    }))
    setSelectedTransitionIndex(transitionIndex)
  }

  function handleRemoveTransition(nodeId: string, transitionIndex: number) {
    patchGraph((currentGraph) => {
      currentGraph.nodes = currentGraph.nodes.map((node) =>
        node.id === nodeId
          ? {
              ...node,
              transitions: node.transitions.filter((_, index) => index !== transitionIndex),
            }
          : node,
      )

      return currentGraph
    })

    setConditionDrafts((currentDrafts) => {
      const nextDrafts: GraphConditionDrafts = {}

      Object.entries(currentDrafts).forEach(([draftKey, draftValue]) => {
        const [draftNodeId, draftTransitionIndexValue] = draftKey.split(':')
        const draftTransitionIndex = Number(draftTransitionIndexValue)

        if (draftNodeId !== nodeId) {
          nextDrafts[draftKey] = draftValue
          return
        }

        if (draftTransitionIndex === transitionIndex) {
          return
        }

        const nextTransitionIndex =
          draftTransitionIndex > transitionIndex ? draftTransitionIndex - 1 : draftTransitionIndex
        nextDrafts[buildConditionDraftKey(draftNodeId, nextTransitionIndex)] = draftValue
      })

      return nextDrafts
    })
    setSelectedTransitionIndex((currentSelectedTransitionIndex) => {
      if (currentSelectedTransitionIndex === null) {
        return currentSelectedTransitionIndex
      }

      if (currentSelectedTransitionIndex === transitionIndex) {
        return null
      }

      return currentSelectedTransitionIndex > transitionIndex
        ? currentSelectedTransitionIndex - 1
        : currentSelectedTransitionIndex
    })
  }

  function handleUpdateTransition(
    nodeId: string,
    transitionIndex: number,
    patch: Record<string, unknown>,
  ) {
    patchGraph((currentGraph) => {
      currentGraph.nodes = currentGraph.nodes.map((node) =>
        node.id === nodeId
          ? {
              ...node,
              transitions: node.transitions.map((transition, index) =>
                index === transitionIndex ? { ...transition, ...patch } : transition,
              ),
            }
          : node,
      )

      return currentGraph
    })
  }

  function handleUpdateTransitionCondition(
    nodeId: string,
    transitionIndex: number,
    patch: Record<string, unknown>,
  ) {
    patchGraph((currentGraph) => {
      currentGraph.nodes = currentGraph.nodes.map((node) =>
        node.id === nodeId
          ? {
              ...node,
              transitions: node.transitions.map((transition, index) => {
                if (index !== transitionIndex) {
                  return transition
                }

                return {
                  ...transition,
                  condition: {
                    ...transition.condition,
                    ...patch,
                  } as typeof transition.condition,
                }
              }),
            }
          : node,
      )

      return currentGraph
    })
  }

  function handleToggleCondition(nodeId: string, transitionIndex: number, enabled: boolean) {
    patchGraph((currentGraph) => {
      currentGraph.nodes = currentGraph.nodes.map((node) =>
        node.id === nodeId
          ? {
              ...node,
              transitions: node.transitions.map((transition, index) => {
                if (index !== transitionIndex) {
                  return transition
                }

                return {
                  ...transition,
                  condition: enabled ? (transition.condition ?? createDefaultCondition()) : null,
                }
              }),
            }
          : node,
      )

      return currentGraph
    })

    if (enabled) {
      setConditionDrafts((currentDrafts) => ({
        ...currentDrafts,
        [buildConditionDraftKey(nodeId, transitionIndex)]:
          currentDrafts[buildConditionDraftKey(nodeId, transitionIndex)] ?? '""',
      }))
    }
  }

  function handleAddOnEnterUpdate(nodeId: string) {
    if (!graphDraft || readOnly) {
      return
    }

    const operationIndex =
      graphDraft.nodes.find((node) => node.id === nodeId)?.on_enter_updates?.length ?? 0

    patchGraph((currentGraph) => {
      currentGraph.nodes = currentGraph.nodes.map((node) =>
        node.id === nodeId
          ? {
              ...node,
              on_enter_updates: [...(node.on_enter_updates ?? []), createDefaultStateOp()],
            }
          : node,
      )

      return currentGraph
    })

    setOnEnterUpdateDrafts((currentDrafts) => ({
      ...currentDrafts,
      [buildOnEnterUpdateDraftKey(nodeId, operationIndex)]: '""',
    }))
  }

  function handleRemoveOnEnterUpdate(nodeId: string, operationIndex: number) {
    patchGraph((currentGraph) => {
      currentGraph.nodes = currentGraph.nodes.map((node) =>
        node.id === nodeId
          ? {
              ...node,
              on_enter_updates: (node.on_enter_updates ?? []).filter(
                (_, index) => index !== operationIndex,
              ),
            }
          : node,
      )

      return currentGraph
    })

    setOnEnterUpdateDrafts((currentDrafts) => {
      const nextDrafts: GraphOnEnterUpdateDrafts = {}

      Object.entries(currentDrafts).forEach(([draftKey, draftValue]) => {
        const [draftNodeId, draftOperationIndexValue] = draftKey.split(':')
        const draftOperationIndex = Number(draftOperationIndexValue)

        if (draftNodeId !== nodeId) {
          nextDrafts[draftKey] = draftValue
          return
        }

        if (draftOperationIndex === operationIndex) {
          return
        }

        const nextOperationIndex =
          draftOperationIndex > operationIndex ? draftOperationIndex - 1 : draftOperationIndex
        nextDrafts[buildOnEnterUpdateDraftKey(draftNodeId, nextOperationIndex)] = draftValue
      })

      return nextDrafts
    })
  }

  function handleUpdateOnEnterUpdate(
    nodeId: string,
    operationIndex: number,
    patch: {
      character?: string
      key?: string
      type?: StoryGraphStateOpType
      value?: unknown
    },
  ) {
    patchGraph((currentGraph) => {
      currentGraph.nodes = currentGraph.nodes.map((node) =>
        node.id === nodeId
          ? {
              ...node,
              on_enter_updates: (node.on_enter_updates ?? []).map((operation, index) => {
                if (index !== operationIndex) {
                  return operation
                }

                if (patch.type !== undefined && patch.type !== operation.type) {
                  let nextOperation: StoryGraphStateOp | null = null

                  switch (patch.type) {
                    case 'SetState':
                    case 'SetPlayerState':
                      nextOperation = {
                        key:
                          typeof patch.key === 'string'
                            ? patch.key
                            : 'key' in operation
                              ? operation.key
                              : '',
                        type: patch.type,
                        value:
                          'value' in patch
                            ? patch.value
                            : 'value' in operation
                              ? operation.value
                              : '',
                      }
                      break
                    case 'SetCharacterState':
                      nextOperation = {
                        character:
                          typeof patch.character === 'string'
                            ? patch.character
                            : 'character' in operation
                              ? operation.character
                              : '',
                        key:
                          typeof patch.key === 'string'
                            ? patch.key
                            : 'key' in operation
                              ? operation.key
                              : '',
                        type: patch.type,
                        value:
                          'value' in patch
                            ? patch.value
                            : 'value' in operation
                              ? operation.value
                              : '',
                      }
                      break
                    case 'RemoveState':
                    case 'RemovePlayerState':
                      nextOperation = {
                        key:
                          typeof patch.key === 'string'
                            ? patch.key
                            : 'key' in operation
                              ? operation.key
                              : '',
                        type: patch.type,
                      }
                      break
                    case 'RemoveCharacterState':
                      nextOperation = {
                        character:
                          typeof patch.character === 'string'
                            ? patch.character
                            : 'character' in operation
                              ? operation.character
                              : '',
                        key:
                          typeof patch.key === 'string'
                            ? patch.key
                            : 'key' in operation
                              ? operation.key
                              : '',
                        type: patch.type,
                      }
                      break
                    case 'AddActiveCharacter':
                    case 'RemoveActiveCharacter':
                      nextOperation = {
                        character:
                          typeof patch.character === 'string'
                            ? patch.character
                            : 'character' in operation
                              ? operation.character
                              : '',
                        type: patch.type,
                      }
                      break
                    case 'SetActiveCharacters':
                      nextOperation = {
                        characters:
                          'characters' in patch
                            ? ((patch as { characters?: string[] }).characters ?? [])
                            : 'characters' in operation
                              ? operation.characters
                              : [],
                        type: patch.type,
                      }
                      break
                    case 'SetCurrentNode':
                      nextOperation = {
                        node_id:
                          'node_id' in patch
                            ? ((patch as { node_id?: string }).node_id ?? '')
                            : 'node_id' in operation
                              ? operation.node_id
                              : '',
                        type: patch.type,
                      }
                      break
                    default:
                      nextOperation = operation
                  }

                  return nextOperation
                }

                return {
                  ...operation,
                  ...patch,
                } as StoryGraphStateOp
              }),
            }
          : node,
      )

      return currentGraph
    })
  }

  function handleResetView() {
    setViewport(INITIAL_STORY_GRAPH_VIEWPORT)
  }

  function handleResetChanges() {
    if (!baselineGraph) {
      return
    }

    const nextGraph = cloneGraph(baselineGraph)
    setGraphDraft(nextGraph)
    setConditionDrafts(baselineConditionDrafts)
    setOnEnterUpdateDrafts(baselineOnEnterUpdateDrafts)
    setNodePositions(buildInitialNodePositions(nextGraph))
    setSelectedNodeId(nextGraph.start_node || (nextGraph.nodes[0]?.id ?? null))
    setSelectedTransitionIndex(null)
    setNewNodeIds(new Set())
  }

  function syncBaselineGraph(nextGraph: StoryGraph) {
    const nextConditionDrafts = createConditionDrafts(nextGraph)
    const nextOnEnterUpdateDrafts = createOnEnterUpdateDrafts(nextGraph)

    setGraphDraft(cloneGraph(nextGraph))
    setBaselineGraph(cloneGraph(nextGraph))
    setConditionDrafts(nextConditionDrafts)
    setBaselineConditionDrafts(nextConditionDrafts)
    setOnEnterUpdateDrafts(nextOnEnterUpdateDrafts)
    setBaselineOnEnterUpdateDrafts(nextOnEnterUpdateDrafts)
    setNewNodeIds(new Set())
  }

  return {
    conditionDrafts,
    graphDraft,
    handleAddNode,
    handleAddOnEnterUpdate,
    handleAddTransition,
    handleDeleteNode,
    handleResetChanges,
    handleResetView,
    handleRemoveOnEnterUpdate,
    handleRemoveTransition,
    handleSelectNode,
    handleToggleCondition,
    handleUpdateCharacters,
    handleUpdateNodeField,
    handleUpdateNodeId,
    handleUpdateOnEnterUpdate,
    handleUpdateTransition,
    handleUpdateTransitionCondition,
    isDirty,
    newNodeIds,
    nodePositions,
    onEnterUpdateDrafts,
    patchGraph,
    selectedEdgeId,
    selectedNodeId,
    selectedTransitionIndex,
    setConditionDraftValue: (nodeId, transitionIndex, value) => {
      setConditionDrafts((currentDrafts) => ({
        ...currentDrafts,
        [buildConditionDraftKey(nodeId, transitionIndex)]: value,
      }))
    },
    setNodePosition: (nodeId, position) => {
      setNodePositions((currentPositions) => ({
        ...currentPositions,
        [nodeId]: position,
      }))
    },
    setOnEnterUpdateDraftValue: (nodeId, operationIndex, value) => {
      setOnEnterUpdateDrafts((currentDrafts) => ({
        ...currentDrafts,
        [buildOnEnterUpdateDraftKey(nodeId, operationIndex)]: value,
      }))
    },
    setSelectedTransitionIndex,
    setViewport,
    syncBaselineGraph,
    viewport,
  }
}

export function normalizeControlledStoryGraph(controller: StoryGraphEditorController) {
  if (!controller.graphDraft) {
    return { errors: ['empty_graph'], graph: null }
  }

  return normalizeGraphForSave(
    controller.graphDraft,
    controller.conditionDrafts,
    controller.onEnterUpdateDrafts,
  )
}
