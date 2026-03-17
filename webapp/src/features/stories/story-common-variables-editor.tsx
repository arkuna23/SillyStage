import { faPlus } from '@fortawesome/free-solid-svg-icons/faPlus'
import { faTrashCan } from '@fortawesome/free-solid-svg-icons/faTrashCan'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { useId, useMemo } from 'react'
import { useTranslation } from 'react-i18next'

import { Button } from '../../components/ui/button'
import { IconButton } from '../../components/ui/icon-button'
import { Input } from '../../components/ui/input'
import { Select, type SelectOption } from '../../components/ui/select'
import { Switch } from '../../components/ui/switch'
import { cn } from '../../lib/cn'
import type { CharacterSummary } from '../characters/types'
import type {
  StoryCommonVariableKeySource,
  StoryCommonVariableSchemaCatalog,
} from './story-common-variable-schema-catalog'
import type { CommonVariableScope } from './types'
import {
  createStoryCommonVariableDraft,
  type StoryCommonVariableDraft,
  type StoryCommonVariableDraftErrors,
} from './story-common-variable-drafts'

type StoryCommonVariablesEditorProps = {
  characters: ReadonlyArray<CharacterSummary>
  disabled?: boolean
  drafts: ReadonlyArray<StoryCommonVariableDraft>
  errors?: StoryCommonVariableDraftErrors
  onChange: (nextDrafts: StoryCommonVariableDraft[]) => void
  resourceCharacterIds: ReadonlyArray<string>
  schemaCatalog: StoryCommonVariableSchemaCatalog
}

type StoryCommonVariableValidationCode =
  | 'characterInvalid'
  | 'characterRequired'
  | 'displayNameRequired'
  | 'keyRequired'

export function StoryCommonVariablesEditor({
  characters,
  disabled = false,
  drafts,
  errors,
  onChange,
  resourceCharacterIds,
  schemaCatalog,
}: StoryCommonVariablesEditorProps) {
  const { t } = useTranslation()
  const fieldIdPrefix = useId()

  const characterLookup = useMemo(
    () => new Map(characters.map((character) => [character.character_id, character])),
    [characters],
  )
  const characterOptions = useMemo(() => {
    const resourceCharacterIdSet = new Set(resourceCharacterIds)
    const knownCharacterIds = new Set(resourceCharacterIds)

    drafts.forEach((draft) => {
      const characterId = draft.character_id.trim()

      if (characterId.length > 0) {
        knownCharacterIds.add(characterId)
      }
    })

    return Array.from(knownCharacterIds).map((characterId) => {
      const character = characterLookup.get(characterId)
      const labelParts = [character?.name ?? characterId]

      if (character?.name) {
        labelParts.push(characterId)
      }

      if (!resourceCharacterIdSet.has(characterId)) {
        labelParts.push(t('stories.commonVariables.unavailableCharacter'))
      }

      return {
        label: labelParts.join(' · '),
        value: characterId,
      }
    })
  }, [characterLookup, drafts, resourceCharacterIds, t])

  const resourceCharacterBadges = useMemo(
    () =>
      resourceCharacterIds.map((characterId) => ({
        id: characterId,
        label: characterLookup.get(characterId)?.name ?? characterId,
      })),
    [characterLookup, resourceCharacterIds],
  )

  function updateDraft(
    draftId: string,
    updater: (draft: StoryCommonVariableDraft) => StoryCommonVariableDraft,
  ) {
    onChange(drafts.map((draft) => (draft.id === draftId ? updater(draft) : draft)))
  }

  function removeDraft(draftId: string) {
    onChange(drafts.filter((draft) => draft.id !== draftId))
  }

  function addDraft() {
    onChange([...drafts, createStoryCommonVariableDraft()])
  }

  function getKeySource(draft: StoryCommonVariableDraft): StoryCommonVariableKeySource {
    if (draft.scope === 'character') {
      return schemaCatalog.characterByCharacterId[draft.character_id.trim()] ?? {
        items: [],
        status: 'missing',
      }
    }

    return draft.scope === 'player' ? schemaCatalog.player : schemaCatalog.world
  }

  function buildKeyOptions(baseItems: ReadonlyArray<SelectOption>, currentKey: string) {
    const normalizedKey = currentKey.trim()

    if (normalizedKey.length === 0 || baseItems.some((item) => item.value === normalizedKey)) {
      return [...baseItems]
    }

    return [
      {
        label: t('stories.commonVariables.legacyKey', { key: normalizedKey }),
        value: normalizedKey,
      },
      ...baseItems,
    ]
  }

  function getKeyPlaceholder(draft: StoryCommonVariableDraft, keySource: StoryCommonVariableKeySource) {
    if (draft.scope === 'character' && draft.character_id.trim().length === 0) {
      return t('stories.commonVariables.placeholders.keyCharacterFirst')
    }

    if (keySource.status === 'loading') {
      return t('stories.commonVariables.placeholders.keyLoading')
    }

    if (keySource.status === 'error') {
      return t('stories.commonVariables.placeholders.keyLoadFailed')
    }

    if (keySource.status === 'missing') {
      if (draft.scope === 'character') {
        return t('stories.commonVariables.placeholders.keySchemaMissingCharacter')
      }

      return draft.scope === 'player'
        ? t('stories.commonVariables.placeholders.keySchemaMissingPlayer')
        : t('stories.commonVariables.placeholders.keySchemaMissingWorld')
    }

    if (keySource.items.length === 0) {
      return t('stories.commonVariables.placeholders.keyEmpty')
    }

    return t('stories.commonVariables.placeholders.key')
  }

  function getValidationMessage(
    errorCode: StoryCommonVariableValidationCode | undefined,
    index: number,
  ) {
    if (!errorCode) {
      return null
    }

    if (errorCode === 'keyRequired') {
      return t('stories.commonVariables.errors.keyRequired', { index: index + 1 })
    }

    if (errorCode === 'displayNameRequired') {
      return t('stories.commonVariables.errors.displayNameRequired', { index: index + 1 })
    }

    if (errorCode === 'characterRequired') {
      return t('stories.commonVariables.errors.characterRequired', { index: index + 1 })
    }

    return t('stories.commonVariables.errors.characterInvalid', { index: index + 1 })
  }

  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
          {t('stories.commonVariables.description')}
        </p>
        {resourceCharacterBadges.length > 0 ? (
          <div className="flex flex-wrap items-center gap-2">
            <span className="text-xs text-[var(--color-text-muted)]">
              {t('stories.commonVariables.availableCharacters')}
            </span>
            {resourceCharacterBadges.map((character) => (
              <span
                className="inline-flex items-center rounded-full border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-3 py-1 text-xs text-[var(--color-text-secondary)]"
                key={character.id}
              >
                {character.label}
              </span>
            ))}
          </div>
        ) : null}
      </div>

      {drafts.length === 0 ? (
        <div className="rounded-[1.45rem] border border-dashed border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-5 text-sm leading-7 text-[var(--color-text-secondary)]">
          {t('stories.commonVariables.empty')}
        </div>
      ) : (
        <div className="space-y-3">
          {drafts.map((draft, index) => {
            const keyErrorMessage = getValidationMessage(errors?.[draft.id]?.key, index)
            const displayNameErrorMessage = getValidationMessage(
              errors?.[draft.id]?.display_name,
              index,
            )
            const characterErrorMessage = getValidationMessage(
              errors?.[draft.id]?.character_id,
              index,
            )
            const keySource = getKeySource(draft)
            const keyOptions = buildKeyOptions(keySource.items, draft.key)
            const keyPlaceholder = getKeyPlaceholder(draft, keySource)
            const keySelectDisabled = disabled || keyOptions.length === 0

            return (
              <div
                className="space-y-4 rounded-[1.5rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4"
                key={draft.id}
              >
                <div className="flex items-center justify-between gap-4">
                  <div className="min-w-0">
                    <p className="text-sm font-medium text-[var(--color-text-primary)]">
                      {t('stories.commonVariables.itemTitle', { index: index + 1 })}
                    </p>
                    <p className="text-xs leading-6 text-[var(--color-text-muted)]">
                      {t('stories.commonVariables.itemSubtitle')}
                    </p>
                  </div>
                  <IconButton
                    disabled={disabled}
                    icon={<FontAwesomeIcon icon={faTrashCan} />}
                    label={t('stories.commonVariables.remove')}
                    onClick={() => {
                      removeDraft(draft.id)
                    }}
                    size="sm"
                    variant="ghost"
                  />
                </div>

                <div className="grid gap-4 md:grid-cols-2">
                  <div className="space-y-2">
                    <label
                      className="block text-sm font-medium text-[var(--color-text-primary)]"
                      htmlFor={`${fieldIdPrefix}-${draft.id}-scope`}
                    >
                      {t('stories.commonVariables.fields.scope')}
                    </label>
                    <Select
                      items={[
                        {
                          label: t('stories.commonVariables.scopes.world'),
                          value: 'world',
                        },
                        {
                          label: t('stories.commonVariables.scopes.player'),
                          value: 'player',
                        },
                        {
                          label: t('stories.commonVariables.scopes.character'),
                          value: 'character',
                        },
                      ]}
                      triggerId={`${fieldIdPrefix}-${draft.id}-scope`}
                      value={draft.scope}
                      onValueChange={(scope) => {
                        updateDraft(draft.id, (currentDraft) => ({
                          ...currentDraft,
                          key: scope === currentDraft.scope ? currentDraft.key : '',
                          character_id: scope === 'character' ? currentDraft.character_id : '',
                          scope: scope as CommonVariableScope,
                        }))
                      }}
                    />
                  </div>

                  <div className="space-y-2">
                    <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                      {t('stories.commonVariables.fields.pinned')}
                    </span>
                    <div className="flex min-h-12 items-start justify-between gap-4 px-1 py-1">
                      <p className="min-w-0 text-sm leading-6 text-[var(--color-text-muted)]">
                        {t('stories.commonVariables.pinnedDescription')}
                      </p>
                      <Switch
                        checked={draft.pinned}
                        disabled={disabled}
                        onCheckedChange={(checked) => {
                          updateDraft(draft.id, (currentDraft) => ({
                            ...currentDraft,
                            pinned: checked,
                          }))
                        }}
                        size="sm"
                      />
                    </div>
                  </div>
                </div>

                <div className="grid gap-4 md:grid-cols-2">
                  <div className="space-y-2">
                    <label
                      className="block text-sm font-medium text-[var(--color-text-primary)]"
                      htmlFor={`${fieldIdPrefix}-${draft.id}-key`}
                    >
                      {t('stories.commonVariables.fields.key')}
                    </label>
                    <Select
                      disabled={keySelectDisabled}
                      items={keyOptions}
                      placeholder={keyPlaceholder}
                      textAlign="start"
                      triggerClassName={cn(
                        keyErrorMessage
                          ? 'border-[var(--color-state-error-line)] focus:border-[var(--color-state-error-line)] focus:ring-[color-mix(in_srgb,var(--color-state-error)_24%,transparent)]'
                          : '',
                      )}
                      triggerId={`${fieldIdPrefix}-${draft.id}-key`}
                      value={draft.key || undefined}
                      onValueChange={(key) => {
                        updateDraft(draft.id, (currentDraft) => ({
                          ...currentDraft,
                          key,
                        }))
                      }}
                    />
                    {keyErrorMessage ? (
                      <p className="text-xs leading-6 text-[var(--color-state-error)]">
                        {keyErrorMessage}
                      </p>
                    ) : null}
                  </div>

                  <div className="space-y-2">
                    <label
                      className="block text-sm font-medium text-[var(--color-text-primary)]"
                      htmlFor={`${fieldIdPrefix}-${draft.id}-display-name`}
                    >
                      {t('stories.commonVariables.fields.displayName')}
                    </label>
                    <Input
                      className={cn(
                        displayNameErrorMessage
                          ? 'border-[var(--color-state-error-line)] focus:border-[var(--color-state-error-line)] focus:ring-[color-mix(in_srgb,var(--color-state-error)_24%,transparent)]'
                          : '',
                      )}
                      disabled={disabled}
                      id={`${fieldIdPrefix}-${draft.id}-display-name`}
                      name={`${fieldIdPrefix}-${draft.id}-display-name`}
                      placeholder={t('stories.commonVariables.placeholders.displayName')}
                      value={draft.display_name}
                      onChange={(event) => {
                        updateDraft(draft.id, (currentDraft) => ({
                          ...currentDraft,
                          display_name: event.target.value,
                        }))
                      }}
                    />
                    {displayNameErrorMessage ? (
                      <p className="text-xs leading-6 text-[var(--color-state-error)]">
                        {displayNameErrorMessage}
                      </p>
                    ) : null}
                  </div>
                </div>

                {draft.scope === 'character' ? (
                  <div className="space-y-2">
                    <label
                      className="block text-sm font-medium text-[var(--color-text-primary)]"
                      htmlFor={`${fieldIdPrefix}-${draft.id}-character`}
                    >
                      {t('stories.commonVariables.fields.characterId')}
                    </label>
                    <Select
                      allowClear
                      clearLabel={t('stories.commonVariables.placeholders.characterId')}
                      items={characterOptions}
                      placeholder={t('stories.commonVariables.placeholders.characterId')}
                      textAlign="start"
                      triggerClassName={cn(
                        characterErrorMessage
                          ? 'border-[var(--color-state-error-line)] focus:border-[var(--color-state-error-line)] focus:ring-[color-mix(in_srgb,var(--color-state-error)_24%,transparent)]'
                          : '',
                      )}
                      triggerId={`${fieldIdPrefix}-${draft.id}-character`}
                      value={draft.character_id || undefined}
                      onValueChange={(characterId) => {
                        updateDraft(draft.id, (currentDraft) => ({
                          ...currentDraft,
                          character_id: characterId,
                          key:
                            characterId === currentDraft.character_id ? currentDraft.key : '',
                        }))
                      }}
                    />
                    {characterErrorMessage ? (
                      <p className="text-xs leading-6 text-[var(--color-state-error)]">
                        {characterErrorMessage}
                      </p>
                    ) : null}
                  </div>
                ) : null}
              </div>
            )
          })}
        </div>
      )}

      <div className="flex justify-end">
        <Button
          disabled={disabled}
          onClick={() => {
            addDraft()
          }}
          variant="secondary"
        >
          <FontAwesomeIcon icon={faPlus} />
          {t('stories.commonVariables.add')}
        </Button>
      </div>
    </div>
  )
}
