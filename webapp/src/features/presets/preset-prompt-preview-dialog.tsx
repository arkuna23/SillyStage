import { useCallback, useMemo } from 'react'
import { useTranslation } from 'react-i18next'

import { Badge } from '../../components/ui/badge'
import { Button } from '../../components/ui/button'
import {
  Dialog,
  DialogBody,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '../../components/ui/dialog'
import type { AgentRoleKey, PresetDetail } from '../apis/types'
import { createPresetRoleLabels } from './preset-labels'
import { PresetPromptPreviewPanel } from './preset-prompt-preview-panel'

type PresetPromptPreviewDialogProps = {
  initialAgent: AgentRoleKey
  initialModuleId?: string | null
  onOpenChange: (open: boolean) => void
  open: boolean
  preset: PresetDetail
  scopeLabel?: string
}

export function PresetPromptPreviewDialog({
  initialAgent,
  initialModuleId,
  onOpenChange,
  open,
  preset,
  scopeLabel,
}: PresetPromptPreviewDialogProps) {
  const { t } = useTranslation()
  const translate = useCallback(
    (key: string, options?: Record<string, unknown>) => String(t(key as never, options as never)),
    [t],
  )
  const roleLabels = useMemo(() => createPresetRoleLabels(translate), [translate])

  return (
    <Dialog onOpenChange={onOpenChange} open={open}>
      <DialogContent contentClassName="w-[min(92vw,46rem)]">
        <DialogHeader className="border-b border-[var(--color-border-subtle)]">
          <div className="w-full space-y-3">
            <DialogTitle>{t('presetsPage.preview.title')}</DialogTitle>
            <div className="flex flex-wrap items-center gap-2">
              <Badge className="normal-case" variant="subtle">
                {roleLabels[initialAgent]}
              </Badge>
              {scopeLabel && initialModuleId ? (
                <Badge className="normal-case" variant="gold">
                  {scopeLabel}
                </Badge>
              ) : null}
            </div>
            <DialogDescription>{t('presetsPage.preview.description')}</DialogDescription>
          </div>
        </DialogHeader>

        <DialogBody className="pt-6">
          <div className="w-full">
            <PresetPromptPreviewPanel
              initialAgent={initialAgent}
              initialModuleId={initialModuleId}
              key={`${preset.preset_id}:${initialAgent}:${initialModuleId ?? '__all__'}`}
              preset={preset}
              t={translate}
            />
          </div>
        </DialogBody>

        <DialogFooter className="justify-start">
          <DialogClose asChild>
            <Button variant="secondary">{t('presetsPage.preview.actions.close')}</Button>
          </DialogClose>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
