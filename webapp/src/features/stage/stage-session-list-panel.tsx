import { faPen } from '@fortawesome/free-solid-svg-icons/faPen'
import { faPlus } from '@fortawesome/free-solid-svg-icons/faPlus'
import { faRotateRight } from '@fortawesome/free-solid-svg-icons/faRotateRight'
import { faTrashCan } from '@fortawesome/free-solid-svg-icons/faTrashCan'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'

import { Card, CardContent } from '../../components/ui/card'
import { IconButton } from '../../components/ui/icon-button'
import { WorkspacePanelShell } from '../../components/layout/workspace-panel-shell'
import { cn } from '../../lib/cn'
import type { StageCopy } from './copy'
import { SessionListSkeleton, StagePanelHeader } from './stage-panel-shared'
import type { SessionSummary } from './types'

function summarizeSessionText(text: string, maxLength = 88) {
  const normalized = text.replace(/\s+/g, ' ').trim()

  if (normalized.length <= maxLength) {
    return normalized
  }

  return `${normalized.slice(0, maxLength).trimEnd()}…`
}

function formatSessionTime(
  dateFormatter: Intl.DateTimeFormat,
  copy: StageCopy,
  session: SessionSummary,
) {
  const timeValue = session.updated_at_ms ?? session.created_at_ms

  if (!timeValue) {
    return copy.time.unknown
  }

  return dateFormatter.format(timeValue)
}

type SessionListStory = {
  introduction?: string | null
}

type StageSessionListPanelProps = {
  copy: StageCopy
  dateFormatter: Intl.DateTimeFormat
  isListLoading: boolean
  isRefreshingList: boolean
  onDeleteSession: (session: SessionSummary) => void
  onEditSession: (session: SessionSummary) => void
  onRefreshSessions: () => Promise<void> | void
  onSelectSession: (sessionId: string) => void
  onStartSession: () => void
  routeSessionId?: string
  sessions: ReadonlyArray<SessionSummary>
  storiesById: Map<string, SessionListStory>
}

export function StageSessionListPanel({
  copy,
  dateFormatter,
  isListLoading,
  isRefreshingList,
  onDeleteSession,
  onEditSession,
  onRefreshSessions,
  onSelectSession,
  onStartSession,
  routeSessionId,
  sessions,
  storiesById,
}: StageSessionListPanelProps) {
  return (
    <WorkspacePanelShell className="h-full min-h-0">
      <Card className="flex h-full min-h-0 flex-col overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_94%,transparent)] shadow-none">
        <StagePanelHeader
          actions={
            <>
              <IconButton
                disabled={isRefreshingList}
                icon={
                  <FontAwesomeIcon
                    className={cn(isRefreshingList ? 'animate-spin' : '')}
                    icon={faRotateRight}
                  />
                }
                label={copy.list.refresh}
                onClick={() => void onRefreshSessions()}
                variant="ghost"
              />
              <IconButton
                icon={<FontAwesomeIcon icon={faPlus} />}
                label={copy.createSession.title}
                onClick={onStartSession}
              />
            </>
          }
          title={copy.list.section}
          titleClassName="text-[1.35rem]"
        />

        <CardContent className="min-h-0 flex-1 overflow-y-auto pt-5">
          <div className="space-y-4 pr-1">
            {isListLoading ? (
              <SessionListSkeleton />
            ) : sessions.length === 0 ? (
              <div className="rounded-[1.45rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_72%,transparent)] px-4 py-5 text-sm leading-7 text-[var(--color-text-secondary)]">
                {copy.list.empty}
              </div>
            ) : (
              <div className="space-y-3">
                {sessions.map((session) => {
                  const story = storiesById.get(session.story_id)
                  const isActive = session.session_id === routeSessionId
                  const timeText = formatSessionTime(dateFormatter, copy, session)

                  return (
                    <div
                      className={cn(
                        'w-full rounded-[1.4rem] border px-4 py-4 text-left transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-focus-ring)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--color-bg-canvas)]',
                        isActive
                          ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] text-[var(--color-text-primary)] shadow-[0_18px_40px_var(--color-accent-glow-soft)]'
                          : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] text-[var(--color-text-secondary)] hover:border-[var(--color-accent-copper-soft)] hover:text-[var(--color-text-primary)]',
                      )}
                      key={session.session_id}
                      onClick={() => {
                        onSelectSession(session.session_id)
                      }}
                      onKeyDown={(event) => {
                        if (event.key === 'Enter' || event.key === ' ') {
                          event.preventDefault()
                          onSelectSession(session.session_id)
                        }
                      }}
                      role="button"
                      tabIndex={0}
                    >
                      <div className="flex items-start justify-between gap-3">
                        <div className="min-w-0 space-y-2">
                          <p className="truncate font-display text-[1.05rem] leading-tight">
                            {session.display_name}
                          </p>
                          <p className="text-xs text-[var(--color-text-muted)]">{timeText}</p>
                          <p className="line-clamp-2 text-sm leading-6">
                            {story?.introduction
                              ? summarizeSessionText(story.introduction, 88)
                              : copy.list.untitledStory}
                          </p>
                        </div>
                        <IconButton
                          className="shrink-0"
                          icon={<FontAwesomeIcon icon={faPen} />}
                          label={copy.editSession}
                          onClick={(event) => {
                            event.stopPropagation()
                            onEditSession(session)
                          }}
                          size="sm"
                          variant="secondary"
                        />
                        <IconButton
                          className="shrink-0"
                          icon={<FontAwesomeIcon icon={faTrashCan} />}
                          label={copy.deleteSession.title}
                          onClick={(event) => {
                            event.stopPropagation()
                            onDeleteSession(session)
                          }}
                          size="sm"
                          variant="danger"
                        />
                      </div>
                    </div>
                  )
                })}
              </div>
            )}
          </div>
        </CardContent>
      </Card>
    </WorkspacePanelShell>
  )
}
