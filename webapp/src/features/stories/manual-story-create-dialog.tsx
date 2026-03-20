import { useEffect, useId, useMemo, useState, type ReactNode } from 'react'
import { useTranslation } from 'react-i18next'

import { appPaths } from '../../app/paths'
import { Badge } from '../../components/ui/badge'
import { Button } from '../../components/ui/button'
import {
  Dialog,
  DialogBody,
  DialogClose,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../../components/ui/dialog'
import { DialogRouteButton } from '../../components/ui/dialog-route-button'
import { Input } from '../../components/ui/input'
import { Select, type SelectOption } from '../../components/ui/select'
import { Textarea } from '../../components/ui/textarea'
import { useToast, useToastMessage } from '../../components/ui/toast-context'
import type { CharacterSummary } from '../characters/types'
import { listSchemas } from '../schemas/api'
import type { SchemaResource } from '../schemas/types'
import type { StoryResource } from '../story-resources/types'
import { createStory } from './api'
import { ManualStoryGraphDialog } from './manual-story-graph-dialog'
import {
  createStoryCommonVariableDrafts,
  serializeStoryCommonVariableDrafts,
  type StoryCommonVariableDraft,
  type StoryCommonVariableDraftErrors,
  validateStoryCommonVariableDrafts,
} from './story-common-variable-drafts'
import { useStoryCommonVariableSchemaCatalog } from './story-common-variable-schema-catalog'
import {
  normalizeControlledStoryGraph,
  useStoryGraphEditorController,
} from './story-graph-editor-controller'
import { StoryCommonVariablesEditor } from './story-common-variables-editor'
import {
  createDefaultStoryGraph,
  getStoryGraphValidationMessage,
} from './story-graph-editor-utils'
import type { StoryDetail, StoryGraph } from './types'

type ManualStoryCreateDialogProps = {
  availableCharacters: ReadonlyArray<CharacterSummary>
  onCompleted: (result: { message: string; story: StoryDetail }) => Promise<void> | void
  onOpenChange: (open: boolean) => void
  open: boolean
  resources: ReadonlyArray<StoryResource>
}

type FormState = {
  commonVariables: StoryCommonVariableDraft[]
  displayName: string
  introduction: string
  playerSchemaId: string
  resourceId: string
  worldSchemaId: string
}

function createInitialFormState(resource?: StoryResource): FormState {
  return {
    commonVariables: createStoryCommonVariableDrafts([]),
    displayName: '',
    introduction: '',
    playerSchemaId: resource?.player_schema_id_seed ?? '',
    resourceId: resource?.resource_id ?? '',
    worldSchemaId: resource?.world_schema_id_seed ?? '',
  }
}

function createInitialManualGraph(
  defaults: {
    goal: string
    scene: string
    title: string
  },
): StoryGraph {
  const graph = createDefaultStoryGraph()
  const firstNode = graph.nodes[0]

  if (firstNode) {
    firstNode.title = defaults.title
    firstNode.scene = defaults.scene
    firstNode.goal = defaults.goal
  }

  return graph
}

function summarizeStoryInput(resource: StoryResource) {
  return (resource.planned_story?.trim() || resource.story_concept).replace(/\s+/g, ' ').trim()
}

function getErrorMessage(error: unknown, fallback: string) {
  return error instanceof Error ? error.message : fallback
}

function buildSchemaOptions(
  schemas: ReadonlyArray<SchemaResource>,
  tag: 'player' | 'world',
  currentId: string,
  fallbackLabel: string,
): SelectOption[] {
  const filteredSchemas = schemas.filter((schema) => schema.tags.includes(tag))
  const baseSchemas = filteredSchemas.length > 0 ? filteredSchemas : schemas
  const options = baseSchemas.map((schema) => ({
    label: `${schema.display_name} · ${schema.schema_id}`,
    value: schema.schema_id,
  }))

  if (currentId.trim().length > 0 && !options.some((option) => option.value === currentId)) {
    return [
      {
        label: `${currentId} · ${fallbackLabel}`,
        value: currentId,
      },
      ...options,
    ]
  }

  return options
}

function Field({
  children,
  htmlFor,
  label,
}: {
  children: ReactNode
  htmlFor?: string
  label: string
}) {
  return (
    <div className="space-y-2.5">
      {htmlFor ? (
        <label
          className="block text-sm font-medium text-[var(--color-text-primary)]"
          htmlFor={htmlFor}
        >
          {label}
        </label>
      ) : (
        <span className="block text-sm font-medium text-[var(--color-text-primary)]">{label}</span>
      )}
      {children}
    </div>
  )
}

export function ManualStoryCreateDialog({
  availableCharacters,
  onCompleted,
  onOpenChange,
  open,
  resources,
}: ManualStoryCreateDialogProps) {
  const { t } = useTranslation()
  const { pushToast } = useToast()
  const fieldIdPrefix = useId()
  const [formState, setFormState] = useState<FormState>(() => createInitialFormState(resources[0]))
  const [schemas, setSchemas] = useState<SchemaResource[]>([])
  const [isSchemasLoading, setIsSchemasLoading] = useState(false)
  const [isSubmitting, setIsSubmitting] = useState(false)
  const [isGraphEditorOpen, setIsGraphEditorOpen] = useState(false)
  const [submitError, setSubmitError] = useState<string | null>(null)
  const [commonVariableErrors, setCommonVariableErrors] = useState<StoryCommonVariableDraftErrors>(
    {},
  )
  const [initialGraph, setInitialGraph] = useState<StoryGraph | null>(null)
  useToastMessage(submitError)

  const graphController = useStoryGraphEditorController({
    graph: initialGraph,
    onLastNodeWarning: () => {
      pushToast({
        message: t('stories.graph.errors.lastNode'),
        tone: 'warning',
      })
    },
    open,
  })

  const fieldIds = {
    displayName: `${fieldIdPrefix}-display-name`,
    introduction: `${fieldIdPrefix}-introduction`,
    playerSchemaId: `${fieldIdPrefix}-player-schema-id`,
    resourceId: `${fieldIdPrefix}-resource-id`,
    worldSchemaId: `${fieldIdPrefix}-world-schema-id`,
  } as const

  const selectedResource = useMemo(
    () => resources.find((resource) => resource.resource_id === formState.resourceId) ?? null,
    [formState.resourceId, resources],
  )
  const resourceCharacterIds = useMemo(() => selectedResource?.character_ids ?? [], [selectedResource])
  const commonVariableCharacterIds = useMemo(() => {
    const knownCharacterIds = new Set(resourceCharacterIds)

    formState.commonVariables.forEach((draft) => {
      if (draft.scope !== 'character') {
        return
      }

      const characterId = draft.character_id.trim()

      if (characterId.length > 0) {
        knownCharacterIds.add(characterId)
      }
    })

    return Array.from(knownCharacterIds)
  }, [formState.commonVariables, resourceCharacterIds])
  const commonVariableSchemaCatalog = useStoryCommonVariableSchemaCatalog({
    characterIds: commonVariableCharacterIds,
    enabled: open && Boolean(selectedResource),
    playerSchemaId: formState.playerSchemaId,
    worldSchemaId: formState.worldSchemaId,
  })

  const resourceOptions = useMemo(
    () =>
      resources.map((resource) => ({
        label: resource.resource_id,
        value: resource.resource_id,
      })),
    [resources],
  )
  const playerSchemaOptions = useMemo(
    () =>
      buildSchemaOptions(
        schemas,
        'player',
        formState.playerSchemaId,
        t('stories.manualCreate.currentSchema'),
      ),
    [formState.playerSchemaId, schemas, t],
  )
  const worldSchemaOptions = useMemo(
    () =>
      buildSchemaOptions(
        schemas,
        'world',
        formState.worldSchemaId,
        t('stories.manualCreate.currentSchema'),
      ),
    [formState.worldSchemaId, schemas, t],
  )

  const hasResources = resources.length > 0
  const hasSchemas = schemas.length > 0
  const graphSummary = useMemo(() => {
    const graphDraft = graphController.graphDraft

    if (!graphDraft) {
      return null
    }

    return {
      nodeCount: graphDraft.nodes.length,
      startNode: graphDraft.start_node,
      terminalCount: graphDraft.nodes.filter((node) => node.transitions.length === 0).length,
    }
  }, [graphController.graphDraft])

  useEffect(() => {
    if (!open) {
      setFormState(createInitialFormState(resources[0]))
      setSchemas([])
      setIsSchemasLoading(false)
      setIsSubmitting(false)
      setIsGraphEditorOpen(false)
      setSubmitError(null)
      setCommonVariableErrors({})
      setInitialGraph(null)
      return
    }

    const controller = new AbortController()
    const initialResource = resources[0]

    setFormState(createInitialFormState(initialResource))
    setIsSchemasLoading(true)
    setIsSubmitting(false)
    setIsGraphEditorOpen(false)
    setSubmitError(null)
    setCommonVariableErrors({})
    setInitialGraph(
      createInitialManualGraph({
        goal: t('stories.manualCreate.defaults.goal'),
        scene: t('stories.manualCreate.defaults.scene'),
        title: t('stories.manualCreate.defaults.title'),
      }),
    )

    void listSchemas(controller.signal)
      .then((nextSchemas) => {
        if (!controller.signal.aborted) {
          setSchemas(nextSchemas)
        }
      })
      .catch((error) => {
        if (!controller.signal.aborted) {
          setSubmitError(
            getErrorMessage(error, t('stories.manualCreate.errors.loadSchemasFailed')),
          )
        }
      })
      .finally(() => {
        if (!controller.signal.aborted) {
          setIsSchemasLoading(false)
        }
      })

    return () => {
      controller.abort()
    }
  }, [open, resources, t])

  function validateForm() {
    if (formState.resourceId.trim().length === 0) {
      return t('stories.form.errors.resourceRequired')
    }

    if (formState.playerSchemaId.trim().length === 0) {
      return t('stories.manualCreate.errors.playerSchemaRequired')
    }

    if (formState.worldSchemaId.trim().length === 0) {
      return t('stories.manualCreate.errors.worldSchemaRequired')
    }

    if (formState.introduction.trim().length === 0) {
      return t('stories.manualCreate.errors.introductionRequired')
    }

    const nextCommonVariableErrors = validateStoryCommonVariableDrafts(
      formState.commonVariables,
      new Set(resourceCharacterIds),
    )

    setCommonVariableErrors(nextCommonVariableErrors)

    if (Object.keys(nextCommonVariableErrors).length > 0) {
      return t('stories.form.errors.commonVariablesInvalid')
    }

    return null
  }

  function handleResourceChange(resourceId: string) {
    const nextResource = resources.find((resource) => resource.resource_id === resourceId)

    setCommonVariableErrors({})
    setFormState((currentFormState) => ({
      ...currentFormState,
      playerSchemaId: nextResource?.player_schema_id_seed?.trim()
        ? nextResource.player_schema_id_seed
        : currentFormState.playerSchemaId,
      resourceId,
      worldSchemaId: nextResource?.world_schema_id_seed?.trim()
        ? nextResource.world_schema_id_seed
        : currentFormState.worldSchemaId,
    }))
  }

  async function handleSubmit() {
    const validationError = validateForm()

    if (validationError) {
      setSubmitError(validationError)
      return
    }

    const normalizedGraph = normalizeControlledStoryGraph(graphController)

    if (!normalizedGraph.graph) {
      pushToast({
        message: getStoryGraphValidationMessage(
          (key, options) => t(key as never, options),
          normalizedGraph.errors[0] ?? 'invalid_graph',
        ),
        tone: 'error',
      })
      return
    }

    setSubmitError(null)
    setIsSubmitting(true)

    try {
      const result = await createStory({
        common_variables: serializeStoryCommonVariableDrafts(formState.commonVariables),
        ...(formState.displayName.trim()
          ? { display_name: formState.displayName.trim() }
          : {}),
        graph: normalizedGraph.graph,
        introduction: formState.introduction.trim(),
        player_schema_id: formState.playerSchemaId.trim(),
        resource_id: formState.resourceId.trim(),
        world_schema_id: formState.worldSchemaId.trim(),
      })

      await onCompleted({
        message: t('stories.feedback.created', { name: result.display_name }),
        story: {
          ...result,
          type: 'story',
        },
      })

      onOpenChange(false)
    } catch (error) {
      setSubmitError(getErrorMessage(error, t('stories.manualCreate.errors.submitFailed')))
    } finally {
      setIsSubmitting(false)
    }
  }

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent
        aria-describedby={undefined}
        className="w-[min(96vw,64rem)]"
        onEscapeKeyDown={(event) => {
          if (isSubmitting) {
            event.preventDefault()
          }
        }}
        onInteractOutside={(event) => {
          if (isSubmitting) {
            event.preventDefault()
          }
        }}
      >
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <DialogTitle>{t('stories.manualCreate.title')}</DialogTitle>
          <p className="text-sm leading-7 text-[var(--color-text-secondary)]">
            {t('stories.manualCreate.description')}
          </p>
        </DialogHeader>

        <DialogBody className="max-h-[calc(92vh-12rem)] overflow-y-auto pt-6">
          {!hasResources ? (
            <div className="space-y-5">
              <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4 text-sm leading-7 text-[var(--color-text-secondary)]">
                {t('stories.form.emptyResources')}
              </div>

              <div className="flex justify-end">
                <DialogRouteButton
                  onRequestClose={() => {
                    onOpenChange(false)
                  }}
                  to={appPaths.storyResources}
                  variant="secondary"
                >
                  {t('stories.form.openResources')}
                </DialogRouteButton>
              </div>
            </div>
          ) : isSchemasLoading ? (
            <div className="space-y-4">
              <div className="h-12 animate-pulse rounded-2xl bg-[var(--color-bg-elevated)]" />
              <div className="h-12 animate-pulse rounded-2xl bg-[var(--color-bg-elevated)]" />
              <div className="h-32 animate-pulse rounded-[1.45rem] bg-[var(--color-bg-elevated)]" />
              <div className="h-16 animate-pulse rounded-[1.45rem] bg-[var(--color-bg-elevated)]" />
              <div className="h-56 animate-pulse rounded-[1.55rem] bg-[var(--color-bg-elevated)]" />
            </div>
          ) : !hasSchemas ? (
            <div className="space-y-5">
              <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4 text-sm leading-7 text-[var(--color-text-secondary)]">
                {t('stories.manualCreate.emptySchemas')}
              </div>

              <div className="flex justify-end">
                <DialogRouteButton
                  onRequestClose={() => {
                    onOpenChange(false)
                  }}
                  to={appPaths.schemas}
                  variant="secondary"
                >
                  {t('stories.manualCreate.openSchemas')}
                </DialogRouteButton>
              </div>
            </div>
          ) : (
            <div className="space-y-6">
              <Field htmlFor={fieldIds.resourceId} label={t('stories.form.fields.resourceId')}>
                <Select
                  items={resourceOptions}
                  textAlign="start"
                  triggerId={fieldIds.resourceId}
                  value={formState.resourceId}
                  onValueChange={handleResourceChange}
                />
              </Field>

              <div className="grid gap-4 md:grid-cols-2">
                <Field
                  htmlFor={fieldIds.playerSchemaId}
                  label={t('stories.form.fields.playerSchemaId')}
                >
                  <Select
                    items={playerSchemaOptions}
                    placeholder={t('stories.manualCreate.placeholders.playerSchemaId')}
                    textAlign="start"
                    triggerId={fieldIds.playerSchemaId}
                    value={formState.playerSchemaId || undefined}
                    onValueChange={(playerSchemaId) => {
                      setFormState((currentFormState) => ({
                        ...currentFormState,
                        playerSchemaId,
                      }))
                    }}
                  />
                </Field>

                <Field
                  htmlFor={fieldIds.worldSchemaId}
                  label={t('stories.form.fields.worldSchemaId')}
                >
                  <Select
                    items={worldSchemaOptions}
                    placeholder={t('stories.manualCreate.placeholders.worldSchemaId')}
                    textAlign="start"
                    triggerId={fieldIds.worldSchemaId}
                    value={formState.worldSchemaId || undefined}
                    onValueChange={(worldSchemaId) => {
                      setFormState((currentFormState) => ({
                        ...currentFormState,
                        worldSchemaId,
                      }))
                    }}
                  />
                </Field>
              </div>

              <div className="space-y-4">
                {selectedResource ? (
                  <div className="rounded-[1.4rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
                    <p className="text-xs text-[var(--color-text-muted)]">
                      {t('stories.form.fields.inputPreview')}
                    </p>
                    <p className="mt-2 text-sm leading-7 text-[var(--color-text-primary)]">
                      {summarizeStoryInput(selectedResource)}
                    </p>
                  </div>
                ) : null}

                <div className="rounded-[1.4rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
                  <div className="flex flex-wrap items-start justify-between gap-3">
                    <div className="min-w-0">
                      <p className="text-xs text-[var(--color-text-muted)]">
                        {t('stories.manualCreate.graphCard.title')}
                      </p>
                      <p className="mt-2 text-sm leading-7 text-[var(--color-text-secondary)]">
                        {graphController.isDirty
                          ? t('stories.manualCreate.graphCard.edited')
                          : t('stories.manualCreate.graphCard.default')}
                      </p>
                    </div>
                    <Button
                      className="shrink-0"
                      disabled={isSubmitting || !graphController.graphDraft}
                      onClick={() => {
                        setIsGraphEditorOpen(true)
                      }}
                      size="sm"
                      variant="secondary"
                    >
                      {t('stories.actions.editGraph')}
                    </Button>
                  </div>
                  {graphSummary ? (
                    <div className="mt-3 flex flex-wrap gap-2">
                      <Badge className="normal-case px-3 py-1.5" variant="info">
                        {t('stories.details.nodeCount', { count: graphSummary.nodeCount })}
                      </Badge>
                      <Badge className="normal-case px-3 py-1.5" variant="subtle">
                        {t('stories.details.startNode', { id: graphSummary.startNode })}
                      </Badge>
                      <Badge className="normal-case px-3 py-1.5" variant="subtle">
                        {t('stories.details.terminalCount', { count: graphSummary.terminalCount })}
                      </Badge>
                    </div>
                  ) : null}
                </div>
              </div>

              <Field htmlFor={fieldIds.displayName} label={t('stories.form.fields.displayName')}>
                <Input
                  id={fieldIds.displayName}
                  name={fieldIds.displayName}
                  placeholder={t('stories.manualCreate.placeholders.displayName')}
                  value={formState.displayName}
                  onChange={(event) => {
                    setFormState((currentFormState) => ({
                      ...currentFormState,
                      displayName: event.target.value,
                    }))
                  }}
                />
              </Field>

              <Field htmlFor={fieldIds.introduction} label={t('stories.form.fields.introduction')}>
                <Textarea
                  id={fieldIds.introduction}
                  name={fieldIds.introduction}
                  placeholder={t('stories.manualCreate.placeholders.introduction')}
                  rows={5}
                  value={formState.introduction}
                  onChange={(event) => {
                    setFormState((currentFormState) => ({
                      ...currentFormState,
                      introduction: event.target.value,
                    }))
                  }}
                />
              </Field>

              <StoryCommonVariablesEditor
                characters={availableCharacters}
                disabled={isSubmitting}
                drafts={formState.commonVariables}
                errors={commonVariableErrors}
                resourceCharacterIds={resourceCharacterIds}
                schemaCatalog={commonVariableSchemaCatalog}
                onChange={(commonVariables) => {
                  setCommonVariableErrors({})
                  setFormState((currentFormState) => ({
                    ...currentFormState,
                    commonVariables,
                  }))
                }}
              />
            </div>
          )}
        </DialogBody>

        <DialogFooter className="justify-between">
          <div>
            <DialogClose asChild>
              <Button disabled={isSubmitting} variant="secondary">
                {t('stories.actions.cancel')}
              </Button>
            </DialogClose>
          </div>
          <div className="flex flex-wrap items-center justify-end gap-3">
            {hasResources && hasSchemas ? (
              <Button disabled={isSubmitting} onClick={() => void handleSubmit()}>
                {isSubmitting ? t('stories.actions.creating') : t('stories.manualCreate.create')}
              </Button>
            ) : null}
          </div>
        </DialogFooter>
      </DialogContent>

      <ManualStoryGraphDialog
        controller={graphController}
        onOpenChange={setIsGraphEditorOpen}
        open={open && isGraphEditorOpen}
        playerSchemaId={formState.playerSchemaId}
        resourceId={formState.resourceId}
        worldSchemaId={formState.worldSchemaId}
      />
    </Dialog>
  )
}
