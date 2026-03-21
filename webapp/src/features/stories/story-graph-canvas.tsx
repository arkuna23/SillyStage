import { type PointerEvent as ReactPointerEvent, useMemo, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { Badge } from '../../components/ui/badge'
import { cn } from '../../lib/cn'
import {
  GRAPH_NODE_HEIGHT,
  GRAPH_NODE_WIDTH,
  GRAPH_STAGE_PADDING,
  type GraphNodePosition,
  type GraphViewport,
  getGraphNodeLabel,
} from './story-graph-editor-utils'
import type { StoryGraph } from './types'

type StoryGraphCanvasProps = {
  graph: StoryGraph
  nodePositions: Record<string, GraphNodePosition>
  onNodePositionChange: (nodeId: string, position: GraphNodePosition) => void
  onSelectNode: (nodeId: string | null) => void
  onViewportChange: (viewport: GraphViewport) => void
  readOnly?: boolean
  selectedEdgeId: string | null
  selectedNodeId: string | null
  viewport: GraphViewport
}

type DragState =
  | {
      kind: 'node'
      nodeId: string
      origin: GraphNodePosition
      pointerX: number
      pointerY: number
    }
  | {
      kind: 'pan'
      origin: GraphViewport
      pointerX: number
      pointerY: number
    }

export function StoryGraphCanvas({
  graph,
  nodePositions,
  onNodePositionChange,
  onSelectNode,
  onViewportChange,
  readOnly = false,
  selectedEdgeId,
  selectedNodeId,
  viewport,
}: StoryGraphCanvasProps) {
  const { t } = useTranslation()
  const containerRef = useRef<HTMLDivElement | null>(null)
  const [dragState, setDragState] = useState<DragState | null>(null)

  const stageMetrics = useMemo(() => {
    const positionedNodes = graph.nodes.map((node) => ({
      node,
      position: nodePositions[node.id] ?? { x: GRAPH_STAGE_PADDING, y: GRAPH_STAGE_PADDING },
    }))

    const width =
      Math.max(
        ...positionedNodes.map(({ position }) => position.x + GRAPH_NODE_WIDTH),
        GRAPH_NODE_WIDTH + GRAPH_STAGE_PADDING,
      ) + GRAPH_STAGE_PADDING
    const height =
      Math.max(
        ...positionedNodes.map(({ position }) => position.y + GRAPH_NODE_HEIGHT),
        GRAPH_NODE_HEIGHT + GRAPH_STAGE_PADDING,
      ) + GRAPH_STAGE_PADDING

    const edges = graph.nodes.flatMap((node) => {
      const fromPosition = nodePositions[node.id]

      if (!fromPosition) {
        return []
      }

      return node.transitions.flatMap((transition, transitionIndex) => {
        const toPosition = nodePositions[transition.to]

        if (!toPosition) {
          return []
        }

        const startX = fromPosition.x + GRAPH_NODE_WIDTH
        const startY = fromPosition.y + GRAPH_NODE_HEIGHT / 2
        const endX = toPosition.x
        const endY = toPosition.y + GRAPH_NODE_HEIGHT / 2
        const controlDistance = Math.max(44, (endX - startX) * 0.45)
        const controlX = startX + controlDistance

        return [
          {
            from: node.id,
            id: `${node.id}:${transitionIndex}:${transition.to}`,
            path: `M ${startX} ${startY} C ${controlX} ${startY}, ${endX - controlDistance} ${endY}, ${endX} ${endY}`,
            to: transition.to,
          },
        ]
      })
    })

    return {
      edges,
      height,
      positionedNodes,
      width,
    }
  }, [graph.nodes, nodePositions])

  function handleCanvasPointerDown(event: ReactPointerEvent<HTMLDivElement>) {
    if (event.button !== 0) {
      return
    }

    onSelectNode(null)
    setDragState({
      kind: 'pan',
      origin: viewport,
      pointerX: event.clientX,
      pointerY: event.clientY,
    })
    event.currentTarget.setPointerCapture(event.pointerId)
  }

  function handleNodePointerDown(event: ReactPointerEvent<HTMLButtonElement>, nodeId: string) {
    if (event.button !== 0) {
      return
    }

    event.stopPropagation()
    onSelectNode(nodeId)

    if (readOnly) {
      return
    }

    setDragState({
      kind: 'node',
      nodeId,
      origin: nodePositions[nodeId] ?? { x: GRAPH_STAGE_PADDING, y: GRAPH_STAGE_PADDING },
      pointerX: event.clientX,
      pointerY: event.clientY,
    })
    event.currentTarget.setPointerCapture(event.pointerId)
  }

  function handlePointerMove(event: ReactPointerEvent<HTMLDivElement>) {
    if (!dragState) {
      return
    }

    const deltaX = event.clientX - dragState.pointerX
    const deltaY = event.clientY - dragState.pointerY

    if (dragState.kind === 'pan') {
      onViewportChange({
        ...dragState.origin,
        x: dragState.origin.x + deltaX,
        y: dragState.origin.y + deltaY,
      })

      return
    }

    onNodePositionChange(dragState.nodeId, {
      x: dragState.origin.x + deltaX / viewport.zoom,
      y: dragState.origin.y + deltaY / viewport.zoom,
    })
  }

  function handlePointerEnd(event: ReactPointerEvent<HTMLDivElement>) {
    if (dragState) {
      try {
        event.currentTarget.releasePointerCapture(event.pointerId)
      } catch {
        // ignore pointer capture release failures
      }
    }

    setDragState(null)
  }

  return (
    <div className="flex min-h-0 flex-1 flex-col overflow-hidden rounded-[1.6rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_75%,transparent)]">
      <div className="flex items-center justify-between gap-3 border-b border-[var(--color-border-subtle)] px-4 py-3">
        <div className="space-y-1">
          <p className="text-sm font-medium text-[var(--color-text-primary)]">
            {t('stories.graph.canvasTitle')}
          </p>
          <p className="text-xs text-[var(--color-text-muted)]">{t('stories.graph.hint')}</p>
        </div>
        <Badge className="normal-case px-3 py-1.5" variant="subtle">
          {Math.round(viewport.zoom * 100)}%
        </Badge>
      </div>

      <div
        className="relative min-h-0 flex-1 overflow-hidden"
        onPointerDown={handleCanvasPointerDown}
        onPointerMove={handlePointerMove}
        onPointerUp={handlePointerEnd}
        onPointerCancel={handlePointerEnd}
        ref={containerRef}
      >
        <div
          className="absolute left-0 top-0 select-none"
          style={{
            height: stageMetrics.height,
            transform: `translate(${viewport.x}px, ${viewport.y}px) scale(${viewport.zoom})`,
            transformOrigin: '0 0',
            width: stageMetrics.width,
          }}
        >
          <svg
            className="absolute left-0 top-0 h-full w-full"
            height={stageMetrics.height}
            viewBox={`0 0 ${stageMetrics.width} ${stageMetrics.height}`}
            width={stageMetrics.width}
          >
            <defs>
              <marker
                id="story-graph-edge-arrow"
                markerHeight="9"
                markerWidth="9"
                orient="auto-start-reverse"
                refX="8"
                refY="4.5"
              >
                <path
                  d="M0,0 L9,4.5 L0,9 z"
                  fill="color-mix(in srgb, var(--color-accent-copper) 55%, var(--color-border-subtle))"
                />
              </marker>
              <marker
                id="story-graph-edge-arrow-active"
                markerHeight="9"
                markerWidth="9"
                orient="auto-start-reverse"
                refX="8"
                refY="4.5"
              >
                <path
                  d="M0,0 L9,4.5 L0,9 z"
                  fill="color-mix(in srgb, var(--color-accent-gold) 70%, white)"
                />
              </marker>
            </defs>
            {stageMetrics.edges.map((edge) => {
              const isSelected = selectedEdgeId === edge.id
              const isConnectedToSelection =
                !isSelected &&
                selectedNodeId !== null &&
                (edge.from === selectedNodeId || edge.to === selectedNodeId)
              const stroke = isSelected
                ? 'color-mix(in srgb, var(--color-accent-gold) 74%, white)'
                : isConnectedToSelection
                  ? 'color-mix(in srgb, var(--color-accent-copper) 78%, white)'
                  : 'color-mix(in srgb, var(--color-accent-copper) 42%, var(--color-border-subtle))'

              return (
                <path
                  d={edge.path}
                  fill="none"
                  key={edge.id}
                  markerEnd={`url(#${isSelected || isConnectedToSelection ? 'story-graph-edge-arrow-active' : 'story-graph-edge-arrow'})`}
                  stroke={stroke}
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeOpacity={isSelected ? '1' : isConnectedToSelection ? '0.96' : '0.9'}
                  strokeWidth={isSelected ? '4.2' : isConnectedToSelection ? '3.4' : '2.75'}
                  vectorEffect="non-scaling-stroke"
                />
              )
            })}
          </svg>

          {stageMetrics.positionedNodes.map(({ node, position }) => {
            const isSelected = selectedNodeId === node.id
            const isStart = graph.start_node === node.id

            return (
              <button
                className={cn(
                  'absolute flex flex-col items-start gap-3 overflow-hidden rounded-[1.5rem] border bg-[var(--color-bg-panel-strong)] px-4 py-4 text-left shadow-[0_18px_44px_rgba(0,0,0,0.18)] transition',
                  isSelected
                    ? 'border-[var(--color-accent-gold-line)] ring-2 ring-[var(--color-focus-ring)]'
                    : 'border-[var(--color-border-subtle)] hover:border-[var(--color-accent-copper-soft)]',
                )}
                key={node.id}
                onPointerDown={(event) => {
                  handleNodePointerDown(event, node.id)
                }}
                style={{
                  height: GRAPH_NODE_HEIGHT,
                  left: position.x,
                  top: position.y,
                  width: GRAPH_NODE_WIDTH,
                }}
                type="button"
              >
                <div className="flex w-full items-start justify-between gap-3">
                  <div className="space-y-1">
                    <p className="line-clamp-2 text-base font-semibold leading-6 text-[var(--color-text-primary)]">
                      {getGraphNodeLabel(node)}
                    </p>
                    <p className="text-xs text-[var(--color-text-muted)]">{node.id}</p>
                  </div>
                  {isStart ? (
                    <Badge className="shrink-0 px-2.5 py-1" variant="info">
                      {t('stories.graph.start')}
                    </Badge>
                  ) : null}
                </div>

                <div className="space-y-2 text-xs leading-6 text-[var(--color-text-secondary)]">
                  <p className="line-clamp-3">
                    <span className="font-medium text-[var(--color-text-muted)]">
                      {t('stories.graph.scene')}:
                    </span>{' '}
                    {node.scene || '—'}
                  </p>
                  <p className="line-clamp-3">
                    <span className="font-medium text-[var(--color-text-muted)]">
                      {t('stories.graph.goal')}:
                    </span>{' '}
                    {node.goal || '—'}
                  </p>
                </div>

                <div className="mt-auto flex w-full items-center justify-between gap-3 text-xs text-[var(--color-text-muted)]">
                  <span className="truncate">
                    {t('stories.graph.charactersCount', { count: node.characters.length })}
                  </span>
                  <span>
                    {t('stories.graph.transitionsCount', { count: node.transitions.length })}
                  </span>
                </div>
              </button>
            )
          })}
        </div>
      </div>
    </div>
  )
}
