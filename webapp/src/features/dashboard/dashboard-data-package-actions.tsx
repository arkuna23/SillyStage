import { faDownload } from '@fortawesome/free-solid-svg-icons/faDownload'
import { faRotateRight } from '@fortawesome/free-solid-svg-icons/faRotateRight'
import { faUpload } from '@fortawesome/free-solid-svg-icons/faUpload'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import type { TFunction } from 'i18next'
import type { ChangeEvent } from 'react'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { Badge } from '../../components/ui/badge'
import { Button } from '../../components/ui/button'
import {
  Dialog,
  DialogBody,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../../components/ui/dialog'
import { IconButton } from '../../components/ui/icon-button'
import { Input } from '../../components/ui/input'
import { SegmentedSelector } from '../../components/ui/segmented-selector'
import { Switch } from '../../components/ui/switch'
import { useToast } from '../../components/ui/toast-context'
import { cn } from '../../lib/cn'
import { listPresets } from '../apis/api'
import type { Preset } from '../apis/types'
import { listCharacters } from '../characters/api'
import { normalizeCharacterFolderRegistryName } from '../characters/folder-registry'
import type { CharacterSummary } from '../characters/types'
import { listLorebooks } from '../lorebooks/api'
import type { Lorebook } from '../lorebooks/types'
import { listPlayerProfiles } from '../player-profiles/api'
import type { PlayerProfile } from '../player-profiles/types'
import { listSchemas } from '../schemas/api'
import type { SchemaResource } from '../schemas/types'
import { listStories } from '../stories/api'
import type { StorySummary } from '../stories/types'
import { listStoryResources } from '../story-resources/api'
import type { StoryResource } from '../story-resources/types'
import {
  commitDataPackageImport,
  downloadDataPackageArchive,
  prepareDataPackageExport,
  prepareDataPackageImport,
  uploadDataPackageArchive,
} from './api'
import type { DataPackageContents, DataPackageExportPrepareParams } from './types'

type DataPackageGroupKind = keyof DataPackageContents

type ExportCatalogItem = {
  description?: string
  folder?: string
  id: string
  label: string
  meta?: string
  searchText?: string
  tags?: string[]
}

type ExportCatalog = Record<DataPackageGroupKind, ExportCatalogItem[]>
type ExportSelectionState = Record<DataPackageGroupKind, string[]>

const dataPackageGroupOrder: DataPackageGroupKind[] = [
  'presets',
  'schemas',
  'lorebooks',
  'player_profiles',
  'characters',
  'story_resources',
  'stories',
]

const allCharacterFolderValue = '__all__'
const unfiledCharacterFolderValue = '__unfiled__'

function createEmptyExportCatalog(): ExportCatalog {
  return {
    characters: [],
    lorebooks: [],
    player_profiles: [],
    presets: [],
    schemas: [],
    stories: [],
    story_resources: [],
  }
}

function createEmptySelectionState(): ExportSelectionState {
  return {
    characters: [],
    lorebooks: [],
    player_profiles: [],
    presets: [],
    schemas: [],
    stories: [],
    story_resources: [],
  }
}

function trimSingleLine(value: string | null | undefined) {
  return (value ?? '').replace(/\s+/g, ' ').trim()
}

function truncateText(value: string | null | undefined, maxLength = 92) {
  const normalized = trimSingleLine(value)

  if (normalized.length <= maxLength) {
    return normalized
  }

  return `${normalized.slice(0, Math.max(0, maxLength - 1)).trimEnd()}…`
}

function compareCatalogItems(a: ExportCatalogItem, b: ExportCatalogItem) {
  return a.label.localeCompare(b.label, 'zh-Hans-CN-u-co-pinyin')
}

function compareTextItems(a: string, b: string) {
  return a.localeCompare(b, 'zh-Hans-CN-u-co-pinyin')
}

function normalizeCharacterFolder(folder: string | null | undefined) {
  return normalizeCharacterFolderRegistryName(folder ?? '')
}

function buildCharacterExportSearchText(character: CharacterSummary) {
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

function buildExportCatalog(args: {
  characters: CharacterSummary[]
  lorebooks: Lorebook[]
  playerProfiles: PlayerProfile[]
  presets: Preset[]
  schemas: SchemaResource[]
  stories: StorySummary[]
  storyResources: StoryResource[]
}): ExportCatalog {
  return {
    presets: args.presets
      .map((preset) => ({
        id: preset.preset_id,
        label: trimSingleLine(preset.display_name) || preset.preset_id,
        meta: preset.preset_id,
      }))
      .sort(compareCatalogItems),
    schemas: args.schemas
      .map((schema) => ({
        id: schema.schema_id,
        label: trimSingleLine(schema.display_name) || schema.schema_id,
        meta: schema.schema_id,
      }))
      .sort(compareCatalogItems),
    lorebooks: args.lorebooks
      .map((lorebook) => ({
        id: lorebook.lorebook_id,
        label: trimSingleLine(lorebook.display_name) || lorebook.lorebook_id,
        meta: lorebook.lorebook_id,
      }))
      .sort(compareCatalogItems),
    player_profiles: args.playerProfiles
      .map((profile) => ({
        description: truncateText(profile.description, 88),
        id: profile.player_profile_id,
        label: trimSingleLine(profile.display_name) || profile.player_profile_id,
        meta: profile.player_profile_id,
      }))
      .sort(compareCatalogItems),
    characters: args.characters
      .map((character) => ({
        description: truncateText(character.personality, 88),
        folder: normalizeCharacterFolder(character.folder),
        id: character.character_id,
        label: trimSingleLine(character.name) || character.character_id,
        meta: character.character_id,
        searchText: buildCharacterExportSearchText(character),
        tags: Array.from(
          new Set(character.tags.map((tag) => trimSingleLine(tag)).filter(Boolean)),
        ).sort(compareTextItems),
      }))
      .sort(compareCatalogItems),
    story_resources: args.storyResources
      .map((resource) => ({
        description: truncateText(resource.story_concept, 88),
        id: resource.resource_id,
        label: resource.resource_id,
      }))
      .sort(compareCatalogItems),
    stories: args.stories
      .map((story) => ({
        description: truncateText(story.introduction, 88),
        id: story.story_id,
        label: trimSingleLine(story.display_name) || story.story_id,
        meta: story.story_id,
      }))
      .sort(compareCatalogItems),
  }
}

function pruneSelectionState(
  currentSelection: ExportSelectionState,
  catalog: ExportCatalog,
): ExportSelectionState {
  return dataPackageGroupOrder.reduce<ExportSelectionState>((nextSelection, group) => {
    const availableIds = new Set(catalog[group].map((item) => item.id))
    nextSelection[group] = currentSelection[group].filter((itemId) => availableIds.has(itemId))
    return nextSelection
  }, createEmptySelectionState())
}

function buildExportParams(
  selection: ExportSelectionState,
  includeDependencies: boolean,
): DataPackageExportPrepareParams {
  return {
    character_ids: selection.characters,
    include_dependencies: includeDependencies,
    lorebook_ids: selection.lorebooks,
    player_profile_ids: selection.player_profiles,
    preset_ids: selection.presets,
    schema_ids: selection.schemas,
    story_ids: selection.stories,
    story_resource_ids: selection.story_resources,
  }
}

function formatBytes(sizeBytes: number) {
  if (!Number.isFinite(sizeBytes) || sizeBytes <= 0) {
    return '0 B'
  }

  const units = ['B', 'KB', 'MB', 'GB']
  const exponent = Math.min(Math.floor(Math.log(sizeBytes) / Math.log(1024)), units.length - 1)
  const value = sizeBytes / 1024 ** exponent
  const digits = exponent === 0 ? 0 : value >= 10 ? 1 : 2

  return `${value.toFixed(digits)} ${units[exponent]}`
}

function getDataPackageGroupLabel(t: TFunction, group: DataPackageGroupKind) {
  switch (group) {
    case 'presets':
      return t('dashboard.dataPackage.groups.presets')
    case 'schemas':
      return t('dashboard.dataPackage.groups.schemas')
    case 'lorebooks':
      return t('dashboard.dataPackage.groups.lorebooks')
    case 'player_profiles':
      return t('dashboard.dataPackage.groups.playerProfiles')
    case 'characters':
      return t('dashboard.dataPackage.groups.characters')
    case 'story_resources':
      return t('dashboard.dataPackage.groups.storyResources')
    case 'stories':
      return t('dashboard.dataPackage.groups.stories')
  }
}

function summarizeDataPackageContents(t: TFunction, contents: DataPackageContents) {
  const parts = dataPackageGroupOrder.flatMap((group) => {
    const count = contents[group].count

    if (count <= 0) {
      return []
    }

    return [
      t('dashboard.dataPackage.summary.segment', {
        count,
        label: getDataPackageGroupLabel(t, group),
      }),
    ]
  })

  return parts.join(' · ')
}

function BasicExportGroupPanel({
  clearLabel,
  disabled,
  emptyLabel,
  items,
  onClear,
  onSelectAll,
  onToggle,
  selectedIds,
  selectAllLabel,
  title,
}: {
  clearLabel: string
  disabled: boolean
  emptyLabel: string
  items: ExportCatalogItem[]
  onClear: () => void
  onSelectAll: () => void
  onToggle: (id: string) => void
  selectedIds: string[]
  selectAllLabel: string
  title: string
}) {
  const selectedIdSet = new Set(selectedIds)

  return (
    <section className="rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] p-4">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div className="space-y-1">
          <h3 className="text-sm font-semibold text-[var(--color-text-primary)]">{title}</h3>
          <p className="text-xs text-[var(--color-text-muted)]">
            {selectedIds.length} / {items.length}
          </p>
        </div>

        <div className="flex items-center gap-2">
          <Button
            disabled={disabled || items.length === 0 || selectedIds.length === items.length}
            onClick={onSelectAll}
            size="sm"
            variant="ghost"
          >
            {selectAllLabel}
          </Button>
          <Button
            disabled={disabled || selectedIds.length === 0}
            onClick={onClear}
            size="sm"
            variant="ghost"
          >
            {clearLabel}
          </Button>
        </div>
      </div>

      {items.length === 0 ? (
        <div className="mt-4 rounded-[1.15rem] border border-dashed border-[var(--color-border-subtle)] px-4 py-5 text-sm text-[var(--color-text-secondary)]">
          {emptyLabel}
        </div>
      ) : (
        <div className="mt-4 space-y-2">
          {items.map((item) => {
            const checked = selectedIdSet.has(item.id)

            return (
              <label
                className={cn(
                  'flex cursor-pointer items-start gap-3 rounded-[1.15rem] border px-3.5 py-3 transition',
                  checked
                    ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)]'
                    : 'border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_82%,transparent)] hover:border-[var(--color-accent-copper-soft)]',
                  disabled && 'cursor-not-allowed opacity-60',
                )}
                key={item.id}
              >
                <input
                  checked={checked}
                  className="mt-1 h-4 w-4 rounded border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] text-[var(--color-accent-gold)]"
                  disabled={disabled}
                  name={`data-package-item-${item.id}`}
                  onChange={() => {
                    onToggle(item.id)
                  }}
                  type="checkbox"
                />
                <div className="min-w-0 flex-1 space-y-1">
                  <p className="truncate text-sm font-medium text-[var(--color-text-primary)]">
                    {item.label}
                  </p>
                  {item.meta ? (
                    <p className="truncate text-xs text-[var(--color-text-muted)]">{item.meta}</p>
                  ) : null}
                  {item.description ? (
                    <p className="line-clamp-2 text-xs leading-6 text-[var(--color-text-secondary)]">
                      {item.description}
                    </p>
                  ) : null}
                </div>
              </label>
            )
          })}
        </div>
      )}
    </section>
  )
}

function CharacterExportGroupPanel({
  activeFolder,
  allLabel,
  characterFolderOptions,
  clearVisibleLabel,
  disabled,
  emptyFolderLabel,
  emptyResultsLabel,
  emptyTagsLabel,
  filteredCharacterCount,
  filteredCharacters,
  folderSearchPlaceholder,
  folderListFilter,
  folderTitle,
  hasUnfiledCharacters,
  items,
  onClearVisible,
  onFolderListFilterChange,
  onSearchQueryChange,
  onSelectAllVisible,
  onSetActiveFolder,
  onToggleCharacter,
  onToggleTag,
  resultsTitle,
  searchPlaceholder,
  searchQuery,
  selectVisibleLabel,
  selectedCharacterTags,
  selectedIds,
  selectionTitle,
  tags,
  tagsTitle,
  totalCountLabel,
  unfiledLabel,
}: {
  activeFolder: string
  allLabel: string
  characterFolderOptions: string[]
  clearVisibleLabel: string
  disabled: boolean
  emptyFolderLabel: string
  emptyResultsLabel: string
  emptyTagsLabel: string
  filteredCharacterCount: number
  filteredCharacters: ExportCatalogItem[]
  folderSearchPlaceholder: string
  folderListFilter: string
  folderTitle: string
  hasUnfiledCharacters: boolean
  items: ExportCatalogItem[]
  onClearVisible: () => void
  onFolderListFilterChange: (value: string) => void
  onSearchQueryChange: (value: string) => void
  onSelectAllVisible: () => void
  onSetActiveFolder: (value: string) => void
  onToggleCharacter: (id: string) => void
  onToggleTag: (tag: string) => void
  resultsTitle: string
  searchPlaceholder: string
  searchQuery: string
  selectVisibleLabel: string
  selectedCharacterTags: string[]
  selectedIds: string[]
  selectionTitle: string
  tags: string[]
  tagsTitle: string
  totalCountLabel: string
  unfiledLabel: string
}) {
  const normalizedFolderListFilter = folderListFilter.trim().toLocaleLowerCase()
  const selectedIdSet = new Set(selectedIds)
  const selectedTagSet = new Set(selectedCharacterTags)
  const visibleCharacterIds = filteredCharacters.map((item) => item.id)
  const allVisibleSelected =
    visibleCharacterIds.length > 0 &&
    visibleCharacterIds.every((itemId) => selectedIdSet.has(itemId))
  const hasVisibleSelected = visibleCharacterIds.some((itemId) => selectedIdSet.has(itemId))

  const filteredFolderOptions = characterFolderOptions.filter((folder) =>
    normalizedFolderListFilter.length === 0
      ? true
      : folder.toLocaleLowerCase().includes(normalizedFolderListFilter),
  )
  const showUnfiledOption =
    hasUnfiledCharacters &&
    (normalizedFolderListFilter.length === 0 ||
      unfiledLabel.toLocaleLowerCase().includes(normalizedFolderListFilter))

  return (
    <section className="rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] p-4">
      <div className="grid gap-4 xl:grid-cols-[15rem_minmax(0,1fr)]">
        <div className="space-y-4">
          <div className="space-y-3 rounded-[1.15rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_78%,transparent)] p-4">
            <div className="space-y-1">
              <h3 className="text-sm font-semibold text-[var(--color-text-primary)]">
                {selectionTitle}
              </h3>
              <p className="text-xs leading-6 text-[var(--color-text-muted)]">{totalCountLabel}</p>
            </div>
            <div className="grid w-full grid-cols-[repeat(auto-fit,minmax(4.5rem,1fr))] gap-2">
              <Button
                className="w-full"
                disabled={disabled || visibleCharacterIds.length === 0 || allVisibleSelected}
                onClick={onSelectAllVisible}
                size="sm"
                variant="ghost"
              >
                {selectVisibleLabel}
              </Button>
              <Button
                className="w-full"
                disabled={disabled || !hasVisibleSelected}
                onClick={onClearVisible}
                size="sm"
                variant="ghost"
              >
                {clearVisibleLabel}
              </Button>
            </div>
          </div>

          <div className="space-y-3 rounded-[1.15rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_78%,transparent)] p-4">
            <div className="space-y-1">
              <h4 className="text-sm font-medium text-[var(--color-text-primary)]">
                {folderTitle}
              </h4>
            </div>

            <Input
              disabled={disabled}
              onChange={(event) => {
                onFolderListFilterChange(event.target.value)
              }}
              placeholder={folderSearchPlaceholder}
              value={folderListFilter}
            />

            <div className="max-h-[16rem] space-y-2 overflow-y-auto pr-1">
              <button
                className={cn(
                  'w-full rounded-[1rem] border px-3 py-2.5 text-left text-sm transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-focus-ring)] disabled:pointer-events-none disabled:opacity-45',
                  activeFolder === allCharacterFolderValue
                    ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] text-[var(--color-text-primary)]'
                    : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] text-[var(--color-text-secondary)] hover:border-[var(--color-accent-copper-soft)] hover:text-[var(--color-text-primary)]',
                )}
                disabled={disabled}
                onClick={() => {
                  onSetActiveFolder(allCharacterFolderValue)
                }}
                type="button"
              >
                {allLabel}
              </button>

              {showUnfiledOption ? (
                <button
                  className={cn(
                    'w-full rounded-[1rem] border px-3 py-2.5 text-left text-sm transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-focus-ring)] disabled:pointer-events-none disabled:opacity-45',
                    activeFolder === unfiledCharacterFolderValue
                      ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] text-[var(--color-text-primary)]'
                      : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] text-[var(--color-text-secondary)] hover:border-[var(--color-accent-copper-soft)] hover:text-[var(--color-text-primary)]',
                  )}
                  disabled={disabled}
                  onClick={() => {
                    onSetActiveFolder(unfiledCharacterFolderValue)
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
                      onSetActiveFolder(folder)
                    }}
                    type="button"
                  >
                    {folder}
                  </button>
                ))
              ) : !showUnfiledOption ? (
                <div className="rounded-[1rem] border border-dashed border-[var(--color-border-subtle)] px-3 py-3 text-xs leading-6 text-[var(--color-text-muted)]">
                  {emptyFolderLabel}
                </div>
              ) : null}
            </div>
          </div>

          <div className="space-y-3 rounded-[1.15rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_78%,transparent)] p-4">
            <div className="space-y-1">
              <h4 className="text-sm font-medium text-[var(--color-text-primary)]">{tagsTitle}</h4>
              <p className="text-xs leading-6 text-[var(--color-text-muted)]">
                {selectedCharacterTags.length} / {tags.length}
              </p>
            </div>

            {tags.length > 0 ? (
              <div className="flex flex-wrap gap-2">
                {tags.map((tag) => {
                  const selected = selectedTagSet.has(tag)

                  return (
                    <button
                      className={cn(
                        'rounded-full border px-3 py-1.5 text-xs transition focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--color-focus-ring)]',
                        selected
                          ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] text-[var(--color-text-primary)]'
                          : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]',
                      )}
                      disabled={disabled}
                      key={tag}
                      onClick={() => {
                        onToggleTag(tag)
                      }}
                      type="button"
                    >
                      #{tag}
                    </button>
                  )
                })}
              </div>
            ) : (
              <div className="rounded-[1rem] border border-dashed border-[var(--color-border-subtle)] px-3 py-3 text-xs leading-6 text-[var(--color-text-muted)]">
                {emptyTagsLabel}
              </div>
            )}
          </div>
        </div>

        <div className="space-y-3 rounded-[1.15rem] border border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_78%,transparent)] p-4">
          <div className="space-y-1">
            <h4 className="text-sm font-medium text-[var(--color-text-primary)]">{resultsTitle}</h4>
            <p className="text-xs leading-6 text-[var(--color-text-muted)]">
              {filteredCharacterCount} / {items.length}
            </p>
          </div>

          <Input
            disabled={disabled}
            onChange={(event) => {
              onSearchQueryChange(event.target.value)
            }}
            placeholder={searchPlaceholder}
            value={searchQuery}
          />

          {filteredCharacters.length > 0 ? (
            <div className="grid gap-2 sm:grid-cols-2">
              {filteredCharacters.map((item) => {
                const checked = selectedIdSet.has(item.id)
                const folderLabel = item.folder?.length ? item.folder : unfiledLabel

                return (
                  <label
                    className={cn(
                      'flex cursor-pointer items-start gap-3 rounded-[1.15rem] border px-3.5 py-3 transition',
                      checked
                        ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)]'
                        : 'border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_82%,transparent)] hover:border-[var(--color-accent-copper-soft)]',
                      disabled && 'cursor-not-allowed opacity-60',
                    )}
                    key={item.id}
                  >
                    <input
                      checked={checked}
                      className="mt-1 h-4 w-4 rounded border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] text-[var(--color-accent-gold)]"
                      disabled={disabled}
                      name={`data-package-character-${item.id}`}
                      onChange={() => {
                        onToggleCharacter(item.id)
                      }}
                      type="checkbox"
                    />
                    <div className="min-w-0 flex-1 space-y-1.5">
                      <p className="truncate text-sm font-medium text-[var(--color-text-primary)]">
                        {item.label}
                      </p>
                      {item.meta ? (
                        <p className="truncate text-xs text-[var(--color-text-muted)]">
                          {item.meta}
                        </p>
                      ) : null}
                      <div className="flex flex-wrap gap-1.5 text-[0.72rem]">
                        <span className="rounded-full border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-2 py-1 text-[var(--color-text-muted)]">
                          {folderLabel}
                        </span>
                        {item.tags?.slice(0, 3).map((tag) => (
                          <span
                            className="rounded-full border border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] px-2 py-1 text-[var(--color-text-muted)]"
                            key={tag}
                          >
                            #{tag}
                          </span>
                        ))}
                      </div>
                      {item.description ? (
                        <p className="line-clamp-2 text-xs leading-6 text-[var(--color-text-secondary)]">
                          {item.description}
                        </p>
                      ) : null}
                    </div>
                  </label>
                )
              })}
            </div>
          ) : (
            <div className="rounded-[1.15rem] border border-dashed border-[var(--color-border-subtle)] px-4 py-5 text-sm text-[var(--color-text-secondary)]">
              {emptyResultsLabel}
            </div>
          )}
        </div>
      </div>
    </section>
  )
}

function ExportCatalogSkeleton() {
  return (
    <div className="space-y-4">
      <div className="h-12 w-full animate-pulse rounded-[1.2rem] bg-[var(--color-bg-elevated)]" />
      <div className="rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] p-4">
        <div className="flex items-start justify-between gap-3">
          <div className="space-y-2">
            <div className="h-4 w-24 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
            <div className="h-3 w-16 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
          </div>
          <div className="flex gap-2">
            <div className="h-8 w-20 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
            <div className="h-8 w-20 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
          </div>
        </div>
        <div className="mt-4 space-y-2">
          {Array.from({ length: 5 }).map((_, index) => (
            <div
              className="h-16 animate-pulse rounded-[1.15rem] bg-[var(--color-bg-panel)]"
              key={index}
            />
          ))}
        </div>
      </div>
    </div>
  )
}

export function DashboardDataPackageActions({
  onImported,
}: {
  onImported: () => Promise<void> | void
}) {
  const { t } = useTranslation()
  const { pushToast } = useToast()
  const importInputRef = useRef<HTMLInputElement | null>(null)
  const [catalog, setCatalog] = useState<ExportCatalog>(() => createEmptyExportCatalog())
  const [selectedItems, setSelectedItems] = useState<ExportSelectionState>(() =>
    createEmptySelectionState(),
  )
  const [includeDependencies, setIncludeDependencies] = useState(true)
  const [isExportDialogOpen, setIsExportDialogOpen] = useState(false)
  const [isImportDialogOpen, setIsImportDialogOpen] = useState(false)
  const [catalogError, setCatalogError] = useState<string | null>(null)
  const [isCatalogLoading, setIsCatalogLoading] = useState(false)
  const [isExporting, setIsExporting] = useState(false)
  const [isImporting, setIsImporting] = useState(false)
  const [pendingImportFile, setPendingImportFile] = useState<File | null>(null)
  const [activeExportTab, setActiveExportTab] = useState<DataPackageGroupKind>('presets')
  const [activeCharacterFolder, setActiveCharacterFolder] =
    useState<string>(allCharacterFolderValue)
  const [characterFolderListFilter, setCharacterFolderListFilter] = useState('')
  const [characterSearchQuery, setCharacterSearchQuery] = useState('')
  const [selectedCharacterTags, setSelectedCharacterTags] = useState<string[]>([])

  const selectedExportCount = useMemo(
    () => dataPackageGroupOrder.reduce((count, group) => count + selectedItems[group].length, 0),
    [selectedItems],
  )

  const loadExportCatalog = useCallback(
    async (signal?: AbortSignal) => {
      setIsCatalogLoading(true)
      setCatalogError(null)

      try {
        const [presets, schemas, lorebooks, playerProfiles, characters, storyResources, stories] =
          await Promise.all([
            listPresets(signal),
            listSchemas(signal),
            listLorebooks(signal),
            listPlayerProfiles(signal),
            listCharacters(signal),
            listStoryResources(signal),
            listStories(signal),
          ])

        if (signal?.aborted) {
          return
        }

        const nextCatalog = buildExportCatalog({
          characters,
          lorebooks,
          playerProfiles,
          presets,
          schemas,
          stories,
          storyResources,
        })

        setCatalog(nextCatalog)
        setSelectedItems((currentSelection) => pruneSelectionState(currentSelection, nextCatalog))
      } catch (error) {
        if (signal?.aborted) {
          return
        }

        const message =
          error instanceof Error
            ? error.message
            : t('dashboard.dataPackage.feedback.loadCatalogFailed')

        setCatalogError(message)
        pushToast({
          message,
          tone: 'error',
        })
      } finally {
        if (!signal?.aborted) {
          setIsCatalogLoading(false)
        }
      }
    },
    [pushToast, t],
  )

  useEffect(() => {
    if (!isExportDialogOpen) {
      return
    }

    const controller = new AbortController()
    void loadExportCatalog(controller.signal)

    return () => {
      controller.abort()
    }
  }, [isExportDialogOpen, loadExportCatalog])

  const characterItems = catalog.characters
  const normalizedCharacterSearchQuery = characterSearchQuery.trim().toLocaleLowerCase()
  const characterFolderOptions = useMemo(
    () =>
      Array.from(
        new Set(
          characterItems.map((item) => item.folder ?? '').filter((folder) => folder.length > 0),
        ),
      ).sort(compareTextItems),
    [characterItems],
  )
  const characterTagOptions = useMemo(
    () =>
      Array.from(
        new Set(characterItems.flatMap((item) => item.tags ?? []).filter((tag) => tag.length > 0)),
      ).sort(compareTextItems),
    [characterItems],
  )
  const hasUnfiledCharacters = useMemo(
    () => characterItems.some((item) => !item.folder),
    [characterItems],
  )
  const filteredCharacters = useMemo(
    () =>
      characterItems.filter((item) => {
        const matchesFolder =
          activeCharacterFolder === allCharacterFolderValue ||
          (activeCharacterFolder === unfiledCharacterFolderValue
            ? !item.folder
            : item.folder === activeCharacterFolder)
        const matchesSearch =
          normalizedCharacterSearchQuery.length === 0 ||
          (item.searchText ?? '').includes(normalizedCharacterSearchQuery)
        const matchesTags = selectedCharacterTags.every((tag) => item.tags?.includes(tag))

        return matchesFolder && matchesSearch && matchesTags
      }),
    [activeCharacterFolder, characterItems, normalizedCharacterSearchQuery, selectedCharacterTags],
  )

  useEffect(() => {
    const availableTags = new Set(characterTagOptions)
    setSelectedCharacterTags((currentTags) => currentTags.filter((tag) => availableTags.has(tag)))
  }, [characterTagOptions])

  useEffect(() => {
    if (activeCharacterFolder === allCharacterFolderValue) {
      return
    }

    if (activeCharacterFolder === unfiledCharacterFolderValue) {
      if (!hasUnfiledCharacters) {
        setActiveCharacterFolder(allCharacterFolderValue)
      }
      return
    }

    if (!characterFolderOptions.includes(activeCharacterFolder)) {
      setActiveCharacterFolder(allCharacterFolderValue)
    }
  }, [activeCharacterFolder, characterFolderOptions, hasUnfiledCharacters])

  function toggleSelectedItem(kind: DataPackageGroupKind, itemId: string) {
    setSelectedItems((currentSelection) => ({
      ...currentSelection,
      [kind]: currentSelection[kind].includes(itemId)
        ? currentSelection[kind].filter((currentId) => currentId !== itemId)
        : [...currentSelection[kind], itemId],
    }))
  }

  function selectGroupItems(kind: DataPackageGroupKind, itemIds: string[]) {
    if (itemIds.length === 0) {
      return
    }

    setSelectedItems((currentSelection) => ({
      ...currentSelection,
      [kind]: Array.from(new Set([...currentSelection[kind], ...itemIds])),
    }))
  }

  function clearGroupItems(kind: DataPackageGroupKind, itemIds?: string[]) {
    setSelectedItems((currentSelection) => ({
      ...currentSelection,
      [kind]:
        itemIds === undefined
          ? []
          : currentSelection[kind].filter((itemId) => !itemIds.includes(itemId)),
    }))
  }

  function toggleCharacterTag(tag: string) {
    setSelectedCharacterTags((currentTags) =>
      currentTags.includes(tag)
        ? currentTags.filter((currentTag) => currentTag !== tag)
        : [...currentTags, tag],
    )
  }

  async function handleExport() {
    if (selectedExportCount === 0) {
      return
    }

    setIsExporting(true)

    try {
      const prepared = await prepareDataPackageExport(
        buildExportParams(selectedItems, includeDependencies),
      )

      await downloadDataPackageArchive({
        archive: prepared.archive,
        fallbackFileName: `sillystage-data-package-${prepared.export_id}.zip`,
      })

      setIsExportDialogOpen(false)
      pushToast({
        message: t('dashboard.dataPackage.feedback.exported', {
          summary: summarizeDataPackageContents(t, prepared.contents),
        }),
        tone: 'success',
      })
    } catch (error) {
      pushToast({
        message:
          error instanceof Error ? error.message : t('dashboard.dataPackage.feedback.exportFailed'),
        tone: 'error',
      })
    } finally {
      setIsExporting(false)
    }
  }

  function handleImportFileSelected(event: ChangeEvent<HTMLInputElement>) {
    const nextFile = event.target.files?.[0] ?? null
    event.target.value = ''

    if (!nextFile) {
      return
    }

    if (!nextFile.name.toLowerCase().endsWith('.zip')) {
      pushToast({
        message: t('dashboard.dataPackage.feedback.invalidFile'),
        tone: 'error',
      })
      return
    }

    setPendingImportFile(nextFile)
    setIsImportDialogOpen(true)
  }

  async function handleConfirmImport() {
    if (!pendingImportFile) {
      return
    }

    setIsImporting(true)

    try {
      const prepared = await prepareDataPackageImport()
      await uploadDataPackageArchive({
        archive: prepared.archive,
        file: pendingImportFile,
      })
      const committed = await commitDataPackageImport(prepared.import_id)

      setIsImportDialogOpen(false)
      setPendingImportFile(null)
      pushToast({
        message: t('dashboard.dataPackage.feedback.imported', {
          summary: summarizeDataPackageContents(t, committed.contents),
        }),
        tone: 'success',
      })
      await onImported()
    } catch (error) {
      setIsImportDialogOpen(false)
      setPendingImportFile(null)
      pushToast({
        message:
          error instanceof Error ? error.message : t('dashboard.dataPackage.feedback.importFailed'),
        tone: 'error',
      })
    } finally {
      setIsImporting(false)
    }
  }

  const exportTabItems = useMemo(
    () =>
      dataPackageGroupOrder.map((group) => ({
        label: getDataPackageGroupLabel(t, group),
        value: group,
      })),
    [t],
  )

  const activeExportGroupItems = catalog[activeExportTab]

  return (
    <>
      <div className="flex flex-wrap justify-end gap-2">
        <IconButton
          disabled={isImporting}
          icon={<FontAwesomeIcon icon={faDownload} />}
          label={t('dashboard.dataPackage.actions.import')}
          onClick={() => {
            importInputRef.current?.click()
          }}
          variant="secondary"
        />
        <IconButton
          icon={<FontAwesomeIcon icon={faUpload} />}
          label={t('dashboard.dataPackage.actions.export')}
          onClick={() => {
            setIsExportDialogOpen(true)
          }}
          variant="primary"
        />
      </div>

      <input
        accept=".zip,application/zip"
        className="hidden"
        name="dashboard_data_package_import"
        onChange={handleImportFileSelected}
        ref={importInputRef}
        type="file"
      />

      <Dialog
        onOpenChange={(open) => {
          if (!isExporting) {
            setIsExportDialogOpen(open)
          }
        }}
        open={isExportDialogOpen}
      >
        <DialogContent className="w-[min(96vw,78rem)] overflow-hidden">
          <DialogHeader className="border-b border-[var(--color-border-subtle)]">
            <div className="flex flex-wrap items-start justify-between gap-3">
              <div className="space-y-3">
                <DialogTitle>{t('dashboard.dataPackage.dialogs.export.title')}</DialogTitle>
                <DialogDescription>
                  {t('dashboard.dataPackage.dialogs.export.description')}
                </DialogDescription>
              </div>
              <Badge variant="info">
                {t('dashboard.dataPackage.dialogs.export.selectedCount', {
                  count: selectedExportCount,
                })}
              </Badge>
            </div>
          </DialogHeader>

          <DialogBody className="max-h-[calc(92vh-13rem)] space-y-5 pt-6">
            {catalogError ? (
              <div className="rounded-[1.45rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_78%,transparent)] px-5 py-6">
                <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
                  {catalogError}
                </p>
                <div className="mt-4">
                  <Button
                    onClick={() => {
                      void loadExportCatalog()
                    }}
                    size="sm"
                    variant="secondary"
                  >
                    <FontAwesomeIcon icon={faRotateRight} />
                    {t('dashboard.dataPackage.actions.retry')}
                  </Button>
                </div>
              </div>
            ) : isCatalogLoading ? (
              <ExportCatalogSkeleton />
            ) : (
              <div className="space-y-4">
                <div className="overflow-x-auto pb-1">
                  <SegmentedSelector
                    ariaLabel={t('dashboard.dataPackage.dialogs.export.tabsLabel')}
                    className="min-w-max"
                    items={exportTabItems}
                    layoutId="dashboard-data-package-export-tabs"
                    onValueChange={(value) => {
                      setActiveExportTab(value as DataPackageGroupKind)
                    }}
                    value={activeExportTab}
                  />
                </div>

                {activeExportTab === 'characters' ? (
                  <CharacterExportGroupPanel
                    activeFolder={activeCharacterFolder}
                    allLabel={t('dashboard.dataPackage.filters.all')}
                    characterFolderOptions={characterFolderOptions}
                    clearVisibleLabel={t('dashboard.dataPackage.actions.clearVisible')}
                    disabled={isExporting}
                    emptyFolderLabel={t('dashboard.dataPackage.filters.folderEmpty')}
                    emptyResultsLabel={t('dashboard.dataPackage.filters.charactersEmpty')}
                    emptyTagsLabel={t('dashboard.dataPackage.filters.tagsEmpty')}
                    filteredCharacterCount={filteredCharacters.length}
                    filteredCharacters={filteredCharacters}
                    folderSearchPlaceholder={t(
                      'dashboard.dataPackage.filters.folderSearchPlaceholder',
                    )}
                    folderListFilter={characterFolderListFilter}
                    folderTitle={t('dashboard.dataPackage.filters.folderTitle')}
                    hasUnfiledCharacters={hasUnfiledCharacters}
                    items={characterItems}
                    onClearVisible={() => {
                      clearGroupItems(
                        'characters',
                        filteredCharacters.map((item) => item.id),
                      )
                    }}
                    onFolderListFilterChange={setCharacterFolderListFilter}
                    onSearchQueryChange={setCharacterSearchQuery}
                    onSelectAllVisible={() => {
                      selectGroupItems(
                        'characters',
                        filteredCharacters.map((item) => item.id),
                      )
                    }}
                    onSetActiveFolder={setActiveCharacterFolder}
                    onToggleCharacter={(itemId) => {
                      toggleSelectedItem('characters', itemId)
                    }}
                    onToggleTag={toggleCharacterTag}
                    resultsTitle={t('dashboard.dataPackage.filters.charactersTitle')}
                    searchPlaceholder={t('dashboard.dataPackage.filters.searchPlaceholder')}
                    searchQuery={characterSearchQuery}
                    selectVisibleLabel={t('dashboard.dataPackage.actions.selectVisible')}
                    selectedCharacterTags={selectedCharacterTags}
                    selectedIds={selectedItems.characters}
                    selectionTitle={t('dashboard.dataPackage.filters.selectionTitle')}
                    tags={characterTagOptions}
                    tagsTitle={t('dashboard.dataPackage.filters.tagsTitle')}
                    totalCountLabel={t('dashboard.dataPackage.filters.charactersSummary', {
                      count: selectedItems.characters.length,
                      total: characterItems.length,
                    })}
                    unfiledLabel={t('dashboard.dataPackage.filters.unfiled')}
                  />
                ) : (
                  <BasicExportGroupPanel
                    clearLabel={t('dashboard.dataPackage.actions.clear')}
                    disabled={isExporting}
                    emptyLabel={t('dashboard.dataPackage.dialogs.export.emptyGroup')}
                    items={activeExportGroupItems}
                    onClear={() => {
                      clearGroupItems(activeExportTab)
                    }}
                    onSelectAll={() => {
                      selectGroupItems(
                        activeExportTab,
                        activeExportGroupItems.map((item) => item.id),
                      )
                    }}
                    onToggle={(itemId) => {
                      toggleSelectedItem(activeExportTab, itemId)
                    }}
                    selectedIds={selectedItems[activeExportTab]}
                    selectAllLabel={t('dashboard.dataPackage.actions.selectAll')}
                    title={getDataPackageGroupLabel(t, activeExportTab)}
                  />
                )}
              </div>
            )}
          </DialogBody>

          <DialogFooter className="justify-between gap-3">
            <div className="flex min-w-0 flex-1 items-start gap-3">
              <Switch
                checked={includeDependencies}
                disabled={isExporting}
                onCheckedChange={setIncludeDependencies}
                size="sm"
              />
              <div className="min-w-0 space-y-1">
                <p className="text-sm font-medium text-[var(--color-text-primary)]">
                  {t('dashboard.dataPackage.dialogs.export.includeDependencies')}
                </p>
                <p className="text-xs leading-6 text-[var(--color-text-secondary)]">
                  {t('dashboard.dataPackage.dialogs.export.includeDependenciesHint')}
                </p>
              </div>
            </div>

            <div className="flex flex-wrap justify-end gap-2">
              <Button
                disabled={isExporting}
                onClick={() => {
                  setIsExportDialogOpen(false)
                }}
                variant="secondary"
              >
                {t('dashboard.dataPackage.actions.cancel')}
              </Button>
              <Button
                disabled={selectedExportCount === 0 || isExporting || isCatalogLoading}
                onClick={() => {
                  void handleExport()
                }}
              >
                <FontAwesomeIcon icon={faDownload} />
                {isExporting
                  ? t('dashboard.dataPackage.actions.exporting')
                  : t('dashboard.dataPackage.actions.export')}
              </Button>
            </div>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <Dialog
        onOpenChange={(open) => {
          if (!isImporting) {
            setIsImportDialogOpen(open)
            if (!open) {
              setPendingImportFile(null)
            }
          }
        }}
        open={isImportDialogOpen}
      >
        <DialogContent className="w-[min(92vw,34rem)]">
          <DialogHeader className="border-b border-[var(--color-border-subtle)]">
            <DialogTitle>{t('dashboard.dataPackage.dialogs.import.title')}</DialogTitle>
            <DialogDescription>
              {t('dashboard.dataPackage.dialogs.import.description')}
            </DialogDescription>
          </DialogHeader>

          <DialogBody className="space-y-4 pt-6">
            <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] p-4">
              <div className="space-y-3">
                <div className="space-y-1">
                  <p className="text-xs uppercase tracking-[0.08em] text-[var(--color-text-muted)]">
                    {t('dashboard.dataPackage.dialogs.import.fileName')}
                  </p>
                  <p className="break-all text-sm font-medium text-[var(--color-text-primary)]">
                    {pendingImportFile?.name ?? '—'}
                  </p>
                </div>
                <div className="space-y-1">
                  <p className="text-xs uppercase tracking-[0.08em] text-[var(--color-text-muted)]">
                    {t('dashboard.dataPackage.dialogs.import.fileSize')}
                  </p>
                  <p className="text-sm font-medium text-[var(--color-text-primary)]">
                    {pendingImportFile ? formatBytes(pendingImportFile.size) : '—'}
                  </p>
                </div>
              </div>
            </div>

            <div className="rounded-[1.35rem] border border-[var(--color-state-warning-line)] bg-[var(--color-state-warning-soft)] px-4 py-4 text-sm leading-7 text-[var(--color-text-primary)]">
              {t('dashboard.dataPackage.dialogs.import.conflictHint')}
            </div>
          </DialogBody>

          <DialogFooter className="justify-end gap-2">
            <Button
              disabled={isImporting}
              onClick={() => {
                setIsImportDialogOpen(false)
                setPendingImportFile(null)
              }}
              variant="secondary"
            >
              {t('dashboard.dataPackage.actions.cancel')}
            </Button>
            <Button
              disabled={!pendingImportFile || isImporting}
              onClick={() => {
                void handleConfirmImport()
              }}
            >
              <FontAwesomeIcon icon={faUpload} />
              {isImporting
                ? t('dashboard.dataPackage.actions.importing')
                : t('dashboard.dataPackage.actions.importConfirm')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}
