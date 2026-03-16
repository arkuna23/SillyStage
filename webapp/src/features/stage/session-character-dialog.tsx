import { useMemo, useState } from 'react'

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
import { Input } from '../../components/ui/input'
import { Textarea } from '../../components/ui/textarea'
import type { StageCopy } from './copy'
import type { SessionCharacter } from './types'

type SessionCharacterDialogProps = {
  character: SessionCharacter | null
  copy: StageCopy
  onDelete: (sessionCharacterId: string) => Promise<void> | void
  onOpenChange: (open: boolean) => void
  onSave: (character: SessionCharacter) => Promise<void> | void
  onToggleScene: (sessionCharacterId: string, inScene: boolean) => Promise<void> | void
  open: boolean
}

type FormState = {
  displayName: string
  personality: string
  style: string
  systemPrompt: string
}

type SessionCharacterDialogFormProps = {
  character: SessionCharacter
  copy: StageCopy
  onDelete: (sessionCharacterId: string) => Promise<void> | void
  onSave: (character: SessionCharacter) => Promise<void> | void
  onToggleScene: (sessionCharacterId: string, inScene: boolean) => Promise<void> | void
}

function createInitialFormState(character: SessionCharacter | null): FormState {
  return {
    displayName: character?.display_name ?? '',
    personality: character?.personality ?? '',
    style: character?.style ?? '',
    systemPrompt: character?.system_prompt ?? '',
  }
}

function SessionCharacterDialogForm({
  character,
  copy,
  onDelete,
  onSave,
  onToggleScene,
}: SessionCharacterDialogFormProps) {
  const [formState, setFormState] = useState<FormState>(() => createInitialFormState(character))
  const normalizedCharacter = useMemo(() => {
    return {
      ...character,
      display_name: formState.displayName.trim(),
      personality: formState.personality.trim(),
      style: formState.style.trim(),
      system_prompt: formState.systemPrompt.trim(),
    }
  }, [character, formState])

  return (
    <>
      <DialogHeader className="border-b border-[var(--color-border-subtle)]">
        <p className="font-mono text-xs text-[var(--color-text-muted)]">
          {character.session_character_id}
        </p>
        <DialogTitle>{copy.sessionCharacterDialog.title}</DialogTitle>
      </DialogHeader>

      <DialogBody className="max-h-[calc(92vh-13rem)] overflow-y-auto pt-6">
        <div className="space-y-5">
          <div className="grid gap-4 md:grid-cols-2">
            <label className="space-y-2.5">
              <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                {copy.sessionCharacterDialog.displayName}
              </span>
              <Input
                id="stage-session-character-display-name"
                name="stage-session-character-display-name"
                onChange={(event) => {
                  setFormState((current) => ({
                    ...current,
                    displayName: event.target.value,
                  }))
                }}
                value={formState.displayName}
              />
            </label>

            <div className="space-y-2.5 rounded-[1.2rem] border border-[var(--color-border-subtle)] bg-[var(--color-bg-elevated)] px-4 py-3.5">
              <span className="block text-sm font-medium text-[var(--color-text-primary)]">
                {copy.sessionCharacterDialog.inScene}
              </span>
              <p className="text-sm text-[var(--color-text-secondary)]">
                {character.in_scene
                  ? copy.settings.sessionCharacters.inScene
                  : copy.settings.sessionCharacters.outOfScene}
              </p>
            </div>
          </div>

          <label className="space-y-2.5">
            <span className="block text-sm font-medium text-[var(--color-text-primary)]">
              {copy.sessionCharacterDialog.personality}
            </span>
            <Textarea
              className="min-h-[6rem]"
              id="stage-session-character-personality"
              name="stage-session-character-personality"
              onChange={(event) => {
                setFormState((current) => ({
                  ...current,
                  personality: event.target.value,
                }))
              }}
              value={formState.personality}
            />
          </label>

          <label className="space-y-2.5">
            <span className="block text-sm font-medium text-[var(--color-text-primary)]">
              {copy.sessionCharacterDialog.style}
            </span>
            <Textarea
              className="min-h-[6rem]"
              id="stage-session-character-style"
              name="stage-session-character-style"
              onChange={(event) => {
                setFormState((current) => ({
                  ...current,
                  style: event.target.value,
                }))
              }}
              value={formState.style}
            />
          </label>

          <label className="space-y-2.5">
            <span className="block text-sm font-medium text-[var(--color-text-primary)]">
              {copy.sessionCharacterDialog.sectionPrompt}
            </span>
            <Textarea
              className="min-h-[8rem]"
              id="stage-session-character-system-prompt"
              name="stage-session-character-system-prompt"
              onChange={(event) => {
                setFormState((current) => ({
                  ...current,
                  systemPrompt: event.target.value,
                }))
              }}
              value={formState.systemPrompt}
            />
          </label>
        </div>
      </DialogBody>

      <DialogFooter className="sm:items-center">
        <DialogClose asChild>
          <Button variant="ghost">关闭</Button>
        </DialogClose>

        <div className="flex flex-wrap items-center justify-end gap-3 sm:ml-auto">
          <Button
            onClick={() => {
              void onToggleScene(character.session_character_id, !character.in_scene)
            }}
            variant="secondary"
          >
            {character.in_scene
              ? copy.sessionCharacterDialog.leaveScene
              : copy.sessionCharacterDialog.enterScene}
          </Button>
          <Button
            onClick={() => {
              void onSave(normalizedCharacter)
            }}
          >
            {copy.sessionCharacterDialog.save}
          </Button>
          <Button
            onClick={() => {
              void onDelete(character.session_character_id)
            }}
            variant="danger"
          >
            {copy.sessionCharacterDialog.delete}
          </Button>
        </div>
      </DialogFooter>
    </>
  )
}

export function SessionCharacterDialog({
  character,
  copy,
  onDelete,
  onOpenChange,
  onSave,
  onToggleScene,
  open,
}: SessionCharacterDialogProps) {
  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent aria-describedby={undefined} className="w-[min(92vw,44rem)] overflow-hidden">
        {character ? (
          <SessionCharacterDialogForm
            character={character}
            copy={copy}
            key={`${character.session_character_id}:${open ? 'open' : 'closed'}`}
            onDelete={onDelete}
            onSave={onSave}
            onToggleScene={onToggleScene}
          />
        ) : null}
      </DialogContent>
    </Dialog>
  )
}
