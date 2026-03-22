import type { CharacterSummary } from '../characters/types'
import { StoryGraphCanvas } from './story-graph-canvas'
import type { StoryGraphEditorController } from './story-graph-editor-controller'
import { GRAPH_MAX_ZOOM, GRAPH_MIN_ZOOM, GRAPH_ZOOM_STEP } from './story-graph-editor-utils'
import { StoryGraphInspector } from './story-graph-inspector'
import { StoryGraphToolbar } from './story-graph-toolbar'
import type { StoryGraph } from './types'

type StoryGraphEditorWorkspaceProps = {
  availableCharacters: ReadonlyArray<CharacterSummary>
  controller: StoryGraphEditorController
  graphType: 'draft' | 'story'
  isFullscreen?: boolean
  onToggleFullscreen?: () => void
  playerSchemaId?: string | null
  readOnly?: boolean
  showFullscreenToggle?: boolean
  worldSchemaId?: string | null
}

export function StoryGraphEditorWorkspace({
  availableCharacters,
  controller,
  graphType,
  isFullscreen = false,
  onToggleFullscreen,
  playerSchemaId,
  readOnly = false,
  showFullscreenToggle = true,
  worldSchemaId,
}: StoryGraphEditorWorkspaceProps) {
  if (!controller.graphDraft) {
    return null
  }

  const graphDraft: StoryGraph = controller.graphDraft

  return (
    <>
      <StoryGraphToolbar
        graphType={graphType}
        isFullscreen={isFullscreen}
        nodeCount={graphDraft.nodes.length}
        onAddNode={controller.handleAddNode}
        onResetView={controller.handleResetView}
        onToggleFullscreen={() => {
          onToggleFullscreen?.()
        }}
        onZoomIn={() => {
          controller.setViewport((currentViewport) => ({
            ...currentViewport,
            zoom: Math.min(
              GRAPH_MAX_ZOOM,
              Number((currentViewport.zoom + GRAPH_ZOOM_STEP).toFixed(2)),
            ),
          }))
        }}
        onZoomOut={() => {
          controller.setViewport((currentViewport) => ({
            ...currentViewport,
            zoom: Math.max(
              GRAPH_MIN_ZOOM,
              Number((currentViewport.zoom - GRAPH_ZOOM_STEP).toFixed(2)),
            ),
          }))
        }}
        readOnly={readOnly}
        showFullscreenToggle={showFullscreenToggle}
      />

      <div className="flex min-h-0 flex-1 gap-4 overflow-hidden px-6 pb-6 pt-4 md:px-7 md:pb-7">
        <StoryGraphCanvas
          graph={graphDraft}
          nodePositions={controller.nodePositions}
          onNodePositionChange={controller.setNodePosition}
          onSelectNode={controller.handleSelectNode}
          onViewportChange={controller.setViewport}
          readOnly={readOnly}
          selectedEdgeId={controller.selectedEdgeId}
          selectedNodeId={controller.selectedNodeId}
          viewport={controller.viewport}
        />

        <div className="min-h-0 w-[24rem] overflow-hidden rounded-[1.6rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel-strong)]">
          <StoryGraphInspector
            availableCharacters={availableCharacters}
            conditionDrafts={controller.conditionDrafts}
            graph={graphDraft}
            newNodeIds={controller.newNodeIds}
            onAddOnEnterUpdate={controller.handleAddOnEnterUpdate}
            onAddTransition={controller.handleAddTransition}
            onConditionDraftChange={controller.setConditionDraftValue}
            onDeleteNode={controller.handleDeleteNode}
            onOnEnterUpdateDraftChange={controller.setOnEnterUpdateDraftValue}
            onRemoveTransition={controller.handleRemoveTransition}
            onRemoveOnEnterUpdate={controller.handleRemoveOnEnterUpdate}
            onSelectTransition={controller.setSelectedTransitionIndex}
            onSetStartNode={(nodeId) => {
              controller.patchGraph((currentGraph) => ({
                ...currentGraph,
                start_node: nodeId,
              }))
            }}
            onToggleCondition={controller.handleToggleCondition}
            onUpdateCharacters={controller.handleUpdateCharacters}
            onUpdateNodeField={controller.handleUpdateNodeField}
            onUpdateNodeId={controller.handleUpdateNodeId}
            onUpdateOnEnterUpdate={controller.handleUpdateOnEnterUpdate}
            onUpdateTransition={controller.handleUpdateTransition}
            onEnterUpdateDrafts={controller.onEnterUpdateDrafts}
            onUpdateTransitionCondition={controller.handleUpdateTransitionCondition}
            playerSchemaId={playerSchemaId}
            readOnly={readOnly}
            selectedNodeId={controller.selectedNodeId}
            selectedTransitionIndex={controller.selectedTransitionIndex}
            worldSchemaId={worldSchemaId}
          />
        </div>
      </div>
    </>
  )
}
