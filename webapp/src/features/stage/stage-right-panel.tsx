import { faDatabase } from '@fortawesome/free-solid-svg-icons/faDatabase'
import { faSliders } from '@fortawesome/free-solid-svg-icons/faSliders'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { AnimatePresence, LayoutGroup, motion } from 'framer-motion'
import { WorkspacePanelShell } from '../../components/layout/workspace-panel-shell'
import { Badge } from '../../components/ui/badge'
import { Button } from '../../components/ui/button'
import { Card, CardContent } from '../../components/ui/card'
import { SegmentedSelector } from '../../components/ui/segmented-selector'
import { cn } from '../../lib/cn'
import type { StageCopy } from './copy'
import { CharacterAvatar, RightPanelSection, StagePanelHeader } from './stage-panel-shared'
import type { StageCastMember, StageCommonVariable, StageRightRailTab } from './stage-ui-types'
import type { RuntimeSnapshot } from './types'

const panelEase = [0.16, 1, 0.3, 1] as const

type StoryNodeSummary = {
  goal?: string | null
  scene?: string | null
}

type StageRightPanelProps = {
  activeCast: ReadonlyArray<StageCastMember>
  activeSpeakerId: string | null
  commonVariables: ReadonlyArray<StageCommonVariable>
  copy: StageCopy
  currentNode: StoryNodeSummary | null
  currentSnapshot: RuntimeSnapshot | null
  hasExpandableNodeDetails: boolean
  isStoryIntroExpanded: boolean
  isStoryNodeExpanded: boolean
  onChangeRailTab: (tab: StageRightRailTab) => void
  onOpenCharacter: (characterId: string) => void
  onOpenSessionCharacter: (sessionCharacterId: string) => void
  onToggleStoryIntro: () => void
  onToggleStoryNode: () => void
  prefersReducedMotion: boolean | null
  railTab: StageRightRailTab
  storyIntroNeedsExpand: boolean
  visibleStoryIntroduction: string
}

function CastPanelContent({
  activeCast,
  activeSpeakerId,
  copy,
  onOpenCharacter,
  onOpenSessionCharacter,
  prefersReducedMotion,
}: {
  activeCast: ReadonlyArray<StageCastMember>
  activeSpeakerId: string | null
  copy: StageCopy
  onOpenCharacter: (characterId: string) => void
  onOpenSessionCharacter: (sessionCharacterId: string) => void
  prefersReducedMotion: boolean | null
}) {
  if (activeCast.length === 0) {
    return (
      <div className="rounded-[1.35rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_72%,transparent)] px-4 py-4 text-sm leading-7 text-[var(--color-text-secondary)]">
        {copy.cast.empty}
      </div>
    )
  }

  return (
    <LayoutGroup id="stage-cast">
      <div className="space-y-3">
        <AnimatePresence initial={false}>
          {activeCast.map((character) => {
            const isActive = character.id === activeSpeakerId

            return (
              <motion.button
                animate={{ opacity: 1, y: 0 }}
                className={cn(
                  'w-full rounded-[1.2rem] border px-3.5 py-3 text-left',
                  isActive
                    ? 'border-[var(--color-accent-gold-line)] bg-[var(--color-accent-gold-soft)] shadow-[0_16px_38px_var(--color-accent-glow-soft)]'
                    : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)]',
                )}
                exit={prefersReducedMotion ? { opacity: 1 } : { opacity: 0, y: -12 }}
                initial={prefersReducedMotion ? { opacity: 1, y: 0 } : { opacity: 0, y: 18 }}
                key={character.id}
                layout
                onClick={() => {
                  if (character.isSessionCharacter) {
                    onOpenSessionCharacter(character.id)
                    return
                  }

                  onOpenCharacter(character.id)
                }}
                transition={
                  prefersReducedMotion ? { duration: 0 } : { duration: 0.24, ease: panelEase }
                }
                type="button"
              >
                <div className="flex items-start gap-2.5">
                  <CharacterAvatar coverUrl={character.coverUrl} name={character.name} />
                  <div className="min-w-0 flex-1 space-y-1">
                    <div className="flex items-center gap-2">
                      <p className="truncate text-sm font-medium text-[var(--color-text-primary)]">
                        {character.name}
                      </p>
                      {character.isSessionCharacter ? (
                        <Badge variant="subtle">{copy.cast.sessionCharacterBadge}</Badge>
                      ) : null}
                      {isActive ? <Badge variant="subtle">{copy.cast.active}</Badge> : null}
                    </div>
                    <p className="truncate font-mono text-[0.68rem] text-[var(--color-text-muted)]">
                      {character.id}
                    </p>
                  </div>
                </div>
              </motion.button>
            )
          })}
        </AnimatePresence>
      </div>
    </LayoutGroup>
  )
}

function VariablesPanelContent({
  commonVariables,
  copy,
  currentNode,
}: {
  commonVariables: ReadonlyArray<StageCommonVariable>
  copy: StageCopy
  currentNode: StoryNodeSummary | null
}) {
  return (
    <div className="space-y-3">
      <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
        <p className="text-xs text-[var(--color-text-muted)]">{copy.storyNode.goal}</p>
        <p className="mt-2 text-sm leading-6 text-[var(--color-text-primary)]">
          {currentNode?.goal ?? '—'}
        </p>
      </div>

      {commonVariables.length === 0 ? (
        <div className="rounded-[1.35rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_72%,transparent)] px-4 py-4 text-sm leading-7 text-[var(--color-text-secondary)]">
          {copy.commonVariables.empty}
        </div>
      ) : null}

      {commonVariables.map((variable) => (
        <div
          className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4"
          key={variable.id}
        >
          <p className="text-xs text-[var(--color-text-muted)]">{variable.label}</p>
          <p className="mt-2 break-words text-sm leading-6 text-[var(--color-text-primary)]">
            {variable.value}
          </p>
        </div>
      ))}
    </div>
  )
}

function StatusPanelContent({
  copy,
  currentNode,
  currentSnapshot,
  isStoryNodeExpanded,
}: {
  copy: StageCopy
  currentNode: StoryNodeSummary | null
  currentSnapshot: RuntimeSnapshot | null
  isStoryNodeExpanded: boolean
}) {
  if (!isStoryNodeExpanded) {
    return null
  }

  return (
    <div className="space-y-3">
      <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
        <p className="text-xs text-[var(--color-text-muted)]">{copy.storyNode.nodeId}</p>
        <p className="mt-2 font-mono text-sm text-[var(--color-text-primary)]">
          {currentSnapshot?.world_state.current_node ?? '—'}
        </p>
      </div>
      <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
        <p className="text-xs text-[var(--color-text-muted)]">{copy.storyNode.scene}</p>
        <p className="mt-2 text-sm leading-6 text-[var(--color-text-primary)]">
          {currentNode?.scene ?? '—'}
        </p>
      </div>
    </div>
  )
}

function IntroPanelContent({ visibleStoryIntroduction }: { visibleStoryIntroduction: string }) {
  return (
    <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4 text-sm leading-6 text-[var(--color-text-secondary)]">
      {visibleStoryIntroduction}
    </div>
  )
}

export function StageRightPanel({
  activeCast,
  activeSpeakerId,
  commonVariables,
  copy,
  currentNode,
  currentSnapshot,
  hasExpandableNodeDetails,
  isStoryIntroExpanded,
  isStoryNodeExpanded,
  onChangeRailTab,
  onOpenCharacter,
  onOpenSessionCharacter,
  onToggleStoryIntro,
  onToggleStoryNode,
  prefersReducedMotion,
  railTab,
  storyIntroNeedsExpand,
  visibleStoryIntroduction,
}: StageRightPanelProps) {
  return (
    <WorkspacePanelShell className="h-full min-h-0">
      <Card className="flex h-full min-h-0 flex-col overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_92%,transparent)] shadow-none">
        <StagePanelHeader
          actions={
            <SegmentedSelector
              ariaLabel={copy.stage.title}
              className="shrink-0 [&_button]:min-w-[2.7rem] [&_button]:px-0 [&_button_span]:gap-0 xl:[&_button]:min-w-[2.7rem] xl:[&_button]:px-0"
              items={[
                {
                  ariaLabel: copy.rail.variables,
                  icon: <FontAwesomeIcon icon={faDatabase} />,
                  label: <span className="sr-only">{copy.commonVariables.section}</span>,
                  value: 'variables',
                },
                {
                  ariaLabel: copy.rail.status,
                  icon: <FontAwesomeIcon icon={faSliders} />,
                  label: <span className="sr-only">{copy.storyNode.section}</span>,
                  value: 'status',
                },
              ]}
              onValueChange={(value) => {
                onChangeRailTab(value as StageRightRailTab)
              }}
              value={railTab}
            />
          }
          title={copy.stage.title}
          titleClassName="text-[1.5rem]"
        />

        <CardContent className="min-h-0 flex-1 overflow-y-auto pt-6">
          <RightPanelSection
            title={railTab === 'variables' ? copy.commonVariables.section : copy.storyNode.section}
          >
            <AnimatePresence initial={false} mode="wait">
              <motion.div
                animate={{ opacity: 1, x: 0 }}
                className="space-y-3"
                exit={
                  prefersReducedMotion
                    ? { opacity: 1 }
                    : { opacity: 0, x: railTab === 'variables' ? 10 : -10 }
                }
                initial={
                  prefersReducedMotion
                    ? { opacity: 1, x: 0 }
                    : { opacity: 0, x: railTab === 'variables' ? -10 : 10 }
                }
                key={`stage-rail-${railTab}`}
                transition={
                  prefersReducedMotion ? { duration: 0 } : { duration: 0.22, ease: panelEase }
                }
              >
                {railTab === 'variables' ? (
                  <VariablesPanelContent
                    commonVariables={commonVariables}
                    copy={copy}
                    currentNode={currentNode}
                  />
                ) : (
                  <div className="space-y-6">
                    <RightPanelSection title={copy.cast.section}>
                      <CastPanelContent
                        activeCast={activeCast}
                        activeSpeakerId={activeSpeakerId}
                        copy={copy}
                        onOpenCharacter={onOpenCharacter}
                        onOpenSessionCharacter={onOpenSessionCharacter}
                        prefersReducedMotion={prefersReducedMotion}
                      />
                    </RightPanelSection>

                    <RightPanelSection
                      action={
                        hasExpandableNodeDetails ? (
                          <Button onClick={onToggleStoryNode} size="sm" variant="ghost">
                            {isStoryNodeExpanded ? copy.storyNode.collapse : copy.storyNode.expand}
                          </Button>
                        ) : null
                      }
                      title={copy.storyNode.section}
                    >
                      <StatusPanelContent
                        copy={copy}
                        currentNode={currentNode}
                        currentSnapshot={currentSnapshot}
                        isStoryNodeExpanded={isStoryNodeExpanded}
                      />
                    </RightPanelSection>

                    <RightPanelSection
                      action={
                        storyIntroNeedsExpand ? (
                          <Button onClick={onToggleStoryIntro} size="sm" variant="ghost">
                            {isStoryIntroExpanded ? copy.intro.collapse : copy.intro.expand}
                          </Button>
                        ) : null
                      }
                      title={copy.intro.section}
                    >
                      <IntroPanelContent visibleStoryIntroduction={visibleStoryIntroduction} />
                    </RightPanelSection>
                  </div>
                )}
              </motion.div>
            </AnimatePresence>
          </RightPanelSection>
        </CardContent>
      </Card>
    </WorkspacePanelShell>
  )
}
