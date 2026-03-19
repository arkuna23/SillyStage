import type { ChangeEvent } from 'react'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { faDownload } from '@fortawesome/free-solid-svg-icons/faDownload'
import { faRotateRight } from '@fortawesome/free-solid-svg-icons/faRotateRight'
import { faUpload } from '@fortawesome/free-solid-svg-icons/faUpload'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import type { TFunction } from 'i18next'
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
import { Switch } from '../../components/ui/switch'
import { useToast } from '../../components/ui/toast-context'
import { cn } from '../../lib/cn'
import { listPresets } from '../apis/api'
import type { Preset } from '../apis/types'
import { listCharacters } from '../characters/api'
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
import type {
  DataPackageContents,
  DataPackageExportPrepareParams,
} from './types'

type DataPackageGroupKind = keyof DataPackageContents

type ExportCatalogItem = {
  description?: string
  id: string
  label: string
  meta?: string
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
        id: character.character_id,
        label: trimSingleLine(character.name) || character.character_id,
        meta: character.character_id,
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

function ExportGroupCard({
  clearLabel,
  disabled,
  emptyLabel,
  items,
  kind,
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
  kind: DataPackageGroupKind
  onClear: (kind: DataPackageGroupKind) => void
  onSelectAll: (kind: DataPackageGroupKind) => void
  onToggle: (kind: DataPackageGroupKind, id: string) => void
  selectedIds: string[]
  selectAllLabel: string
  title: string
}) {
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
            onClick={() => {
              onSelectAll(kind)
            }}
            size="sm"
            variant="ghost"
          >
            {selectAllLabel}
          </Button>
          <Button
            disabled={disabled || selectedIds.length === 0}
            onClick={() => {
              onClear(kind)
            }}
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
            const checked = selectedIds.includes(item.id)

            return (
              <label
                className={cn(
                  'flex cursor-pointer items-start gap-3 rounded-[1.15rem] border px-3.5 py-3 transition',
                  checked
                    ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)]'
                    : 'border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_82%,transparent)] hover:border-[var(--color-accent-copper-soft)]',
                  disabled && 'cursor-not-allowed opacity-60',
                )}
                key={`${kind}:${item.id}`}
              >
                <input
                  checked={checked}
                  className="mt-1 h-4 w-4 rounded border-[var(--color-border-subtle)] bg-[var(--color-bg-panel)] text-[var(--color-accent-gold)]"
                  disabled={disabled}
                  name={`data-package-${kind}-${item.id}`}
                  onChange={() => {
                    onToggle(kind, item.id)
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

function ExportCatalogSkeleton() {
  return (
    <div className="grid gap-4 xl:grid-cols-2">
      {Array.from({ length: 6 }).map((_, index) => (
        <div
          className="rounded-[1.45rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] p-4"
          key={index}
        >
          <div className="flex items-start justify-between gap-3">
            <div className="space-y-2">
              <div className="h-4 w-20 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
              <div className="h-3 w-14 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
            </div>
            <div className="flex gap-2">
              <div className="h-8 w-14 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
              <div className="h-8 w-14 animate-pulse rounded-full bg-[var(--color-bg-panel)]" />
            </div>
          </div>
          <div className="mt-4 space-y-2">
            {Array.from({ length: 3 }).map((__, rowIndex) => (
              <div
                className="h-16 animate-pulse rounded-[1.15rem] bg-[var(--color-bg-panel)]"
                key={rowIndex}
              />
            ))}
          </div>
        </div>
      ))}
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

  const selectedExportCount = useMemo(
    () =>
      dataPackageGroupOrder.reduce(
        (count, group) => count + selectedItems[group].length,
        0,
      ),
    [selectedItems],
  )

  const loadExportCatalog = useCallback(
    async (signal?: AbortSignal) => {
      setIsCatalogLoading(true)
      setCatalogError(null)

      try {
        const [
          presets,
          schemas,
          lorebooks,
          playerProfiles,
          characters,
          storyResources,
          stories,
        ] = await Promise.all([
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
        setSelectedItems((currentSelection) =>
          pruneSelectionState(currentSelection, nextCatalog),
        )
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

  function toggleSelectedItem(kind: DataPackageGroupKind, itemId: string) {
    setSelectedItems((currentSelection) => ({
      ...currentSelection,
      [kind]: currentSelection[kind].includes(itemId)
        ? currentSelection[kind].filter((currentId) => currentId !== itemId)
        : [...currentSelection[kind], itemId],
    }))
  }

  function selectAllGroup(kind: DataPackageGroupKind) {
    setSelectedItems((currentSelection) => ({
      ...currentSelection,
      [kind]: catalog[kind].map((item) => item.id),
    }))
  }

  function clearGroup(kind: DataPackageGroupKind) {
    setSelectedItems((currentSelection) => ({
      ...currentSelection,
      [kind]: [],
    }))
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
          error instanceof Error
            ? error.message
            : t('dashboard.dataPackage.feedback.exportFailed'),
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
          error instanceof Error
            ? error.message
            : t('dashboard.dataPackage.feedback.importFailed'),
        tone: 'error',
      })
    } finally {
      setIsImporting(false)
    }
  }

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
              <div className="grid gap-4 xl:grid-cols-2">
                {dataPackageGroupOrder.map((group) => (
                  <ExportGroupCard
                    clearLabel={t('dashboard.dataPackage.actions.clear')}
                    disabled={isExporting}
                    emptyLabel={t('dashboard.dataPackage.dialogs.export.emptyGroup')}
                    items={catalog[group]}
                    key={group}
                    kind={group}
                    onClear={clearGroup}
                    onSelectAll={selectAllGroup}
                    onToggle={toggleSelectedItem}
                    selectedIds={selectedItems[group]}
                    selectAllLabel={t('dashboard.dataPackage.actions.selectAll')}
                    title={getDataPackageGroupLabel(t, group)}
                  />
                ))}
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
