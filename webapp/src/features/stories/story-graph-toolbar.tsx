import { useTranslation } from 'react-i18next'

import { Badge } from '../../components/ui/badge'
import { Button } from '../../components/ui/button'

type StoryGraphToolbarProps = {
  graphType: 'draft' | 'story'
  isFullscreen?: boolean
  nodeCount: number
  onAddNode: () => void
  onResetView: () => void
  onToggleFullscreen: () => void
  onZoomIn: () => void
  onZoomOut: () => void
  readOnly?: boolean
  showFullscreenToggle?: boolean
}

export function StoryGraphToolbar({
  graphType,
  isFullscreen = false,
  nodeCount,
  onAddNode,
  onResetView,
  onToggleFullscreen,
  onZoomIn,
  onZoomOut,
  readOnly = false,
  showFullscreenToggle = true,
}: StoryGraphToolbarProps) {
  const { t } = useTranslation()

  return (
    <div className="flex flex-wrap items-center justify-between gap-3 border-b border-[var(--color-border-subtle)] px-6 py-4">
      <div className="flex flex-wrap items-center gap-2">
        <Badge
          className="normal-case px-3 py-1.5"
          variant={graphType === 'draft' ? 'info' : 'subtle'}
        >
          {graphType === 'draft' ? t('stories.graph.partialGraph') : t('stories.graph.finalGraph')}
        </Badge>
        <Badge className="normal-case px-3 py-1.5" variant="subtle">
          {t('stories.details.nodeCount', { count: nodeCount })}
        </Badge>
        {readOnly ? (
          <Badge className="normal-case px-3 py-1.5" variant="gold">
            {t('stories.graph.readOnly')}
          </Badge>
        ) : null}
      </div>

      <div className="flex flex-wrap items-center gap-2">
        <Button onClick={onZoomOut} size="sm" variant="ghost">
          {t('stories.graph.zoomOut')}
        </Button>
        <Button onClick={onZoomIn} size="sm" variant="ghost">
          {t('stories.graph.zoomIn')}
        </Button>
        <Button onClick={onResetView} size="sm" variant="ghost">
          {t('stories.graph.reset')}
        </Button>
        {showFullscreenToggle ? (
          <Button onClick={onToggleFullscreen} size="sm" variant="ghost">
            {isFullscreen ? t('stories.graph.exitFullscreen') : t('stories.graph.fullscreen')}
          </Button>
        ) : null}
        {!readOnly ? (
          <Button onClick={onAddNode} size="sm" variant="secondary">
            {t('stories.graph.addNode')}
          </Button>
        ) : null}
      </div>
    </div>
  )
}
