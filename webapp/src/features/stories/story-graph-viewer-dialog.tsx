import { useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { Button } from '../../components/ui/button'
import {
  Dialog,
  DialogBody,
  DialogClose,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../../components/ui/dialog'
import { Badge } from '../../components/ui/badge'
import type { StoryGraph } from './types'

type StoryGraphViewerDialogProps = {
  graph: StoryGraph | null
  graphType: 'draft' | 'story'
  highlightNodeId?: string | null
  onOpenChange: (open: boolean) => void
  open: boolean
  subtitle?: string
  title: string
}

type PositionedNode = {
  characters: string[]
  goal: string
  id: string
  level: number
  scene: string
  title: string
  x: number
  y: number
}

type PositionedEdge = {
  from: string
  path: string
  to: string
}

type GraphLayout = {
  edges: PositionedEdge[]
  height: number
  nodes: PositionedNode[]
  width: number
}

const NODE_HEIGHT = 176
const NODE_WIDTH = 248
const COLUMN_GAP = 136
const ROW_GAP = 28
const STAGE_PADDING = 48
const INITIAL_OFFSET = { x: 36, y: 28 }
const MAX_ZOOM = 1.6
const MIN_ZOOM = 0.75
const ZOOM_STEP = 0.12

function summarizeCharacters(
  characters: string[],
  formatOverflow: (names: string, count: number) => string,
  noCharactersLabel: string,
) {
  if (characters.length === 0) {
    return noCharactersLabel
  }

  if (characters.length <= 2) {
    return characters.join(' · ')
  }

  return formatOverflow(characters.slice(0, 2).join(' · '), characters.length)
}

function buildGraphLayout(graph: StoryGraph | null): GraphLayout {
  if (!graph || graph.nodes.length === 0) {
    return {
      edges: [],
      height: NODE_HEIGHT + STAGE_PADDING * 2,
      nodes: [],
      width: NODE_WIDTH + STAGE_PADDING * 2,
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

  const nodesByLevel = new Map<number, typeof graph.nodes>()

  graph.nodes.forEach((node) => {
    const level = levels.get(node.id) ?? 0
    const collection = nodesByLevel.get(level) ?? []
    collection.push(node)
    nodesByLevel.set(level, collection)
  })

  const orderedLevels = Array.from(nodesByLevel.keys()).sort((a, b) => a - b)
  const positionedNodes: PositionedNode[] = []

  orderedLevels.forEach((level, levelIndex) => {
    const layerNodes = (nodesByLevel.get(level) ?? []).sort((left, right) =>
      left.title.localeCompare(right.title),
    )

    layerNodes.forEach((node, rowIndex) => {
      positionedNodes.push({
        ...node,
        level,
        x: STAGE_PADDING + levelIndex * (NODE_WIDTH + COLUMN_GAP),
        y: STAGE_PADDING + rowIndex * (NODE_HEIGHT + ROW_GAP),
      })
    })
  })

  const nodePositionMap = new Map(positionedNodes.map((node) => [node.id, node]))

  const edges: PositionedEdge[] = []

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

      const startX = fromNode.x + NODE_WIDTH
      const startY = fromNode.y + NODE_HEIGHT / 2
      const endX = toNode.x
      const endY = toNode.y + NODE_HEIGHT / 2
      const controlX = startX + Math.max(44, (endX - startX) * 0.45)

      edges.push({
        from: node.id,
        path: `M ${startX} ${startY} C ${controlX} ${startY}, ${endX - (controlX - startX)} ${endY}, ${endX} ${endY}`,
        to: transition.to,
      })
    })
  })

  const width =
    Math.max(...positionedNodes.map((node) => node.x + NODE_WIDTH), NODE_WIDTH + STAGE_PADDING) +
    STAGE_PADDING
  const height =
    Math.max(...positionedNodes.map((node) => node.y + NODE_HEIGHT), NODE_HEIGHT + STAGE_PADDING) +
    STAGE_PADDING

  return {
    edges,
    height,
    nodes: positionedNodes,
    width,
  }
}

export function StoryGraphViewerDialog({
  graph,
  graphType,
  highlightNodeId,
  onOpenChange,
  open,
  subtitle,
  title,
}: StoryGraphViewerDialogProps) {
  const { t } = useTranslation()
  const [zoom, setZoom] = useState(1)
  const [offset, setOffset] = useState(INITIAL_OFFSET)
  const [draggingPointerId, setDraggingPointerId] = useState<number | null>(null)
  const [dragStart, setDragStart] = useState<{ x: number; y: number } | null>(null)

  const layout = useMemo(() => buildGraphLayout(graph), [graph])
  const noCharactersLabel = t('stories.graph.emptyCharacters')

  function handleResetView() {
    setZoom(1)
    setOffset(INITIAL_OFFSET)
  }

  function handleZoom(nextZoom: number) {
    setZoom(Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, nextZoom)))
  }

  return (
    <Dialog
      onOpenChange={(nextOpen) => {
        onOpenChange(nextOpen)
        if (!nextOpen) {
          handleResetView()
        }
      }}
      open={open}
    >
      <DialogContent aria-describedby={undefined} className="w-[min(96vw,78rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <div className="flex items-start justify-between gap-4">
            <div className="space-y-2">
              <DialogTitle>{title}</DialogTitle>
              <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
                {subtitle}
              </p>
            </div>
            <div className="flex items-center gap-2">
              <Button
                onClick={() => {
                  handleZoom(zoom - ZOOM_STEP)
                }}
                size="sm"
                variant="ghost"
              >
                {t('stories.graph.zoomOut')}
              </Button>
              <Button
                onClick={() => {
                  handleZoom(zoom + ZOOM_STEP)
                }}
                size="sm"
                variant="ghost"
              >
                {t('stories.graph.zoomIn')}
              </Button>
              <Button onClick={handleResetView} size="sm" variant="secondary">
                {t('stories.graph.reset')}
              </Button>
            </div>
          </div>
        </DialogHeader>

        <DialogBody className="space-y-4 pt-6">
          <div className="flex flex-wrap gap-2">
            <Badge className="normal-case px-3 py-1.5" variant="info">
              {graphType === 'draft' ? t('stories.graph.partialGraph') : t('stories.graph.finalGraph')}
            </Badge>
            {graph?.start_node ? (
              <Badge className="normal-case px-3 py-1.5" variant="subtle">
                {t('stories.details.startNode', { id: graph.start_node })}
              </Badge>
            ) : null}
            <Badge className="normal-case px-3 py-1.5" variant="subtle">
              {t('stories.details.nodeCount', { count: graph?.nodes.length ?? 0 })}
            </Badge>
          </div>

          {layout.nodes.length === 0 ? (
            <div className="rounded-[1.4rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_72%,transparent)] px-5 py-10 text-center text-sm leading-7 text-[var(--color-text-secondary)]">
              {t('stories.graph.empty')}
            </div>
          ) : (
            <div className="overflow-hidden rounded-[1.6rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_62%,transparent)]">
              <div
                className="relative h-[min(70vh,42rem)] cursor-grab overflow-hidden touch-none bg-[radial-gradient(circle_at_top,rgba(255,255,255,0.06),transparent_45%)] active:cursor-grabbing"
                onPointerDown={(event) => {
                  setDraggingPointerId(event.pointerId)
                  setDragStart({
                    x: event.clientX - offset.x,
                    y: event.clientY - offset.y,
                  })
                }}
                onPointerMove={(event) => {
                  if (draggingPointerId !== event.pointerId || !dragStart) {
                    return
                  }

                  setOffset({
                    x: event.clientX - dragStart.x,
                    y: event.clientY - dragStart.y,
                  })
                }}
                onPointerUp={(event) => {
                  if (draggingPointerId !== event.pointerId) {
                    return
                  }

                  setDraggingPointerId(null)
                  setDragStart(null)
                }}
                onPointerLeave={(event) => {
                  if (draggingPointerId !== event.pointerId) {
                    return
                  }

                  setDraggingPointerId(null)
                  setDragStart(null)
                }}
              >
                <div
                  className="absolute left-0 top-0"
                  style={{
                    height: `${layout.height}px`,
                    transform: `translate(${offset.x}px, ${offset.y}px) scale(${zoom})`,
                    transformOrigin: 'top left',
                    width: `${layout.width}px`,
                  }}
                >
                  <svg
                    className="absolute left-0 top-0"
                    height={layout.height}
                    viewBox={`0 0 ${layout.width} ${layout.height}`}
                    width={layout.width}
                  >
                    <defs>
                      <marker
                        id="story-graph-arrow"
                        markerHeight="8"
                        markerUnits="strokeWidth"
                        markerWidth="8"
                        orient="auto"
                        refX="6.5"
                        refY="3.5"
                      >
                        <path d="M0,0 L7,3.5 L0,7 z" fill="var(--color-accent-copper)" />
                      </marker>
                    </defs>
                    {layout.edges.map((edge) => (
                      <path
                        d={edge.path}
                        fill="none"
                        key={`${edge.from}:${edge.to}`}
                        markerEnd="url(#story-graph-arrow)"
                        stroke="var(--color-accent-copper)"
                        strokeOpacity="0.58"
                        strokeWidth="2"
                      />
                    ))}
                  </svg>

                  {layout.nodes.map((node) => {
                    const isHighlighted = node.id === (highlightNodeId ?? graph?.start_node)

                    return (
                      <div
                        className="absolute"
                        key={node.id}
                        style={{
                          left: `${node.x}px`,
                          top: `${node.y}px`,
                          width: `${NODE_WIDTH}px`,
                        }}
                      >
                        <div
                          className="rounded-[1.45rem] border px-4 py-4 shadow-[0_16px_36px_rgba(0,0,0,0.12)]"
                          style={{
                            background: isHighlighted
                              ? 'color-mix(in_srgb,var(--color-accent-gold-soft)_86%,var(--color-bg-panel-strong))'
                              : 'var(--color-bg-panel-strong)',
                            borderColor: isHighlighted
                              ? 'var(--color-accent-gold-line)'
                              : 'var(--color-border-subtle)',
                          }}
                        >
                          <div className="space-y-2">
                            <div className="flex items-start justify-between gap-3">
                              <div className="min-w-0">
                                <p className="truncate font-display text-[1.05rem] leading-tight text-[var(--color-text-primary)]">
                                  {node.title}
                                </p>
                                <p className="mt-1 truncate font-mono text-[0.72rem] leading-5 text-[var(--color-text-muted)]">
                                  {node.id}
                                </p>
                              </div>
                              {node.id === graph?.start_node ? (
                                <Badge className="shrink-0 normal-case px-2.5 py-1" variant="info">
                                  {t('stories.graph.start')}
                                </Badge>
                              ) : null}
                            </div>

                            <div className="space-y-3">
                              <div>
                                <p className="text-[0.72rem] text-[var(--color-text-muted)]">
                                  {t('stories.graph.scene')}
                                </p>
                                <p className="mt-1 line-clamp-2 text-sm leading-6 text-[var(--color-text-primary)]">
                                  {node.scene}
                                </p>
                              </div>
                              <div>
                                <p className="text-[0.72rem] text-[var(--color-text-muted)]">
                                  {t('stories.graph.goal')}
                                </p>
                                <p className="mt-1 line-clamp-2 text-sm leading-6 text-[var(--color-text-primary)]">
                                  {node.goal}
                                </p>
                              </div>
                              <div>
                                <p className="text-[0.72rem] text-[var(--color-text-muted)]">
                                  {t('stories.graph.characters')}
                                </p>
                                <p className="mt-1 line-clamp-2 text-sm leading-6 text-[var(--color-text-secondary)]">
                                  {summarizeCharacters(
                                    node.characters,
                                    (names, count) =>
                                      t('stories.graph.charactersSummary', { count, names }),
                                    noCharactersLabel,
                                  )}
                                </p>
                              </div>
                            </div>
                          </div>
                        </div>
                      </div>
                    )
                  })}
                </div>
              </div>
            </div>
          )}

          <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
            {t('stories.graph.hint')}
          </p>
        </DialogBody>

        <DialogFooter className="justify-end">
          <DialogClose asChild>
            <Button variant="secondary">{t('stories.actions.close')}</Button>
          </DialogClose>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
