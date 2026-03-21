import { faRotateRight } from '@fortawesome/free-solid-svg-icons/faRotateRight'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { useEffect, useMemo, useState } from 'react'

import { Button } from '../../components/ui/button'
import { IconButton } from '../../components/ui/icon-button'
import { Select } from '../../components/ui/select'
import { Textarea } from '../../components/ui/textarea'
import { useToastNotice } from '../../components/ui/toast-context'
import type { ApiGroup, Preset } from '../apis/types'
import type { PlayerProfile } from '../player-profiles/types'
import type { StageCopy } from './copy'
import { StagePromptPreviewDialog } from './stage-prompt-preview-dialog'
import type {
  RuntimeSnapshot,
  SessionCharacter,
  SessionConfig,
  UpdateSessionConfigParams,
} from './types'

const NO_PLAYER_PROFILE_OPTION_VALUE = '__none__'

type Notice = {
  message: string
  tone: 'error' | 'success'
}

type StageSessionSettingsPanelProps = {
  actorPreviewCharacterOptions: ReadonlyArray<{
    label: string
    value: string
  }>
  apiGroups: ReadonlyArray<ApiGroup>
  config: SessionConfig
  copy: StageCopy
  currentPlayerProfileId?: string | null
  onRefreshSnapshot: () => Promise<void>
  onSavePlayerDescription: (playerDescription: string) => Promise<void>
  onSavePlayerProfile: (playerProfileId: string | null) => Promise<void>
  onSaveSessionConfig: (params: UpdateSessionConfigParams) => Promise<void>
  onSessionCharacterDelete: (sessionCharacterId: string) => Promise<void> | void
  onSessionCharacterOpen: (sessionCharacterId: string) => void
  onSessionCharacterToggleScene: (
    sessionCharacterId: string,
    inScene: boolean,
  ) => Promise<void> | void
  playerProfiles: ReadonlyArray<PlayerProfile>
  presets: ReadonlyArray<Preset>
  runtimeSnapshot: RuntimeSnapshot | null
  sessionId: string
  sessionCharacters: ReadonlyArray<SessionCharacter>
}

export function StageSessionSettingsPanel({
  actorPreviewCharacterOptions,
  apiGroups,
  config,
  copy,
  currentPlayerProfileId,
  onRefreshSnapshot,
  onSavePlayerDescription,
  onSavePlayerProfile,
  onSaveSessionConfig,
  onSessionCharacterDelete,
  onSessionCharacterOpen,
  onSessionCharacterToggleScene,
  playerProfiles,
  presets,
  runtimeSnapshot,
  sessionId,
  sessionCharacters,
}: StageSessionSettingsPanelProps) {
  const [apiGroupId, setApiGroupId] = useState(config.api_group_id)
  const [presetId, setPresetId] = useState(config.preset_id)
  const [selectedPlayerProfileId, setSelectedPlayerProfileId] = useState(
    currentPlayerProfileId ?? NO_PLAYER_PROFILE_OPTION_VALUE,
  )
  const [manualDescription, setManualDescription] = useState(
    runtimeSnapshot?.player_description ?? '',
  )
  const [notice, setNotice] = useState<Notice | null>(null)
  const [isSavingConfig, setIsSavingConfig] = useState(false)
  const [isSavingPlayerProfile, setIsSavingPlayerProfile] = useState(false)
  const [isSavingDescription, setIsSavingDescription] = useState(false)
  const [isRefreshingSnapshot, setIsRefreshingSnapshot] = useState(false)
  const [isPreviewDialogOpen, setIsPreviewDialogOpen] = useState(false)
  useToastNotice(notice)

  useEffect(() => {
    setApiGroupId(config.api_group_id)
    setPresetId(config.preset_id)
  }, [config.api_group_id, config.preset_id])

  useEffect(() => {
    setSelectedPlayerProfileId(currentPlayerProfileId ?? NO_PLAYER_PROFILE_OPTION_VALUE)
  }, [currentPlayerProfileId])

  useEffect(() => {
    setManualDescription(runtimeSnapshot?.player_description ?? '')
  }, [runtimeSnapshot?.player_description])

  const apiGroupItems = useMemo(
    () =>
      apiGroups.map((apiGroup) => ({
        label: apiGroup.display_name,
        value: apiGroup.api_group_id,
      })),
    [apiGroups],
  )

  const presetItems = useMemo(
    () =>
      presets.map((preset) => ({
        label: preset.display_name,
        value: preset.preset_id,
      })),
    [presets],
  )

  const selectedApiGroup = useMemo(
    () => apiGroups.find((entry) => entry.api_group_id === apiGroupId) ?? null,
    [apiGroupId, apiGroups],
  )
  const selectedPreset = useMemo(
    () => presets.find((entry) => entry.preset_id === presetId) ?? null,
    [presetId, presets],
  )

  const playerProfileItems = useMemo(
    () => [
      {
        label: copy.settings.playerProfile.noProfile,
        value: NO_PLAYER_PROFILE_OPTION_VALUE,
      },
      ...playerProfiles.map((profile) => ({
        label: profile.display_name,
        value: profile.player_profile_id,
      })),
    ],
    [copy.settings.playerProfile.noProfile, playerProfiles],
  )

  const selectedPlayerProfile = useMemo(
    () =>
      selectedPlayerProfileId === NO_PLAYER_PROFILE_OPTION_VALUE
        ? null
        : (playerProfiles.find(
            (profile) => profile.player_profile_id === selectedPlayerProfileId,
          ) ?? null),
    [playerProfiles, selectedPlayerProfileId],
  )

  async function handleSaveConfig() {
    if (!apiGroupId.trim()) {
      setNotice({
        message: copy.settings.bindings.errors.apiGroupRequired,
        tone: 'error',
      })
      return
    }

    if (!presetId.trim()) {
      setNotice({
        message: copy.settings.bindings.errors.presetRequired,
        tone: 'error',
      })
      return
    }

    setNotice(null)
    setIsSavingConfig(true)

    try {
      await onSaveSessionConfig({
        api_group_id: apiGroupId.trim(),
        preset_id: presetId.trim(),
      })
      setNotice({
        message: copy.notice.sessionConfigSaved,
        tone: 'success',
      })
    } catch (error) {
      setNotice({
        message: error instanceof Error ? error.message : copy.notice.updateConfigFailed,
        tone: 'error',
      })
    } finally {
      setIsSavingConfig(false)
    }
  }

  async function handleSavePlayerProfile() {
    setNotice(null)
    setIsSavingPlayerProfile(true)

    try {
      await onSavePlayerProfile(
        selectedPlayerProfileId !== NO_PLAYER_PROFILE_OPTION_VALUE && selectedPlayerProfileId.trim()
          ? selectedPlayerProfileId.trim()
          : null,
      )
      setNotice({
        message: copy.notice.playerProfileSaved,
        tone: 'success',
      })
    } catch (error) {
      setNotice({
        message: error instanceof Error ? error.message : copy.notice.playerProfileFailed,
        tone: 'error',
      })
    } finally {
      setIsSavingPlayerProfile(false)
    }
  }

  async function handleSavePlayerDescription() {
    if (manualDescription.trim().length === 0) {
      setNotice({
        message: copy.settings.playerDescription.errors.required,
        tone: 'error',
      })
      return
    }

    setNotice(null)
    setIsSavingDescription(true)

    try {
      await onSavePlayerDescription(manualDescription.trim())
      setSelectedPlayerProfileId(NO_PLAYER_PROFILE_OPTION_VALUE)
      setNotice({
        message: copy.notice.playerDescriptionSaved,
        tone: 'success',
      })
    } catch (error) {
      setNotice({
        message: error instanceof Error ? error.message : copy.notice.playerDescriptionFailed,
        tone: 'error',
      })
    } finally {
      setIsSavingDescription(false)
    }
  }

  async function handleRefreshSnapshot() {
    setNotice(null)
    setIsRefreshingSnapshot(true)

    try {
      await onRefreshSnapshot()
      setNotice({
        message: copy.notice.snapshotRefreshed,
        tone: 'success',
      })
    } catch (error) {
      setNotice({
        message: error instanceof Error ? error.message : copy.notice.snapshotRefreshFailed,
        tone: 'error',
      })
    } finally {
      setIsRefreshingSnapshot(false)
    }
  }

  return (
    <div className="flex h-full min-h-0 flex-col">
      <StagePromptPreviewDialog
        actorCharacterOptions={actorPreviewCharacterOptions}
        copy={copy}
        onOpenChange={setIsPreviewDialogOpen}
        open={isPreviewDialogOpen}
        presetId={config.preset_id}
        sessionId={sessionId}
      />

      <div className="border-b border-[var(--color-border-subtle)] px-6 py-5 md:px-7">
        <div className="space-y-2">
          <h3 className="font-display text-[1.45rem] leading-tight text-[var(--color-text-primary)]">
            {copy.settings.title}
          </h3>
          <p className="text-sm leading-6 text-[var(--color-text-secondary)]">
            {copy.settings.description}
          </p>
        </div>
      </div>

      <div className="scrollbar-none min-h-0 flex-1 overflow-y-auto px-6 pb-6 pt-6 md:px-7 md:pb-7">
        <div className="space-y-6">
          <section className="space-y-4 rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
            <div className="flex items-start justify-between gap-3">
              <div className="space-y-2">
                <p className="text-sm font-medium text-[var(--color-text-primary)]">
                  {copy.settings.bindings.section}
                </p>
                <p className="text-sm leading-6 text-[var(--color-text-secondary)]">
                  {copy.settings.bindings.description}
                </p>
              </div>
              <Button
                disabled={!config.preset_id.trim()}
                onClick={() => {
                  setIsPreviewDialogOpen(true)
                }}
                size="sm"
                variant="secondary"
              >
                {copy.settings.bindings.preview.open}
              </Button>
            </div>

            <div className="grid gap-4 md:grid-cols-2">
              <label className="space-y-2.5">
                <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                  {copy.settings.bindings.apiGroup}
                </span>
                <Select
                  items={apiGroupItems}
                  placeholder={copy.settings.bindings.apiGroupPlaceholder}
                  textAlign="start"
                  value={apiGroupId}
                  onValueChange={setApiGroupId}
                />
              </label>

              <label className="space-y-2.5">
                <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                  {copy.settings.bindings.preset}
                </span>
                <Select
                  items={presetItems}
                  placeholder={copy.settings.bindings.presetPlaceholder}
                  textAlign="start"
                  value={presetId}
                  onValueChange={setPresetId}
                />
              </label>
            </div>

            <div className="grid gap-3 md:grid-cols-2">
              <div className="rounded-[1.25rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-4">
                <p className="text-xs text-[var(--color-text-muted)]">
                  {copy.settings.bindings.currentApiGroup}
                </p>
                <p className="mt-2 text-sm text-[var(--color-text-primary)]">
                  {selectedApiGroup?.display_name ?? copy.settings.bindings.missing}
                </p>
              </div>
              <div className="rounded-[1.25rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-4">
                <p className="text-xs text-[var(--color-text-muted)]">
                  {copy.settings.bindings.currentPreset}
                </p>
                <p className="mt-2 text-sm text-[var(--color-text-primary)]">
                  {selectedPreset?.display_name ?? copy.settings.bindings.missing}
                </p>
              </div>
            </div>

            <div className="flex justify-end">
              <Button disabled={isSavingConfig} onClick={() => void handleSaveConfig()}>
                {isSavingConfig ? copy.settings.bindings.saving : copy.settings.bindings.save}
              </Button>
            </div>
          </section>

          <section className="space-y-4 rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
            <div className="space-y-2">
              <p className="text-sm font-medium text-[var(--color-text-primary)]">
                {copy.settings.playerProfile.section}
              </p>
              <p className="text-sm leading-6 text-[var(--color-text-secondary)]">
                {copy.settings.playerProfile.description}
              </p>
            </div>

            <label className="space-y-2.5">
              <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                {copy.settings.playerProfile.label}
              </span>
              <Select
                items={playerProfileItems}
                placeholder={copy.settings.playerProfile.placeholder}
                textAlign="start"
                value={selectedPlayerProfileId}
                onValueChange={(value) => {
                  setSelectedPlayerProfileId(value)
                }}
              />
            </label>

            {selectedPlayerProfile ? (
              <div className="mt-2 rounded-[1.25rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-4">
                <p className="text-xs text-[var(--color-text-muted)]">
                  {copy.settings.playerProfile.preview}
                </p>
                <p className="mt-2 text-sm leading-7 text-[var(--color-text-primary)]">
                  {selectedPlayerProfile.description}
                </p>
              </div>
            ) : (
              <div className="mt-2 rounded-[1.25rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-4 text-sm leading-7 text-[var(--color-text-secondary)]">
                {copy.settings.playerProfile.noProfileHint}
              </div>
            )}

            <div className="flex justify-end">
              <Button
                disabled={isSavingPlayerProfile}
                onClick={() => void handleSavePlayerProfile()}
                variant="secondary"
              >
                {isSavingPlayerProfile
                  ? copy.settings.playerProfile.saving
                  : copy.settings.playerProfile.save}
              </Button>
            </div>
          </section>

          <section className="space-y-4 rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
            <div className="space-y-2">
              <p className="text-sm font-medium text-[var(--color-text-primary)]">
                {copy.settings.playerDescription.section}
              </p>
              <p className="text-sm leading-6 text-[var(--color-text-secondary)]">
                {copy.settings.playerDescription.description}
              </p>
            </div>

            <Textarea
              className="min-h-[8.5rem]"
              id="stage-session-player-description"
              name="stage-session-player-description"
              onChange={(event) => {
                setManualDescription(event.target.value)
              }}
              placeholder={copy.settings.playerDescription.placeholder}
              value={manualDescription}
            />

            <div className="flex justify-end">
              <Button
                disabled={isSavingDescription}
                onClick={() => void handleSavePlayerDescription()}
                variant="secondary"
              >
                {isSavingDescription
                  ? copy.settings.playerDescription.saving
                  : copy.settings.playerDescription.save}
              </Button>
            </div>
          </section>

          <section className="space-y-4 rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
            <div className="space-y-2">
              <p className="text-sm font-medium text-[var(--color-text-primary)]">
                {copy.settings.sessionCharacters.section}
              </p>
              <p className="text-sm leading-6 text-[var(--color-text-secondary)]">
                {copy.settings.sessionCharacters.description}
              </p>
            </div>

            {sessionCharacters.length === 0 ? (
              <div className="rounded-[1.25rem] border border-dashed border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-4 text-sm leading-7 text-[var(--color-text-secondary)]">
                {copy.settings.sessionCharacters.empty}
              </div>
            ) : (
              <div className="space-y-3">
                {sessionCharacters.map((character) => (
                  <div
                    className="rounded-[1.25rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-4"
                    key={character.session_character_id}
                  >
                    <div className="flex items-start justify-between gap-3">
                      <div className="min-w-0 space-y-1">
                        <p className="truncate text-sm font-medium text-[var(--color-text-primary)]">
                          {character.display_name}
                        </p>
                        <p className="truncate font-mono text-[0.72rem] text-[var(--color-text-muted)]">
                          {character.session_character_id}
                        </p>
                        <p className="line-clamp-2 text-sm leading-6 text-[var(--color-text-secondary)]">
                          {character.personality}
                        </p>
                      </div>
                      <div className="shrink-0 rounded-full border border-[var(--color-border-subtle)] px-2.5 py-1 text-xs text-[var(--color-text-secondary)]">
                        {character.in_scene
                          ? copy.settings.sessionCharacters.inScene
                          : copy.settings.sessionCharacters.outOfScene}
                      </div>
                    </div>

                    <div className="mt-3 flex flex-wrap justify-end gap-2">
                      <Button
                        onClick={() => {
                          onSessionCharacterOpen(character.session_character_id)
                        }}
                        size="sm"
                        variant="ghost"
                      >
                        {copy.settings.sessionCharacters.view}
                      </Button>
                      <Button
                        onClick={() => {
                          void onSessionCharacterToggleScene(
                            character.session_character_id,
                            !character.in_scene,
                          )
                        }}
                        size="sm"
                        variant="secondary"
                      >
                        {character.in_scene
                          ? copy.settings.sessionCharacters.leaveScene
                          : copy.settings.sessionCharacters.enterScene}
                      </Button>
                      <Button
                        onClick={() => {
                          void onSessionCharacterDelete(character.session_character_id)
                        }}
                        size="sm"
                        variant="danger"
                      >
                        {copy.settings.sessionCharacters.delete}
                      </Button>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </section>

          <section className="space-y-4 rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
            <div className="flex items-start justify-between gap-3">
              <div className="space-y-2">
                <p className="text-sm font-medium text-[var(--color-text-primary)]">
                  {copy.settings.snapshot.section}
                </p>
                <p className="text-sm leading-6 text-[var(--color-text-secondary)]">
                  {copy.settings.snapshot.description}
                </p>
              </div>
              <IconButton
                icon={
                  <FontAwesomeIcon
                    className={isRefreshingSnapshot ? 'animate-spin' : ''}
                    icon={faRotateRight}
                  />
                }
                label={copy.settings.snapshot.refresh}
                onClick={() => void handleRefreshSnapshot()}
                size="sm"
                variant="ghost"
              />
            </div>

            <div className="grid gap-3 md:grid-cols-2">
              <div className="rounded-[1.25rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-4">
                <p className="text-xs text-[var(--color-text-muted)]">
                  {copy.settings.snapshot.turnIndex}
                </p>
                <p className="mt-2 text-sm text-[var(--color-text-primary)]">
                  {runtimeSnapshot?.turn_index ?? '—'}
                </p>
              </div>
              <div className="rounded-[1.25rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-4">
                <p className="text-xs text-[var(--color-text-muted)]">
                  {copy.settings.snapshot.currentNode}
                </p>
                <p className="mt-2 text-sm text-[var(--color-text-primary)]">
                  {runtimeSnapshot?.world_state.current_node || '—'}
                </p>
              </div>
              <div className="rounded-[1.25rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-4">
                <p className="text-xs text-[var(--color-text-muted)]">
                  {copy.settings.snapshot.activeCharacters}
                </p>
                <p className="mt-2 text-sm text-[var(--color-text-primary)]">
                  {runtimeSnapshot?.world_state.active_characters.length ?? 0}
                </p>
              </div>
              <div className="rounded-[1.25rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-4">
                <p className="text-xs text-[var(--color-text-muted)]">
                  {copy.settings.snapshot.playerStateKeys}
                </p>
                <p className="mt-2 text-sm text-[var(--color-text-primary)]">
                  {runtimeSnapshot
                    ? Object.keys(runtimeSnapshot.world_state.player_state).length
                    : '—'}
                </p>
              </div>
            </div>

            <div className="rounded-[1.25rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-4">
              <p className="text-xs text-[var(--color-text-muted)]">
                {copy.settings.snapshot.playerDescription}
              </p>
              <p className="mt-2 text-sm leading-7 text-[var(--color-text-primary)]">
                {runtimeSnapshot?.player_description?.trim() || '—'}
              </p>
            </div>
          </section>
        </div>
      </div>
    </div>
  )
}
