import { faRotateRight } from '@fortawesome/free-solid-svg-icons/faRotateRight'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { useEffect, useMemo, useState } from 'react'

import { Button } from '../../components/ui/button'
import { IconButton } from '../../components/ui/icon-button'
import { Select } from '../../components/ui/select'
import { Textarea } from '../../components/ui/textarea'
import { cn } from '../../lib/cn'
import { agentApiRoleKeys, type AgentApiIds, type LlmApi } from '../apis/types'
import type { PlayerProfile } from '../player-profiles/types'
import type {
  RuntimeSnapshot,
  SessionConfig,
  SessionConfigMode,
  UpdateSessionConfigParams,
} from './types'
import type { StageCopy } from './copy'

const NO_PLAYER_PROFILE_OPTION_VALUE = '__none__'

type Notice = {
  message: string
  tone: 'error' | 'success'
}

type StageSessionSettingsPanelProps = {
  apis: ReadonlyArray<LlmApi>
  config: SessionConfig
  copy: StageCopy
  currentPlayerProfileId?: string | null
  onRefreshSnapshot: () => Promise<void>
  onSavePlayerDescription: (playerDescription: string) => Promise<void>
  onSavePlayerProfile: (playerProfileId: string | null) => Promise<void>
  onSaveSessionConfig: (params: UpdateSessionConfigParams) => Promise<void>
  playerProfiles: ReadonlyArray<PlayerProfile>
  runtimeSnapshot: RuntimeSnapshot | null
}

function createSessionApiDefaults(config: SessionConfig, apis: ReadonlyArray<LlmApi>) {
  const base = config.session_api_ids ?? config.effective_api_ids
  const fallbackApiId = apis[0]?.api_id

  if (base) {
    return base
  }

  if (!fallbackApiId) {
    return null
  }

  return {
    actor_api_id: fallbackApiId,
    architect_api_id: fallbackApiId,
    director_api_id: fallbackApiId,
    keeper_api_id: fallbackApiId,
    narrator_api_id: fallbackApiId,
    planner_api_id: fallbackApiId,
    replyer_api_id: fallbackApiId,
  } satisfies AgentApiIds
}

export function StageSessionSettingsPanel({
  apis,
  config,
  copy,
  currentPlayerProfileId,
  onRefreshSnapshot,
  onSavePlayerDescription,
  onSavePlayerProfile,
  onSaveSessionConfig,
  playerProfiles,
  runtimeSnapshot,
}: StageSessionSettingsPanelProps) {
  const [mode, setMode] = useState<SessionConfigMode>(config.mode)
  const [sessionApiIds, setSessionApiIds] = useState<AgentApiIds | null>(
    createSessionApiDefaults(config, apis),
  )
  const [selectedPlayerProfileId, setSelectedPlayerProfileId] = useState(
    currentPlayerProfileId ?? NO_PLAYER_PROFILE_OPTION_VALUE,
  )
  const [manualDescription, setManualDescription] = useState(runtimeSnapshot?.player_description ?? '')
  const [notice, setNotice] = useState<Notice | null>(null)
  const [isSavingConfig, setIsSavingConfig] = useState(false)
  const [isSavingPlayerProfile, setIsSavingPlayerProfile] = useState(false)
  const [isSavingDescription, setIsSavingDescription] = useState(false)
  const [isRefreshingSnapshot, setIsRefreshingSnapshot] = useState(false)

  useEffect(() => {
    setMode(config.mode)
    setSessionApiIds(createSessionApiDefaults(config, apis))
  }, [apis, config])

  useEffect(() => {
    setSelectedPlayerProfileId(currentPlayerProfileId ?? NO_PLAYER_PROFILE_OPTION_VALUE)
  }, [currentPlayerProfileId])

  useEffect(() => {
    setManualDescription(runtimeSnapshot?.player_description ?? '')
  }, [runtimeSnapshot?.player_description])

  const apiItems = useMemo(
    () =>
      apis.map((api) => ({
        label: `${api.api_id} · ${api.model}`,
        value: api.api_id,
      })),
    [apis],
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
        : playerProfiles.find((profile) => profile.player_profile_id === selectedPlayerProfileId) ?? null,
    [playerProfiles, selectedPlayerProfileId],
  )

  async function handleSaveConfig() {
    if (mode === 'use_session' && !sessionApiIds) {
      return
    }

    setNotice(null)
    setIsSavingConfig(true)

    try {
      await onSaveSessionConfig({
        mode,
        ...(mode === 'use_session' && sessionApiIds ? { session_api_ids: sessionApiIds } : {}),
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
            <div className="space-y-2">
              <p className="text-sm font-medium text-[var(--color-text-primary)]">
                {copy.settings.api.section}
              </p>
              <p className="text-sm leading-6 text-[var(--color-text-secondary)]">
                {copy.settings.api.description}
              </p>
            </div>

            <div className="grid gap-3 md:grid-cols-2">
              <button
                className={cn(
                  'rounded-[1.15rem] border px-4 py-3 text-left transition',
                  mode === 'use_global'
                    ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] text-[var(--color-text-primary)]'
                    : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]',
                )}
                onClick={() => {
                  setMode('use_global')
                }}
                type="button"
              >
                <p className="text-sm font-medium">{copy.settings.api.modeOptions.useGlobal}</p>
              </button>

              <button
                className={cn(
                  'rounded-[1.15rem] border px-4 py-3 text-left transition',
                  mode === 'use_session'
                    ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] text-[var(--color-text-primary)]'
                    : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]',
                )}
                onClick={() => {
                  setMode('use_session')
                  setSessionApiIds((current) => current ?? createSessionApiDefaults(config, apis))
                }}
                type="button"
              >
                <p className="text-sm font-medium">{copy.settings.api.modeOptions.useSession}</p>
              </button>
            </div>

            <div className="rounded-[1.25rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-4">
              <p className="text-xs text-[var(--color-text-muted)]">{copy.settings.api.effective}</p>
              <div className="mt-3 grid gap-3 md:grid-cols-2">
                {agentApiRoleKeys.map((roleKey) => (
                  <div key={roleKey}>
                    <p className="text-xs text-[var(--color-text-muted)]">{copy.settings.api.roles[roleKey]}</p>
                    <p className="mt-1 text-sm text-[var(--color-text-primary)]">
                      {config.effective_api_ids[roleKey] || '—'}
                    </p>
                  </div>
                ))}
              </div>
            </div>

            {mode === 'use_session' ? (
              apiItems.length === 0 || !sessionApiIds ? (
                <div className="rounded-[1.25rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_72%,transparent)] px-4 py-4 text-sm leading-7 text-[var(--color-text-secondary)]">
                  {copy.settings.api.empty}
                </div>
              ) : (
                <div className="grid gap-4 md:grid-cols-2">
                  {agentApiRoleKeys.map((roleKey) => (
                    <label className="space-y-2.5" key={roleKey}>
                      <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                        {copy.settings.api.roles[roleKey]}
                      </span>
                      <Select
                        items={apiItems}
                        placeholder={copy.settings.api.selectPlaceholder}
                        textAlign="start"
                        value={sessionApiIds[roleKey]}
                        onValueChange={(value) => {
                          setSessionApiIds((current) =>
                            current
                              ? {
                                  ...current,
                                  [roleKey]: value,
                                }
                              : current,
                          )
                        }}
                      />
                    </label>
                  ))}
                </div>
              )
            ) : (
              <div className="rounded-[1.25rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-4 text-sm leading-7 text-[var(--color-text-secondary)]">
                {copy.settings.api.usingGlobal}
              </div>
            )}

            <div className="flex justify-end">
              <Button disabled={isSavingConfig || (mode === 'use_session' && !sessionApiIds)} onClick={() => void handleSaveConfig()}>
                {isSavingConfig ? copy.settings.api.saving : copy.settings.api.save}
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
              <div className="rounded-[1.25rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-4">
                <p className="text-xs text-[var(--color-text-muted)]">{copy.settings.playerProfile.preview}</p>
                <p className="mt-2 text-sm leading-7 text-[var(--color-text-primary)]">
                  {selectedPlayerProfile.description}
                </p>
              </div>
            ) : (
              <div className="rounded-[1.25rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-4 text-sm leading-7 text-[var(--color-text-secondary)]">
                {copy.settings.playerProfile.noProfileHint}
              </div>
            )}

            <div className="flex justify-end">
              <Button disabled={isSavingPlayerProfile} onClick={() => void handleSavePlayerProfile()} variant="secondary">
                {isSavingPlayerProfile ? copy.settings.playerProfile.saving : copy.settings.playerProfile.save}
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
              onChange={(event) => {
                setManualDescription(event.target.value)
              }}
              placeholder={copy.settings.playerDescription.placeholder}
              value={manualDescription}
            />

            <div className="flex justify-end">
              <Button disabled={isSavingDescription} onClick={() => void handleSavePlayerDescription()} variant="secondary">
                {isSavingDescription ? copy.settings.playerDescription.saving : copy.settings.playerDescription.save}
              </Button>
            </div>
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
                icon={<FontAwesomeIcon className={cn(isRefreshingSnapshot ? 'animate-spin' : '')} icon={faRotateRight} />}
                label={copy.settings.snapshot.refresh}
                onClick={() => void handleRefreshSnapshot()}
                size="sm"
                variant="ghost"
              />
            </div>

            <div className="grid gap-3 md:grid-cols-2">
              <div className="rounded-[1.25rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-4">
                <p className="text-xs text-[var(--color-text-muted)]">{copy.settings.snapshot.turnIndex}</p>
                <p className="mt-2 text-sm text-[var(--color-text-primary)]">
                  {runtimeSnapshot?.turn_index ?? '—'}
                </p>
              </div>
              <div className="rounded-[1.25rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-4">
                <p className="text-xs text-[var(--color-text-muted)]">{copy.settings.snapshot.currentNode}</p>
                <p className="mt-2 font-mono text-sm text-[var(--color-text-primary)]">
                  {runtimeSnapshot?.world_state.current_node ?? '—'}
                </p>
              </div>
              <div className="rounded-[1.25rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-4">
                <p className="text-xs text-[var(--color-text-muted)]">{copy.settings.snapshot.activeCharacters}</p>
                <p className="mt-2 text-sm text-[var(--color-text-primary)]">
                  {runtimeSnapshot?.world_state.active_characters.length ?? 0}
                </p>
              </div>
              <div className="rounded-[1.25rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-4">
                <p className="text-xs text-[var(--color-text-muted)]">{copy.settings.snapshot.playerStateKeys}</p>
                <p className="mt-2 text-sm text-[var(--color-text-primary)]">
                  {runtimeSnapshot ? Object.keys(runtimeSnapshot.world_state.player_state).length : 0}
                </p>
              </div>
            </div>

            <div className="rounded-[1.25rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-4 py-4">
              <p className="text-xs text-[var(--color-text-muted)]">{copy.settings.snapshot.playerDescription}</p>
              <p className="mt-2 text-sm leading-7 text-[var(--color-text-primary)]">
                {runtimeSnapshot?.player_description || '—'}
              </p>
            </div>
          </section>

          {notice ? (
            <div
              className={cn(
                'rounded-[1.25rem] border px-4 py-3 text-sm text-[var(--color-text-primary)]',
                notice.tone === 'success'
                  ? 'border-[var(--color-state-success-line)] bg-[var(--color-state-success-soft)]'
                  : 'border-[var(--color-state-error-line)] bg-[var(--color-state-error-soft)]',
              )}
            >
              {notice.message}
            </div>
          ) : null}
        </div>
      </div>
    </div>
  )
}
