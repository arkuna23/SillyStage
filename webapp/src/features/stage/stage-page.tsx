import { faComments } from '@fortawesome/free-solid-svg-icons/faComments'
import { faDatabase } from '@fortawesome/free-solid-svg-icons/faDatabase'
import { faPlug } from '@fortawesome/free-solid-svg-icons/faPlug'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { useReducedMotion } from 'framer-motion'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useNavigate, useParams } from 'react-router-dom'

import { appPaths } from '../../app/paths'
import { WorkspacePanelShell } from '../../components/layout/workspace-panel-shell'
import { Card, CardContent } from '../../components/ui/card'
import { SegmentedSelector } from '../../components/ui/segmented-selector'
import { useToastNotice } from '../../components/ui/toast-context'
import { CharacterDetailsDialog } from '../characters/character-details-dialog'
import { SessionCharacterDialog } from './session-character-dialog'
import { SessionDeleteDialog } from './session-delete-dialog'
import { SessionRenameDialog } from './session-rename-dialog'
import { SessionStartDialog } from './session-start-dialog'
import { getStageCopy } from './copy'
import {
  buildStagePath,
  determineActiveCastOrder,
  getDefaultRightPanelTab,
  isTextLong,
  normalizeSessionHistory,
  patchSnapshotVariables,
  summarizeText,
} from './stage-page-utils'
import { StageCharacterVariablesPanel } from './stage-character-variables-panel'
import { StageDialoguePanel } from './stage-dialogue-panel'
import { StagePanelHeader } from './stage-panel-shared'
import { StageRightPanel } from './stage-right-panel'
import { StageSessionListPanel } from './stage-session-list-panel'
import { StageSessionSettingsPanel } from './stage-session-settings-panel'
import { StageSessionVariablesPanel } from './stage-session-variables-panel'
import type { Notice, PanelMode, StageCastMember, StageRightRailTab } from './stage-ui-types'
import type { SessionDetail, SessionSummary, SessionVariables } from './types'
import { useStagePageData } from './use-stage-page-data'
import { useStagePageSessionActions } from './use-stage-page-session-actions'
import { useStagePageTurn } from './use-stage-page-turn'

export function StagePage() {
  const navigate = useNavigate()
  const { i18n } = useTranslation()
  const { sessionId: routeSessionId } = useParams<{ sessionId: string }>()
  const copy = getStageCopy(i18n.language)
  const prefersReducedMotion = useReducedMotion()
  const autoResolvedRightPanelTabSessionRef = useRef<string | null>(null)
  const [panelMode, setPanelMode] = useState<PanelMode>('dialogue')
  const [isStoryIntroExpanded, setIsStoryIntroExpanded] = useState(false)
  const [isStoryNodeExpanded, setIsStoryNodeExpanded] = useState(false)
  const [rightPanelTab, setRightPanelTab] = useState<StageRightRailTab>('status')
  const [detailsCharacterId, setDetailsCharacterId] = useState<string | null>(null)
  const [detailsSessionCharacterId, setDetailsSessionCharacterId] = useState<string | null>(null)
  const [notice, setNotice] = useState<Notice | null>(null)
  const [isStartDialogOpen, setIsStartDialogOpen] = useState(false)
  const [deleteTarget, setDeleteTarget] = useState<SessionSummary | null>(null)
  const [renameTarget, setRenameTarget] = useState<SessionSummary | SessionDetail | null>(null)
  useToastNotice(notice)

  const dateFormatter = useMemo(
    () =>
      new Intl.DateTimeFormat(i18n.language.startsWith('zh') ? 'zh-CN' : 'en', {
        dateStyle: 'medium',
        timeStyle: 'short',
      }),
    [i18n.language],
  )

  const {
    apiGroups,
    apis,
    characterMap,
    coverCache,
    currentNode,
    currentSnapshot,
    isListLoading,
    isSessionLoading,
    playerProfiles,
    presets,
    selectedSession,
    selectedStoryDetail,
    sessionCharacterMap,
    sessionCharacters,
    sessions,
    setLiveSnapshot,
    setSelectedSession,
    setSessionCharacters,
    setSessions,
    stageAccessStatus,
    stageCommonVariables,
    stories,
    storiesById,
  } = useStagePageData({
    copy,
    routeSessionId,
    setNotice,
  })

  const {
    handleCreateSession,
    handleDeleteSession,
    handleDeleteSessionCharacter,
    handleRefreshRuntimeSnapshot,
    handleRefreshSessions,
    handleSaveSessionCharacter,
    handleSaveSessionConfig,
    handleSetPlayerProfile,
    handleSetSessionCharacterScene,
    handleUpdatePlayerDescription,
    isDeleting,
    isRefreshingList,
  } = useStagePageSessionActions({
    copy,
    deleteTarget,
    detailsSessionCharacterId,
    navigate,
    routeSessionId,
    selectedSession,
    setDeleteTarget,
    setDetailsSessionCharacterId,
    setLiveSnapshot,
    setNotice,
    setSelectedSession,
    setSessionCharacters,
    setSessions,
  })

  const {
    activeSpeakerId,
    beatSpeakerIds,
    composerInput,
    composerMode,
    composerRef,
    conversationScrollRef,
    deletingPlayerMessageId,
    editingPlayerDraft,
    editingPlayerMessageId,
    expandedThoughtIds,
    handleCancelEditPlayerMessage,
    handleDeletePlayerMessage,
    handleEditPlayerMessage,
    handleRunTurn,
    handleSaveEditedPlayerMessage,
    handleSuggestReplies,
    handleToggleReplySuggestions,
    handleToggleThought,
    handleUseReplySuggestion,
    isRunningTurn,
    isSuggestingReplies,
    overlayStatus,
    replySuggestions,
    replySuggestionsEnabled,
    savingPlayerMessageId,
    sessionMessages,
    setComposerInput,
    setEditingPlayerDraft,
    suggestionsError,
    updateConversationScrollState,
  } = useStagePageTurn({
    characterMap,
    copy,
    routeSessionId,
    selectedSession,
    sessionCharacterMap,
    setLiveSnapshot,
    setNotice,
    setSelectedSession,
    setSessionCharacters,
    setSessions,
  })

  const selectedStageCharacter = useMemo(
    () => (detailsCharacterId ? characterMap.get(detailsCharacterId) ?? null : null),
    [characterMap, detailsCharacterId],
  )
  const selectedSessionCharacter = useMemo(
    () =>
      detailsSessionCharacterId
        ? sessionCharacterMap.get(detailsSessionCharacterId) ?? null
        : null,
    [detailsSessionCharacterId, sessionCharacterMap],
  )
  const orderedActiveCastIds = useMemo(
    () =>
      determineActiveCastOrder({
        activeCharacterIds: currentSnapshot?.world_state.active_characters ?? [],
        beatSpeakerIds,
        currentSpeakerId: activeSpeakerId,
      }),
    [activeSpeakerId, beatSpeakerIds, currentSnapshot],
  )
  const activeCast = useMemo<StageCastMember[]>(
    () =>
      orderedActiveCastIds.map((characterId) => {
        const character = characterMap.get(characterId)
        const sessionCharacter = sessionCharacterMap.get(characterId)

        if (sessionCharacter) {
          return {
            id: characterId,
            isSessionCharacter: true,
            name: sessionCharacter.display_name,
          }
        }

        return {
          coverUrl: coverCache[characterId],
          id: characterId,
          isSessionCharacter: false,
          name: character?.name ?? characterId,
        }
      }),
    [characterMap, coverCache, orderedActiveCastIds, sessionCharacterMap],
  )

  const storyIntroduction = selectedStoryDetail?.introduction?.trim() ?? ''
  const storyIntroNeedsExpand = isTextLong(storyIntroduction, 140)
  const visibleStoryIntroduction =
    storyIntroduction.length === 0
      ? copy.intro.empty
      : isStoryIntroExpanded || !storyIntroNeedsExpand
        ? storyIntroduction
        : summarizeText(storyIntroduction, 140)
  const hasExpandableNodeDetails = Boolean(
    currentSnapshot?.world_state.current_node || currentNode?.scene,
  )

  const handleVariablesApplied = useCallback(
    (variables: SessionVariables) => {
      setSelectedSession((current) =>
        current
          ? {
              ...current,
              snapshot: patchSnapshotVariables(current.snapshot, variables),
            }
          : current,
      )
      setLiveSnapshot((current) => (current ? patchSnapshotVariables(current, variables) : current))
    },
    [setLiveSnapshot, setSelectedSession],
  )

  useEffect(() => {
    if (stageAccessStatus === 'blockedApiResources') {
      navigate(appPaths.apis, { replace: true })
      return
    }

    if (stageAccessStatus === 'blockedPresets') {
      navigate(appPaths.presets, { replace: true })
    }
  }, [navigate, stageAccessStatus])

  useEffect(() => {
    autoResolvedRightPanelTabSessionRef.current = null

    const frame = requestAnimationFrame(() => {
      setIsStoryIntroExpanded(false)
      setIsStoryNodeExpanded(false)
      setRightPanelTab(getDefaultRightPanelTab(false))
      setDetailsCharacterId(null)
      setDetailsSessionCharacterId(null)
    })

    return () => {
      cancelAnimationFrame(frame)
    }
  }, [routeSessionId])

  useEffect(() => {
    if (!routeSessionId || !selectedSession) {
      return
    }

    if (selectedSession.session_id !== routeSessionId) {
      return
    }

    if (autoResolvedRightPanelTabSessionRef.current === routeSessionId) {
      return
    }

    if (isSessionLoading) {
      return
    }

    const frame = requestAnimationFrame(() => {
      setRightPanelTab(getDefaultRightPanelTab(stageCommonVariables.length > 0))
      autoResolvedRightPanelTabSessionRef.current = routeSessionId
    })

    return () => {
      cancelAnimationFrame(frame)
    }
  }, [isSessionLoading, routeSessionId, selectedSession, stageCommonVariables.length])

  function selectSession(sessionId: string) {
    navigate(buildStagePath(sessionId))
  }

  return (
    <section className="flex h-full min-h-0 w-full flex-1 overflow-visible py-6 sm:py-8">
      <SessionStartDialog
        apis={apis}
        apiGroups={apiGroups}
        onCompleted={(result) => void handleCreateSession(result)}
        onOpenChange={setIsStartDialogOpen}
        open={isStartDialogOpen}
        playerProfiles={playerProfiles}
        presets={presets}
        stories={stories}
      />

      <SessionDeleteDialog
        copy={copy}
        deleting={isDeleting}
        onConfirm={() => void handleDeleteSession()}
        onOpenChange={(open) => {
          if (!open) {
            setDeleteTarget(null)
          }
        }}
        open={deleteTarget !== null}
        session={deleteTarget}
      />

      <SessionRenameDialog
        copy={copy}
        onCompleted={async (session) => {
          setSessions((current) =>
            current.map((entry) =>
              entry.session_id === session.session_id
                ? {
                    ...entry,
                    created_at_ms: session.created_at_ms,
                    display_name: session.display_name,
                    player_profile_id: session.player_profile_id,
                    player_schema_id: session.player_schema_id,
                    story_id: session.story_id,
                    turn_index: session.turn_index,
                    updated_at_ms: session.updated_at_ms,
                  }
                : entry,
            ),
          )
          setSelectedSession((current) =>
            current?.session_id === session.session_id
              ? {
                  ...session,
                  history: normalizeSessionHistory(session.history),
                }
              : current,
          )
          setNotice({
            message: copy.notice.sessionRenamed,
            tone: 'success',
          })
        }}
        onOpenChange={(open) => {
          if (!open) {
            setRenameTarget(null)
          }
        }}
        open={renameTarget !== null}
        session={renameTarget}
      />

      <CharacterDetailsDialog
        coverUrl={detailsCharacterId ? coverCache[detailsCharacterId] ?? undefined : undefined}
        exporting={false}
        onOpenChange={(open) => {
          if (!open) {
            setDetailsCharacterId(null)
          }
        }}
        open={detailsCharacterId !== null}
        showActions={false}
        stageTabs={
          selectedStageCharacter && selectedSession
            ? {
                detailsLabel: copy.characterDialog.details,
                variablesContent: (
                  <StageCharacterVariablesPanel
                    character={selectedStageCharacter}
                    copy={copy}
                    onVariablesApplied={handleVariablesApplied}
                    runtimeSnapshot={currentSnapshot}
                    sessionId={selectedSession.session_id}
                  />
                ),
                variablesLabel: copy.characterDialog.variables,
              }
            : undefined
        }
        summary={selectedStageCharacter}
      />

      <SessionCharacterDialog
        character={selectedSessionCharacter}
        copy={copy}
        onDelete={(sessionCharacterId) => {
          void handleDeleteSessionCharacter(sessionCharacterId)
        }}
        onOpenChange={(open) => {
          if (!open) {
            setDetailsSessionCharacterId(null)
          }
        }}
        onSave={(character) => {
          void handleSaveSessionCharacter(character)
        }}
        onToggleScene={(sessionCharacterId, inScene) => {
          void handleSetSessionCharacterScene(sessionCharacterId, inScene)
        }}
        open={detailsSessionCharacterId !== null}
      />

      <div className="grid h-full min-h-0 w-full gap-5 overflow-visible lg:grid-cols-[17rem_minmax(0,1fr)_18rem]">
        <StageSessionListPanel
          copy={copy}
          dateFormatter={dateFormatter}
          isListLoading={isListLoading}
          isRefreshingList={isRefreshingList}
          onDeleteSession={setDeleteTarget}
          onEditSession={setRenameTarget}
          onRefreshSessions={() => void handleRefreshSessions()}
          onSelectSession={selectSession}
          onStartSession={() => {
            setIsStartDialogOpen(true)
          }}
          routeSessionId={routeSessionId}
          sessions={sessions}
          storiesById={storiesById}
        />

        <WorkspacePanelShell className="h-full min-h-0">
          <Card className="flex h-full min-h-0 flex-col overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_94%,transparent)] shadow-none">
            <StagePanelHeader
              actions={
                <SegmentedSelector
                  ariaLabel={copy.stage.title}
                  className="shrink-0"
                  items={[
                    {
                      icon: <FontAwesomeIcon icon={faComments} />,
                      label: copy.tabs.dialogue,
                      value: 'dialogue',
                    },
                    {
                      icon: <FontAwesomeIcon icon={faPlug} />,
                      label: copy.tabs.settings,
                      value: 'settings',
                    },
                    {
                      icon: <FontAwesomeIcon icon={faDatabase} />,
                      label: copy.tabs.variables,
                      value: 'variables',
                    },
                  ]}
                  onValueChange={(value) => {
                    setPanelMode(value as PanelMode)
                  }}
                  value={panelMode}
                />
              }
              title={selectedSession?.display_name ?? copy.stage.title}
              titleClassName="text-[1.95rem]"
            />

            <CardContent className="min-h-0 flex-1 pt-6">
              {panelMode === 'settings' ? (
                selectedSession ? (
                  <div className="scrollbar-none h-full overflow-y-auto pr-1">
                    <StageSessionSettingsPanel
                      apiGroups={apiGroups}
                      config={selectedSession.config}
                      copy={copy}
                      currentPlayerProfileId={selectedSession.player_profile_id}
                      onRefreshSnapshot={handleRefreshRuntimeSnapshot}
                      onSavePlayerDescription={handleUpdatePlayerDescription}
                      onSavePlayerProfile={handleSetPlayerProfile}
                      onSaveSessionConfig={handleSaveSessionConfig}
                      onSessionCharacterDelete={(sessionCharacterId) => {
                        void handleDeleteSessionCharacter(sessionCharacterId)
                      }}
                      onSessionCharacterOpen={setDetailsSessionCharacterId}
                      onSessionCharacterToggleScene={(sessionCharacterId, inScene) => {
                        void handleSetSessionCharacterScene(sessionCharacterId, inScene)
                      }}
                      playerProfiles={playerProfiles}
                      presets={presets}
                      runtimeSnapshot={currentSnapshot}
                      sessionCharacters={sessionCharacters}
                    />
                  </div>
                ) : (
                  <div className="rounded-[1.45rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_72%,transparent)] px-5 py-6 text-sm leading-7 text-[var(--color-text-secondary)]">
                    {copy.empty.stage}
                  </div>
                )
              ) : panelMode === 'variables' ? (
                selectedSession ? (
                  <div className="scrollbar-none h-full overflow-y-auto pr-1">
                    <StageSessionVariablesPanel
                      characterMap={characterMap}
                      copy={copy}
                      onVariablesApplied={handleVariablesApplied}
                      runtimeSnapshot={currentSnapshot}
                      sessionId={selectedSession.session_id}
                      story={selectedStoryDetail}
                    />
                  </div>
                ) : (
                  <div className="rounded-[1.45rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_72%,transparent)] px-5 py-6 text-sm leading-7 text-[var(--color-text-secondary)]">
                    {copy.variables.empty}
                  </div>
                )
              ) : (
                <StageDialoguePanel
                  characterCovers={coverCache}
                  characterMap={characterMap}
                  composerInput={composerInput}
                  composerLocked={isRunningTurn}
                  composerMode={composerMode}
                  composerRef={composerRef}
                  conversationScrollRef={conversationScrollRef}
                  copy={copy}
                  deletingPlayerMessageId={deletingPlayerMessageId}
                  editingPlayerDraft={editingPlayerDraft}
                  editingPlayerMessageId={editingPlayerMessageId}
                  expandedThoughtIds={expandedThoughtIds}
                  isLoading={Boolean(routeSessionId) && isSessionLoading}
                  isRunningTurn={isRunningTurn}
                  isSuggestingReplies={isSuggestingReplies}
                  messages={sessionMessages}
                  onCancelEditPlayerMessage={handleCancelEditPlayerMessage}
                  onChangeComposerInput={setComposerInput}
                  onChangePlayerMessageDraft={setEditingPlayerDraft}
                  onDeletePlayerMessage={(message) => void handleDeletePlayerMessage(message)}
                  onEditPlayerMessage={handleEditPlayerMessage}
                  onGenerateReplySuggestions={() => void handleSuggestReplies()}
                  onRunTurn={() => void handleRunTurn()}
                  onSavePlayerMessage={() => void handleSaveEditedPlayerMessage()}
                  onScrollConversation={updateConversationScrollState}
                  onSelectReplySuggestion={handleUseReplySuggestion}
                  onToggleReplySuggestions={handleToggleReplySuggestions}
                  onToggleThought={handleToggleThought}
                  overlayStatus={overlayStatus}
                  prefersReducedMotion={prefersReducedMotion}
                  replySuggestions={replySuggestions}
                  replySuggestionsEnabled={replySuggestionsEnabled}
                  savingPlayerMessageId={savingPlayerMessageId}
                  selectedSessionExists={Boolean(selectedSession)}
                  suggestionsError={suggestionsError}
                />
              )}
            </CardContent>
          </Card>
        </WorkspacePanelShell>

        <StageRightPanel
          activeCast={activeCast}
          activeSpeakerId={activeSpeakerId}
          commonVariables={stageCommonVariables}
          copy={copy}
          currentNode={currentNode}
          currentSnapshot={currentSnapshot}
          hasExpandableNodeDetails={hasExpandableNodeDetails}
          isStoryIntroExpanded={isStoryIntroExpanded}
          isStoryNodeExpanded={isStoryNodeExpanded}
          onChangeRailTab={setRightPanelTab}
          onOpenCharacter={setDetailsCharacterId}
          onOpenSessionCharacter={setDetailsSessionCharacterId}
          onToggleStoryIntro={() => {
            setIsStoryIntroExpanded((current) => !current)
          }}
          onToggleStoryNode={() => {
            setIsStoryNodeExpanded((current) => !current)
          }}
          prefersReducedMotion={prefersReducedMotion}
          railTab={rightPanelTab}
          storyIntroNeedsExpand={storyIntroNeedsExpand}
          visibleStoryIntroduction={visibleStoryIntroduction}
        />
      </div>
    </section>
  )
}
