import { AnimatePresence, LayoutGroup, motion } from 'framer-motion'

import { Badge } from '../../components/ui/badge'
import { Button } from '../../components/ui/button'
import { Card, CardContent } from '../../components/ui/card'
import { WorkspacePanelShell } from '../../components/layout/workspace-panel-shell'
import { cn } from '../../lib/cn'
import type { StageCopy } from './copy'
import { CharacterAvatar, RightPanelSection, StagePanelHeader } from './stage-panel-shared'
import type { StageCastMember } from './stage-ui-types'
import type { RuntimeSnapshot } from './types'

const panelEase = [0.16, 1, 0.3, 1] as const

type StoryNodeSummary = {
  goal?: string | null
  scene?: string | null
}

type StageRightPanelProps = {
  activeCast: ReadonlyArray<StageCastMember>
  activeSpeakerId: string | null
  copy: StageCopy
  currentNode: StoryNodeSummary | null
  currentSnapshot: RuntimeSnapshot | null
  hasExpandableNodeDetails: boolean
  isStoryIntroExpanded: boolean
  isStoryNodeExpanded: boolean
  onOpenCharacter: (characterId: string) => void
  onOpenSessionCharacter: (sessionCharacterId: string) => void
  onToggleStoryIntro: () => void
  onToggleStoryNode: () => void
  prefersReducedMotion: boolean | null
  storyIntroNeedsExpand: boolean
  visibleStoryIntroduction: string
}

export function StageRightPanel({
  activeCast,
  activeSpeakerId,
  copy,
  currentNode,
  currentSnapshot,
  hasExpandableNodeDetails,
  isStoryIntroExpanded,
  isStoryNodeExpanded,
  onOpenCharacter,
  onOpenSessionCharacter,
  onToggleStoryIntro,
  onToggleStoryNode,
  prefersReducedMotion,
  storyIntroNeedsExpand,
  visibleStoryIntroduction,
}: StageRightPanelProps) {
  return (
    <WorkspacePanelShell className="h-full min-h-0">
      <Card className="flex h-full min-h-0 flex-col overflow-hidden border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-panel)_92%,transparent)] shadow-none">
        <StagePanelHeader title={copy.stage.title} titleClassName="text-[1.5rem]" />

        <CardContent className="min-h-0 flex-1 overflow-y-auto pt-6">
          <div className="space-y-6 pr-1">
            <RightPanelSection title={copy.cast.section}>
              {activeCast.length === 0 ? (
                <div className="rounded-[1.35rem] border border-dashed border-[var(--color-border-subtle)] bg-[color-mix(in_srgb,var(--color-bg-elevated)_72%,transparent)] px-4 py-4 text-sm leading-7 text-[var(--color-text-secondary)]">
                  {copy.cast.empty}
                </div>
              ) : (
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
                              prefersReducedMotion
                                ? { duration: 0 }
                                : { duration: 0.24, ease: panelEase }
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
                                  {isActive ? (
                                    <Badge variant="subtle">{copy.cast.active}</Badge>
                                  ) : null}
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
              )}
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
              <div className="space-y-3">
                <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4">
                  <p className="text-xs text-[var(--color-text-muted)]">{copy.storyNode.goal}</p>
                  <p className="mt-2 text-sm leading-6 text-[var(--color-text-primary)]">
                    {currentNode?.goal ?? '—'}
                  </p>
                </div>

                {isStoryNodeExpanded ? (
                  <>
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
                  </>
                ) : null}
              </div>
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
              <div className="rounded-[1.35rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-4 text-sm leading-6 text-[var(--color-text-secondary)]">
                {visibleStoryIntroduction}
              </div>
            </RightPanelSection>
          </div>
        </CardContent>
      </Card>
    </WorkspacePanelShell>
  )
}
