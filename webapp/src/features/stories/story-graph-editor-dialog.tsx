import { useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { Button } from '../../components/ui/button'
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../../components/ui/dialog'
import { useToast } from '../../components/ui/toast-context'
import { cn } from '../../lib/cn'
import { updateStoryDraftGraph, updateStoryGraph } from './api'
import { StoryGraphCanvas } from './story-graph-canvas'
import { StoryGraphInspector } from './story-graph-inspector'
import { StoryGraphToolbar } from './story-graph-toolbar'
import type { StoryGraph, StoryGraphStateOp, StoryGraphStateOpType } from './types'
import {
  buildConditionDraftKey,
  buildOnEnterUpdateDraftKey,
  buildInitialNodePositions,
  cloneGraph,
  createConditionDrafts,
  createDefaultCondition,
  createDefaultStateOp,
  createOnEnterUpdateDrafts,
  createEmptyNode,
  createUniqueNodeId,
  GRAPH_MAX_ZOOM,
  GRAPH_MIN_ZOOM,
  GRAPH_STAGE_PADDING,
  GRAPH_ZOOM_STEP,
  type GraphOnEnterUpdateDrafts,
  mergeNodePositions,
  normalizeGraphForSave,
  type GraphConditionDrafts,
  type GraphNodePosition,
  type GraphViewport,
} from './story-graph-editor-utils'

type StoryGraphEditorDialogProps = {
  graph: StoryGraph | null
  graphType: 'draft' | 'story'
  onGraphSaved?: (graph: StoryGraph) => void
  onOpenChange: (open: boolean) => void
  open: boolean
  readOnly?: boolean
  resourceId: string
}

const INITIAL_VIEWPORT: GraphViewport = {
  x: 36,
  y: 28,
  zoom: 1,
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
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

function isEditableTarget(target: EventTarget | null) {
  return (
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement ||
    (target instanceof HTMLElement && target.isContentEditable)
  )
}

export function StoryGraphEditorDialog({
  graph,
  graphType,
  onGraphSaved,
  onOpenChange,
  open,
  readOnly = false,
  resourceId,
}: StoryGraphEditorDialogProps) {
  const { t } = useTranslation()
  const { pushToast } = useToast()
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
  const [viewport, setViewport] = useState<GraphViewport>(INITIAL_VIEWPORT)
  const [isSaving, setIsSaving] = useState(false)
  const [isFullscreen, setIsFullscreen] = useState(false)
  const [newNodeIds, setNewNodeIds] = useState<Set<string>>(new Set())

  useEffect(() => {
    if (!open || !graph) {
      return
    }

    const nextGraph = cloneGraph(graph)
    const nextConditionDrafts = createConditionDrafts(nextGraph)
    const nextOnEnterUpdateDrafts = createOnEnterUpdateDrafts(nextGraph)

    setGraphDraft(nextGraph)
    setBaselineGraph(cloneGraph(graph))
    setConditionDrafts(nextConditionDrafts)
    setBaselineConditionDrafts(nextConditionDrafts)
    setOnEnterUpdateDrafts(nextOnEnterUpdateDrafts)
    setBaselineOnEnterUpdateDrafts(nextOnEnterUpdateDrafts)
    setNodePositions(buildInitialNodePositions(nextGraph))
    setSelectedNodeId(nextGraph.start_node || (nextGraph.nodes[0]?.id ?? null))
    setSelectedTransitionIndex(null)
    setViewport(INITIAL_VIEWPORT)
    setIsSaving(false)
    setIsFullscreen(false)
    setNewNodeIds(new Set())
  }, [graph, open])

  useEffect(() => {
    if (!graphDraft) {
      return
    }

    setNodePositions((currentPositions) => {
      const nextPositions = mergeNodePositions(graphDraft, currentPositions)

      return arePositionsEqual(currentPositions, nextPositions) ? currentPositions : nextPositions
    })
  }, [graphDraft])

  useEffect(() => {
    if (!graphDraft || selectedNodeId === null || selectedTransitionIndex === null) {
      return
    }

    const selectedNode = graphDraft.nodes.find((node) => node.id === selectedNodeId)

    if (!selectedNode || selectedTransitionIndex >= selectedNode.transitions.length) {
      setSelectedTransitionIndex(null)
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

  const title = graphType === 'draft' ? t('stories.graph.editDraftTitle') : t('stories.graph.editStoryTitle')
  const subtitle =
    graphType === 'draft'
      ? t('stories.graph.draftSubtitle')
      : t('stories.graph.storySubtitle')
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

  useEffect(() => {
    if (!open) {
      return
    }

    function handleKeyDown(event: KeyboardEvent) {
      if (!(event.ctrlKey || event.metaKey) || isEditableTarget(event.target)) {
        return
      }

      if (event.key === '=' || event.key === '+') {
        event.preventDefault()
        setViewport((currentViewport) => ({
          ...currentViewport,
          zoom: Math.min(
            GRAPH_MAX_ZOOM,
            Number((currentViewport.zoom + GRAPH_ZOOM_STEP).toFixed(2)),
          ),
        }))
        return
      }

      if (event.key === '-') {
        event.preventDefault()
        setViewport((currentViewport) => ({
          ...currentViewport,
          zoom: Math.max(
            GRAPH_MIN_ZOOM,
            Number((currentViewport.zoom - GRAPH_ZOOM_STEP).toFixed(2)),
          ),
        }))
        return
      }

      if (event.key === '0') {
        event.preventDefault()
        setViewport(INITIAL_VIEWPORT)
      }
    }

    window.addEventListener('keydown', handleKeyDown)

    return () => {
      window.removeEventListener('keydown', handleKeyDown)
    }
  }, [open])

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
      currentGraph.nodes.push(createEmptyNode(nextNodeId))

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
      pushToast({
        message: t('stories.graph.errors.lastNode'),
        tone: 'warning',
      })
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
      currentSelectedNodeId === nodeId ? graphDraft.nodes.find((node) => node.id !== nodeId)?.id ?? null : currentSelectedNodeId,
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
    setSelectedNodeId((currentSelectedNodeId) => (currentSelectedNodeId === nodeId ? trimmedValue : currentSelectedNodeId))
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
        const [draftNodeId] = draftKey.split(':')

        if (draftNodeId !== nodeId) {
          nextDrafts[draftKey] = draftValue
        }
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
                  condition: enabled ? transition.condition ?? createDefaultCondition() : null,
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

                if (
                  patch.type !== undefined &&
                  patch.type !== operation.type
                ) {
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
                    default:
                      nextOperation = operation
                  }

                  return nextOperation
                }

                return { ...operation, ...patch } as StoryGraphStateOp
              }),
            }
          : node,
      )

      return currentGraph
    })
  }

  async function handleSave() {
    if (!graphDraft || readOnly) {
      return
    }

    const normalized = normalizeGraphForSave(graphDraft, conditionDrafts, onEnterUpdateDrafts)

    if (!normalized.graph) {
      const errorKey = normalized.errors[0]
      const message = (() => {
        if (errorKey?.startsWith('duplicate:')) {
          return t('stories.graph.errors.duplicateNodeId')
        }
        if (errorKey?.startsWith('missing_target:')) {
          return t('stories.graph.errors.missingTarget')
        }
        if (errorKey?.startsWith('invalid_condition_value:')) {
          return t('stories.graph.errors.invalidConditionValue')
        }
        if (errorKey?.startsWith('invalid_on_enter_update_value:')) {
          return t('stories.graph.errors.invalidOnEnterUpdateValue')
        }
        if (errorKey?.startsWith('missing_condition_key:')) {
          return t('stories.graph.errors.missingConditionKey')
        }
        if (errorKey?.startsWith('missing_condition_character:')) {
          return t('stories.graph.errors.missingConditionCharacter')
        }
        if (errorKey?.startsWith('missing_on_enter_update_key:')) {
          return t('stories.graph.errors.missingOnEnterUpdateKey')
        }
        if (errorKey?.startsWith('missing_on_enter_update_character:')) {
          return t('stories.graph.errors.missingOnEnterUpdateCharacter')
        }
        if (errorKey?.startsWith('unsupported_on_enter_update:')) {
          return t('stories.graph.errors.unsupportedOnEnterUpdate')
        }
        if (errorKey === 'missing_start_node') {
          return t('stories.graph.errors.missingStartNode')
        }
        if (errorKey === 'empty_graph') {
          return t('stories.graph.errors.emptyGraph')
        }
        return t('stories.graph.errors.invalidGraph')
      })()

      pushToast({
        message,
        tone: 'error',
      })
      return
    }

    setIsSaving(true)

    try {
      const nextGraph =
        graphType === 'draft'
          ? (
              await updateStoryDraftGraph({
                draft_id: resourceId,
                partial_graph: normalized.graph,
              })
            ).partial_graph
          : (
              await updateStoryGraph({
                graph: normalized.graph,
                story_id: resourceId,
              })
            ).graph
      const nextConditionDrafts = createConditionDrafts(nextGraph)
      const nextOnEnterUpdateDrafts = createOnEnterUpdateDrafts(nextGraph)

      setGraphDraft(cloneGraph(nextGraph))
      setBaselineGraph(cloneGraph(nextGraph))
      setConditionDrafts(nextConditionDrafts)
      setBaselineConditionDrafts(nextConditionDrafts)
      setOnEnterUpdateDrafts(nextOnEnterUpdateDrafts)
      setBaselineOnEnterUpdateDrafts(nextOnEnterUpdateDrafts)
      setNewNodeIds(new Set())
      onGraphSaved?.(nextGraph)

      pushToast({
        message: t('stories.graph.feedback.saved'),
        tone: 'success',
      })
    } catch (error) {
      pushToast({
        message: getErrorMessage(error, t('stories.graph.feedback.saveFailed')),
        tone: 'error',
      })
    } finally {
      setIsSaving(false)
    }
  }

  function handleResetView() {
    setViewport(INITIAL_VIEWPORT)
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

  if (!graphDraft) {
    return null
  }

  return (
    <Dialog
      onOpenChange={(nextOpen) => {
        onOpenChange(nextOpen)
      }}
      open={open}
    >
      <DialogContent
        aria-describedby={undefined}
        contentClassName={cn(
          'motion-safe:transition-[top,right,bottom,left,width,height,max-height,transform] motion-safe:duration-300 motion-safe:ease-[cubic-bezier(0.22,1,0.36,1)] motion-safe:will-change-[top,right,bottom,left,width,height,transform]',
          isFullscreen
            ? '!h-screen !w-screen !max-h-none'
            : undefined,
        )}
        className={cn(
          'motion-safe:transition-[width,height,max-height,border-radius,border-color,box-shadow] motion-safe:duration-300 motion-safe:ease-[cubic-bezier(0.22,1,0.36,1)]',
          isFullscreen
            ? 'h-full max-h-none w-full !rounded-none !border-x-0 !border-y-0'
            : 'h-[min(92vh,60rem)] w-[min(98vw,108rem)]',
        )}
      >
        <DialogHeader className="space-y-2 border-b border-[var(--color-border-subtle)]">
          <DialogTitle>{title}</DialogTitle>
          <p className="text-sm leading-7 text-[var(--color-text-secondary)]">{subtitle}</p>
        </DialogHeader>

        <StoryGraphToolbar
          graphType={graphType}
          nodeCount={graphDraft.nodes.length}
          onAddNode={handleAddNode}
          onResetView={handleResetView}
          onToggleFullscreen={() => {
            setIsFullscreen((currentValue) => !currentValue)
          }}
          onZoomIn={() => {
            setViewport((currentViewport) => ({
              ...currentViewport,
              zoom: Math.min(GRAPH_MAX_ZOOM, Number((currentViewport.zoom + GRAPH_ZOOM_STEP).toFixed(2))),
            }))
          }}
          onZoomOut={() => {
            setViewport((currentViewport) => ({
              ...currentViewport,
              zoom: Math.max(GRAPH_MIN_ZOOM, Number((currentViewport.zoom - GRAPH_ZOOM_STEP).toFixed(2))),
            }))
          }}
          isFullscreen={isFullscreen}
          readOnly={readOnly}
        />

        <div className="flex min-h-0 flex-1 gap-4 overflow-hidden px-6 pb-6 pt-4 md:px-7 md:pb-7">
          <StoryGraphCanvas
            graph={graphDraft}
            nodePositions={nodePositions}
            onNodePositionChange={(nodeId, position) => {
              setNodePositions((currentPositions) => ({
                ...currentPositions,
                [nodeId]: position,
              }))
            }}
            onSelectNode={handleSelectNode}
            onViewportChange={setViewport}
            readOnly={readOnly}
            selectedEdgeId={selectedEdgeId}
            selectedNodeId={selectedNodeId}
            viewport={viewport}
          />

          <div className="min-h-0 w-[24rem] overflow-hidden rounded-[1.6rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel-strong)]">
            <StoryGraphInspector
              conditionDrafts={conditionDrafts}
              graph={graphDraft}
              newNodeIds={newNodeIds}
              onAddOnEnterUpdate={handleAddOnEnterUpdate}
              onAddTransition={handleAddTransition}
              onConditionDraftChange={(nodeId, transitionIndex, value) => {
                setConditionDrafts((currentDrafts) => ({
                  ...currentDrafts,
                  [buildConditionDraftKey(nodeId, transitionIndex)]: value,
                }))
              }}
              onDeleteNode={handleDeleteNode}
              onOnEnterUpdateDraftChange={(nodeId, operationIndex, value) => {
                setOnEnterUpdateDrafts((currentDrafts) => ({
                  ...currentDrafts,
                  [buildOnEnterUpdateDraftKey(nodeId, operationIndex)]: value,
                }))
              }}
              onRemoveTransition={handleRemoveTransition}
              onRemoveOnEnterUpdate={handleRemoveOnEnterUpdate}
              onSelectTransition={setSelectedTransitionIndex}
              onSetStartNode={(nodeId) => {
                patchGraph((currentGraph) => ({
                  ...currentGraph,
                  start_node: nodeId,
                }))
              }}
              onToggleCondition={handleToggleCondition}
              onUpdateCharacters={handleUpdateCharacters}
              onUpdateNodeField={handleUpdateNodeField}
              onUpdateNodeId={handleUpdateNodeId}
              onUpdateOnEnterUpdate={handleUpdateOnEnterUpdate}
              onUpdateTransition={handleUpdateTransition}
              onEnterUpdateDrafts={onEnterUpdateDrafts}
              onUpdateTransitionCondition={handleUpdateTransitionCondition}
              readOnly={readOnly}
              selectedNodeId={selectedNodeId}
              selectedTransitionIndex={selectedTransitionIndex}
            />
          </div>
        </div>

        <DialogFooter className="justify-between">
          <div className="text-sm text-[var(--color-text-secondary)]">
            {readOnly ? t('stories.graph.readOnlyHint') : t('stories.graph.footerHint')}
          </div>
          <div className="flex flex-col-reverse gap-3 sm:flex-row">
            {!readOnly ? (
              <Button onClick={handleResetChanges} variant="ghost">
                {t('stories.graph.resetChanges')}
              </Button>
            ) : null}
            <DialogClose asChild>
              <Button variant="secondary">{t('stories.actions.close')}</Button>
            </DialogClose>
            {!readOnly ? (
              <Button disabled={!isDirty || isSaving} onClick={handleSave}>
                {isSaving ? t('stories.graph.saving') : t('stories.graph.save')}
              </Button>
            ) : null}
          </div>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
