import { faXmark } from '@fortawesome/free-solid-svg-icons/faXmark'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { Input } from '../../components/ui/input'
import { Select, type SelectOption } from '../../components/ui/select'
import { cn } from '../../lib/cn'
import type { CharacterSummary } from '../characters/types'

type StoryGraphCharacterSelectProps = {
  allowClear?: boolean
  characters: ReadonlyArray<CharacterSummary>
  disabled?: boolean
  onValueChange: (value: string) => void
  placeholder?: string
  value?: string | null
}

type StoryGraphCharacterMultiSelectProps = {
  characters: ReadonlyArray<CharacterSummary>
  disabled?: boolean
  onChange: (characterIds: string[]) => void
  selectedCharacterIds: ReadonlyArray<string>
}

function buildCharacterSearchText(character: CharacterSummary) {
  return [
    character.character_id,
    character.name,
    character.personality,
    character.style,
    ...character.tags,
  ]
    .join(' ')
    .toLocaleLowerCase()
}

function buildCharacterOption(character: CharacterSummary): SelectOption {
  return {
    label: `${character.name} · ${character.character_id}`,
    value: character.character_id,
  }
}

function buildCharacterFallbackOption(characterId: string, missingLabel: string): SelectOption {
  return {
    label: `${characterId} · ${missingLabel}`,
    value: characterId,
  }
}

function ensureCurrentCharacterOption(
  items: ReadonlyArray<SelectOption>,
  value: string,
  missingLabel: string,
) {
  if (!value.trim() || items.some((item) => item.value === value)) {
    return [...items]
  }

  return [buildCharacterFallbackOption(value, missingLabel), ...items]
}

export function StoryGraphCharacterSelect({
  allowClear = true,
  characters,
  disabled = false,
  onValueChange,
  placeholder,
  value,
}: StoryGraphCharacterSelectProps) {
  const { t } = useTranslation()

  const items = useMemo(
    () =>
      ensureCurrentCharacterOption(
        characters.map(buildCharacterOption),
        value?.trim() ?? '',
        t('stories.graph.missingCharacter'),
      ),
    [characters, t, value],
  )

  return (
    <Select
      allowClear={allowClear}
      clearLabel={t('stories.graph.clearSelection')}
      disabled={disabled}
      items={items}
      onValueChange={onValueChange}
      placeholder={placeholder ?? t('stories.graph.characterSelectPlaceholder')}
      textAlign="start"
      value={value?.trim() ? value : undefined}
    />
  )
}

export function StoryGraphCharacterMultiSelect({
  characters,
  disabled = false,
  onChange,
  selectedCharacterIds,
}: StoryGraphCharacterMultiSelectProps) {
  const { t } = useTranslation()
  const [searchQuery, setSearchQuery] = useState('')

  const characterMap = useMemo(
    () => new Map(characters.map((character) => [character.character_id, character])),
    [characters],
  )
  const selectedCharacterSet = useMemo(() => new Set(selectedCharacterIds), [selectedCharacterIds])

  const selectedEntries = useMemo(
    () =>
      selectedCharacterIds.map((characterId) => ({
        character: characterMap.get(characterId) ?? null,
        characterId,
      })),
    [characterMap, selectedCharacterIds],
  )

  const normalizedSearchQuery = searchQuery.trim().toLocaleLowerCase()
  const filteredCharacters = useMemo(
    () =>
      characters.filter((character) =>
        normalizedSearchQuery.length === 0
          ? true
          : buildCharacterSearchText(character).includes(normalizedSearchQuery),
      ),
    [characters, normalizedSearchQuery],
  )

  function removeCharacter(characterId: string) {
    onChange(selectedCharacterIds.filter((value) => value !== characterId))
  }

  function toggleCharacter(characterId: string) {
    onChange(
      selectedCharacterSet.has(characterId)
        ? selectedCharacterIds.filter((value) => value !== characterId)
        : [...selectedCharacterIds, characterId],
    )
  }

  return (
    <div className="space-y-3">
      <div className="space-y-2 rounded-[1.2rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_82%,transparent)] px-4 py-3.5">
        <p className="text-xs text-[var(--color-text-muted)]">
          {t('stories.graph.selectedCharacters')}
        </p>
        <div className="flex flex-wrap gap-2">
          {selectedEntries.length > 0 ? (
            selectedEntries.map(({ character, characterId }) => (
              <button
                className={cn(
                  'inline-flex min-h-8 items-center gap-2 rounded-full border px-3 py-1 text-xs font-medium transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-focus-ring)] disabled:pointer-events-none disabled:opacity-60',
                  character
                    ? 'border-[var(--color-border-subtle)] bg-white/6 text-[var(--color-text-primary)] hover:border-[var(--color-accent-copper-soft)]'
                    : 'border-[var(--color-state-info-line)] bg-[var(--color-state-info-soft)] text-[var(--color-text-primary)] hover:border-[var(--color-state-info-line)]',
                )}
                disabled={disabled}
                key={characterId}
                onClick={() => {
                  removeCharacter(characterId)
                }}
                title={t('stories.graph.removeCharacter')}
                type="button"
              >
                <span className="max-w-[12rem] truncate">
                  {character
                    ? `${character.name} · ${characterId}`
                    : `${characterId} · ${t('stories.graph.missingCharacter')}`}
                </span>
                <FontAwesomeIcon className="text-[0.7rem]" icon={faXmark} />
              </button>
            ))
          ) : (
            <span className="text-sm text-[var(--color-text-muted)]">
              {t('stories.graph.emptyCharactersSelection')}
            </span>
          )}
        </div>
      </div>

      <Input
        disabled={disabled}
        onChange={(event) => {
          setSearchQuery(event.target.value)
        }}
        placeholder={t('stories.graph.searchCharactersPlaceholder')}
        value={searchQuery}
      />

      <div className="max-h-[15rem] overflow-y-auto pr-1">
        {characters.length === 0 ? (
          <div className="rounded-[1.2rem] border border-dashed border-[var(--color-border-subtle)] px-4 py-4 text-sm text-[var(--color-text-secondary)]">
            {t('stories.graph.noCharactersAvailable')}
          </div>
        ) : filteredCharacters.length === 0 ? (
          <div className="rounded-[1.2rem] border border-dashed border-[var(--color-border-subtle)] px-4 py-4 text-sm text-[var(--color-text-secondary)]">
            {t('stories.graph.noMatchingCharacters')}
          </div>
        ) : (
          <div className="grid gap-2 sm:grid-cols-2">
            {filteredCharacters.map((character) => {
              const isSelected = selectedCharacterSet.has(character.character_id)

              return (
                <button
                  className={cn(
                    'rounded-[1.1rem] border px-3 py-3 text-left transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-focus-ring)] disabled:pointer-events-none disabled:opacity-45',
                    isSelected
                      ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] text-[var(--color-text-primary)]'
                      : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] text-[var(--color-text-secondary)] hover:border-[var(--color-accent-copper-soft)] hover:text-[var(--color-text-primary)]',
                  )}
                  disabled={disabled}
                  key={character.character_id}
                  onClick={() => {
                    toggleCharacter(character.character_id)
                  }}
                  type="button"
                >
                  <div className="truncate text-sm font-medium">{character.name}</div>
                  <div className="truncate pt-1 font-mono text-[0.74rem] text-[var(--color-text-muted)]">
                    {character.character_id}
                  </div>
                </button>
              )
            })}
          </div>
        )}
      </div>
    </div>
  )
}
