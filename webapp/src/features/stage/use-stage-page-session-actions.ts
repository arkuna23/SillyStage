import { useState } from 'react'
import type { NavigateFunction } from 'react-router-dom'

import { isRpcConflict } from '../../lib/rpc'
import {
  deleteSession,
  deleteSessionCharacter,
  enterSessionCharacterScene,
  getRuntimeSnapshot,
  listSessions,
  leaveSessionCharacterScene,
  setSessionPlayerProfile,
  updateSessionCharacter,
  updateSessionConfig,
  updateSessionPlayerDescription,
} from './api'
import type { StageCopy } from './copy'
import {
  buildStagePath,
  getErrorMessage,
  normalizeSessionHistory,
  patchSnapshotActiveCharacter,
} from './stage-page-utils'
import type { Notice } from './stage-ui-types'
import type {
  RuntimeSnapshot,
  SessionCharacter,
  SessionDetail,
  SessionSummary,
  StartedSession,
  UpdateSessionConfigParams,
} from './types'

type UseStagePageSessionActionsArgs = {
  copy: StageCopy
  deleteTarget: SessionSummary | null
  detailsSessionCharacterId: string | null
  navigate: NavigateFunction
  routeSessionId?: string
  selectedSession: SessionDetail | null
  setDeleteTarget: (target: SessionSummary | null) => void
  setDetailsSessionCharacterId: (sessionCharacterId: string | null) => void
  setLiveSnapshot: (snapshot: RuntimeSnapshot | null | ((current: RuntimeSnapshot | null) => RuntimeSnapshot | null)) => void
  setNotice: (notice: Notice | null) => void
  setSelectedSession: (
    value: SessionDetail | null | ((current: SessionDetail | null) => SessionDetail | null),
  ) => void
  setSessionCharacters: (
    value: SessionCharacter[] | ((current: SessionCharacter[]) => SessionCharacter[]),
  ) => void
  setSessions: (value: SessionSummary[] | ((current: SessionSummary[]) => SessionSummary[])) => void
}

export function useStagePageSessionActions({
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
}: UseStagePageSessionActionsArgs) {
  const [isDeleting, setIsDeleting] = useState(false)
  const [isRefreshingList, setIsRefreshingList] = useState(false)

  async function handleRefreshSessions() {
    setIsRefreshingList(true)

    try {
      const nextSessions = await listSessions()
      setSessions(nextSessions)
    } catch (error) {
      setNotice({
        message: getErrorMessage(error, copy.notice.listFailed),
        tone: 'error',
      })
    } finally {
      setIsRefreshingList(false)
    }
  }

  async function handleSaveSessionConfig(params: UpdateSessionConfigParams) {
    if (!selectedSession) {
      return
    }

    const nextConfig = await updateSessionConfig(selectedSession.session_id, params)
    const runtime = await getRuntimeSnapshot(selectedSession.session_id)
    setSelectedSession((current) =>
      current
        ? {
            ...current,
            config: nextConfig,
            snapshot: runtime.snapshot,
          }
        : current,
    )
    setLiveSnapshot(runtime.snapshot)
  }

  async function handleSetPlayerProfile(playerProfileId: string | null) {
    if (!selectedSession) {
      return
    }

    const session = await setSessionPlayerProfile(selectedSession.session_id, {
      ...(playerProfileId ? { player_profile_id: playerProfileId } : {}),
    })

    setSelectedSession({
      ...session,
      history: normalizeSessionHistory(session.history),
    })
    setLiveSnapshot(session.snapshot)
    setSessions((current) =>
      current.map((entry) =>
        entry.session_id === session.session_id
          ? {
              ...entry,
              display_name: session.display_name,
              player_profile_id: session.player_profile_id,
              player_schema_id: session.player_schema_id,
              turn_index: session.turn_index,
              updated_at_ms: session.updated_at_ms,
            }
          : entry,
      ),
    )
  }

  async function handleUpdatePlayerDescription(playerDescription: string) {
    if (!selectedSession) {
      return
    }

    const result = await updateSessionPlayerDescription(selectedSession.session_id, {
      player_description: playerDescription,
    })

    setLiveSnapshot(result.snapshot)
    setSelectedSession((current) =>
      current
        ? {
            ...current,
            player_profile_id: null,
            snapshot: result.snapshot,
          }
        : current,
    )
    setSessions((current) =>
      current.map((entry) =>
        entry.session_id === selectedSession.session_id
          ? {
              ...entry,
              player_profile_id: null,
            }
          : entry,
      ),
    )
  }

  async function handleRefreshRuntimeSnapshot() {
    if (!selectedSession) {
      return
    }

    const result = await getRuntimeSnapshot(selectedSession.session_id)
    setLiveSnapshot(result.snapshot)
    setSelectedSession((current) =>
      current
        ? {
            ...current,
            snapshot: result.snapshot,
          }
        : current,
    )
  }

  async function handleSaveSessionCharacter(character: SessionCharacter) {
    if (!selectedSession) {
      return
    }

    try {
      const updatedCharacter = await updateSessionCharacter(selectedSession.session_id, {
        display_name: character.display_name,
        personality: character.personality,
        session_character_id: character.session_character_id,
        style: character.style,
        system_prompt: character.system_prompt,
      })

      setSessionCharacters((current) =>
        current.map((entry) =>
          entry.session_character_id === updatedCharacter.session_character_id ? updatedCharacter : entry,
        ),
      )
      setNotice({
        message: copy.notice.sessionCharacterSaved,
        tone: 'success',
      })
    } catch (error) {
      setNotice({
        message: getErrorMessage(error, copy.notice.sessionCharacterUpdateFailed),
        tone: 'error',
      })
    }
  }

  async function handleSetSessionCharacterScene(sessionCharacterId: string, inScene: boolean) {
    if (!selectedSession) {
      return
    }

    try {
      const updatedCharacter = inScene
        ? await enterSessionCharacterScene(selectedSession.session_id, {
            session_character_id: sessionCharacterId,
          })
        : await leaveSessionCharacterScene(selectedSession.session_id, {
            session_character_id: sessionCharacterId,
          })

      setSessionCharacters((current) =>
        current.map((entry) =>
          entry.session_character_id === updatedCharacter.session_character_id ? updatedCharacter : entry,
        ),
      )
      setSelectedSession((current) =>
        current
          ? {
              ...current,
              snapshot: patchSnapshotActiveCharacter(current.snapshot, sessionCharacterId, inScene),
            }
          : current,
      )
      setLiveSnapshot((current) =>
        current ? patchSnapshotActiveCharacter(current, sessionCharacterId, inScene) : current,
      )
    } catch (error) {
      setNotice({
        message: getErrorMessage(
          error,
          inScene ? copy.notice.sessionCharacterEnterFailed : copy.notice.sessionCharacterLeaveFailed,
        ),
        tone: 'error',
      })
    }
  }

  async function handleDeleteSessionCharacter(sessionCharacterId: string) {
    if (!selectedSession) {
      return
    }

    try {
      await deleteSessionCharacter(selectedSession.session_id, {
        session_character_id: sessionCharacterId,
      })

      setSessionCharacters((current) =>
        current.filter((entry) => entry.session_character_id !== sessionCharacterId),
      )
      setSelectedSession((current) =>
        current
          ? {
              ...current,
              snapshot: patchSnapshotActiveCharacter(current.snapshot, sessionCharacterId, false),
            }
          : current,
      )
      setLiveSnapshot((current) =>
        current ? patchSnapshotActiveCharacter(current, sessionCharacterId, false) : current,
      )

      if (detailsSessionCharacterId === sessionCharacterId) {
        setDetailsSessionCharacterId(null)
      }
    } catch (error) {
      setNotice({
        message: getErrorMessage(error, copy.notice.sessionCharacterDeleteFailed),
        tone: 'error',
      })
    }
  }

  async function handleCreateSession(result: { message: string; session: StartedSession }) {
    setNotice({ message: result.message, tone: 'success' })
    const nextSessions = await listSessions()
    setSessions(nextSessions)
    navigate(buildStagePath(result.session.session_id))
  }

  async function handleDeleteSession() {
    if (!deleteTarget) {
      return
    }

    setIsDeleting(true)

    try {
      await deleteSession(deleteTarget.session_id)
      setNotice({
        message: copy.notice.deleted,
        tone: 'success',
      })
      setDeleteTarget(null)
      const nextSessions = await listSessions()
      setSessions(nextSessions)

      if (routeSessionId === deleteTarget.session_id) {
        navigate(buildStagePath())
      }
    } catch (error) {
      setDeleteTarget(null)
      setNotice({
        message: isRpcConflict(error)
          ? copy.notice.deleteFailed
          : getErrorMessage(error, copy.notice.deleteFailed),
        tone: isRpcConflict(error) ? 'warning' : 'error',
      })
    } finally {
      setIsDeleting(false)
    }
  }

  return {
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
  }
}
