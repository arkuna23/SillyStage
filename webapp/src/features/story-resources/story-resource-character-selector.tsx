import { useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { Badge } from '../../components/ui/badge'
import { Input } from '../../components/ui/input'
import { cn } from '../../lib/cn'
import { normalizeCharacterFolderRegistryName } from '../characters/folder-registry'
import type { CharacterSummary } from '../characters/types'

const ALL_FOLDER_VALUE = '__all__'
const UNFILED_FOLDER_VALUE = '__unfiled__'

function normalizeCharacterFolder(folder: string) {
  return normalizeCharacterFolderRegistryName(folder)
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

type StoryResourceCharacterSelectorProps = {
  characters: ReadonlyArray<CharacterSummary>
  disabled?: boolean
  loading?: boolean
  selectedCharacterIds: ReadonlyArray<string>
  onChangeSelectedCharacterIds: (characterIds: string[]) => void
}

export function StoryResourceCharacterSelector({
  characters,
  disabled = false,
  loading = false,
  selectedCharacterIds,
  onChangeSelectedCharacterIds,
}: StoryResourceCharacterSelectorProps) {
  const { t } = useTranslation()
  const [activeFolder, setActiveFolder] = useState<string>(ALL_FOLDER_VALUE)
  const [folderListFilter, setFolderListFilter] = useState('')
  const [searchQuery, setSearchQuery] = useState('')

  const selectedCharacterSet = useMemo(() => new Set(selectedCharacterIds), [selectedCharacterIds])
  const characterLookup = useMemo(
    () => new Map(characters.map((character) => [character.character_id, character])),
    [characters],
  )
  const selectedCharacters = useMemo(
    () => selectedCharacterIds.map((characterId) => characterLookup.get(characterId) ?? null),
    [characterLookup, selectedCharacterIds],
  )
  const folderOptions = useMemo(
    () =>
      Array.from(
        new Set(
          characters
            .map((character) => normalizeCharacterFolder(character.folder))
            .filter((folder) => folder.length > 0),
        ),
      ).sort((left, right) => left.localeCompare(right)),
    [characters],
  )

  const normalizedSearchQuery = searchQuery.trim().toLocaleLowerCase()
  const normalizedFolderListFilter = folderListFilter.trim().toLocaleLowerCase()
  const unfiledLabel = t('storyResources.filters.unfiled')
  const hasUnfiledCharacters = characters.some(
    (character) => normalizeCharacterFolder(character.folder).length === 0,
  )

  const filteredFolderOptions = useMemo(() => {
    const nextFolders = folderOptions.filter((folder) =>
      normalizedFolderListFilter.length === 0
        ? true
        : folder.toLocaleLowerCase().includes(normalizedFolderListFilter),
    )

    if (
      activeFolder !== ALL_FOLDER_VALUE &&
      activeFolder !== UNFILED_FOLDER_VALUE &&
      activeFolder.length > 0 &&
      !nextFolders.includes(activeFolder)
    ) {
      return [activeFolder, ...nextFolders]
    }

    return nextFolders
  }, [activeFolder, folderOptions, normalizedFolderListFilter])

  const showUnfiledOption =
    hasUnfiledCharacters &&
    (normalizedFolderListFilter.length === 0 ||
      unfiledLabel.toLocaleLowerCase().includes(normalizedFolderListFilter) ||
      activeFolder === UNFILED_FOLDER_VALUE)

  const filteredCharacters = useMemo(
    () =>
      characters.filter((character) => {
        const folder = normalizeCharacterFolder(character.folder)
        const matchesFolder =
          activeFolder === ALL_FOLDER_VALUE ||
          (activeFolder === UNFILED_FOLDER_VALUE ? folder.length === 0 : folder === activeFolder)
        const matchesSearch =
          normalizedSearchQuery.length === 0 ||
          buildCharacterSearchText(character).includes(normalizedSearchQuery)

        return matchesFolder && matchesSearch
      }),
    [activeFolder, characters, normalizedSearchQuery],
  )

  function toggleCharacter(characterId: string) {
    const isSelected = selectedCharacterSet.has(characterId)

    onChangeSelectedCharacterIds(
      isSelected
        ? selectedCharacterIds.filter((id) => id !== characterId)
        : [...selectedCharacterIds, characterId],
    )
  }

  if (loading) {
    return (
      <div className="grid gap-4 xl:grid-cols-[14rem_minmax(0,1fr)]">
        <div className="space-y-3 rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] p-4">
          <div className="h-10 animate-pulse rounded-2xl bg-[var(--color-bg-panel)]" />
          <div className="space-y-2">
            {Array.from({ length: 5 }).map((_, index) => (
              <div
                className="h-10 animate-pulse rounded-[1rem] bg-[var(--color-bg-panel)]"
                key={index}
              />
            ))}
          </div>
        </div>
        <div className="space-y-4 rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] p-4">
          <div className="h-12 animate-pulse rounded-2xl bg-[var(--color-bg-panel)]" />
          <div className="grid gap-2 sm:grid-cols-2">
            {Array.from({ length: 6 }).map((_, index) => (
              <div
                className="h-24 animate-pulse rounded-[1.2rem] bg-[var(--color-bg-panel)]"
                key={index}
              />
            ))}
          </div>
        </div>
      </div>
    )
  }

  if (characters.length === 0) {
    return (
      <div className="rounded-[1.45rem] border border-dashed border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-5 text-sm text-[var(--color-text-secondary)]">
        {t('storyResources.form.emptyCharacters')}
      </div>
    )
  }

  return (
    <div className="space-y-4">
      <div className="space-y-3 rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_80%,transparent)] px-4 py-4">
        <div className="space-y-1">
          <h4 className="text-sm font-medium text-[var(--color-text-primary)]">
            {t('storyResources.filters.selectedTitle')}
          </h4>
          <p className="text-xs leading-6 text-[var(--color-text-muted)]">
            {t('storyResources.form.emptySelection')}
          </p>
        </div>
        <div className="flex flex-wrap gap-2">
          {selectedCharacters.length > 0 ? (
            selectedCharacters.map((character, index) => (
              <Badge
                className="normal-case px-3 py-1.5"
                key={character?.character_id ?? `${selectedCharacterIds[index]}-${index}`}
                variant="subtle"
              >
                {character?.name ?? selectedCharacterIds[index]}
              </Badge>
            ))
          ) : (
            <span className="text-sm text-[var(--color-text-muted)]">
              {t('storyResources.form.emptySelection')}
            </span>
          )}
        </div>
      </div>

      <div className="grid gap-4 xl:grid-cols-[14rem_minmax(0,1fr)]">
        <div className="space-y-3 rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] p-4">
          <div className="space-y-1">
            <h4 className="text-sm font-medium text-[var(--color-text-primary)]">
              {t('storyResources.filters.folderTitle')}
            </h4>
            <p className="text-xs leading-6 text-[var(--color-text-muted)]">
              {t('storyResources.filters.folderDescription')}
            </p>
          </div>

          <Input
            disabled={disabled}
            onChange={(event) => {
              setFolderListFilter(event.target.value)
            }}
            placeholder={t('storyResources.filters.folderSearchPlaceholder')}
            value={folderListFilter}
          />

          <div className="max-h-[22rem] space-y-2 overflow-y-auto pr-1">
            <button
              className={cn(
                'w-full rounded-[1rem] border px-3 py-2.5 text-left text-sm transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-focus-ring)] disabled:pointer-events-none disabled:opacity-45',
                activeFolder === ALL_FOLDER_VALUE
                  ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] text-[var(--color-text-primary)]'
                  : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] text-[var(--color-text-secondary)] hover:border-[var(--color-accent-copper-soft)] hover:text-[var(--color-text-primary)]',
              )}
              disabled={disabled}
              onClick={() => {
                setActiveFolder(ALL_FOLDER_VALUE)
              }}
              type="button"
            >
              {t('storyResources.filters.all')}
            </button>

            {showUnfiledOption ? (
              <button
                className={cn(
                  'w-full rounded-[1rem] border px-3 py-2.5 text-left text-sm transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-focus-ring)] disabled:pointer-events-none disabled:opacity-45',
                  activeFolder === UNFILED_FOLDER_VALUE
                    ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] text-[var(--color-text-primary)]'
                    : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] text-[var(--color-text-secondary)] hover:border-[var(--color-accent-copper-soft)] hover:text-[var(--color-text-primary)]',
                )}
                disabled={disabled}
                onClick={() => {
                  setActiveFolder(UNFILED_FOLDER_VALUE)
                }}
                type="button"
              >
                {unfiledLabel}
              </button>
            ) : null}

            {filteredFolderOptions.length > 0 ? (
              filteredFolderOptions.map((folder) => (
                <button
                  className={cn(
                    'w-full rounded-[1rem] border px-3 py-2.5 text-left text-sm transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-focus-ring)] disabled:pointer-events-none disabled:opacity-45',
                    activeFolder === folder
                      ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] text-[var(--color-text-primary)]'
                      : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] text-[var(--color-text-secondary)] hover:border-[var(--color-accent-copper-soft)] hover:text-[var(--color-text-primary)]',
                  )}
                  disabled={disabled}
                  key={folder}
                  onClick={() => {
                    setActiveFolder(folder)
                  }}
                  type="button"
                >
                  {folder}
                </button>
              ))
            ) : (
              <div className="rounded-[1rem] border border-dashed border-[var(--color-border-subtle)] px-3 py-3 text-xs leading-6 text-[var(--color-text-muted)]">
                {t('storyResources.filters.folderEmpty')}
              </div>
            )}
          </div>
        </div>

        <div className="space-y-3 rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] p-4">
          <div className="space-y-1">
            <h4 className="text-sm font-medium text-[var(--color-text-primary)]">
              {t('storyResources.filters.charactersTitle')}
            </h4>
            <p className="text-xs leading-6 text-[var(--color-text-muted)]">
              {t('storyResources.filters.searchDescription')}
            </p>
          </div>

          <Input
            disabled={disabled}
            onChange={(event) => {
              setSearchQuery(event.target.value)
            }}
            placeholder={t('storyResources.filters.searchPlaceholder')}
            value={searchQuery}
          />

          {filteredCharacters.length > 0 ? (
            <div className="grid gap-2 sm:grid-cols-2">
              {filteredCharacters.map((character) => {
                const isSelected = selectedCharacterSet.has(character.character_id)
                const folderLabel = normalizeCharacterFolder(character.folder) || unfiledLabel

                return (
                  <button
                    className={cn(
                      'rounded-[1.2rem] border px-3 py-3 text-left transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-focus-ring)] disabled:pointer-events-none disabled:opacity-45',
                      isSelected
                        ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] text-[var(--color-text-primary)]'
                        : 'border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_84%,transparent)] text-[var(--color-text-secondary)] hover:border-[var(--color-accent-copper-soft)] hover:text-[var(--color-text-primary)]',
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
                    <div className="mt-2 flex flex-wrap gap-1.5 text-[0.72rem]">
                      <span className="rounded-full border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-2 py-1 text-[var(--color-text-muted)]">
                        {folderLabel}
                      </span>
                      {character.tags.slice(0, 2).map((tag) => (
                        <span
                          className="rounded-full border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-2 py-1 text-[var(--color-text-muted)]"
                          key={tag}
                        >
                          #{tag}
                        </span>
                      ))}
                    </div>
                  </button>
                )
              })}
            </div>
          ) : (
            <div className="rounded-[1.2rem] border border-dashed border-[var(--color-border-subtle)] px-4 py-5 text-sm text-[var(--color-text-secondary)]">
              {t('storyResources.filters.charactersEmpty')}
            </div>
          )}
        </div>
      </div>
    </div>
  )
}
