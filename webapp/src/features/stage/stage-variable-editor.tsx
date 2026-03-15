import { faPlus } from '@fortawesome/free-solid-svg-icons/faPlus'
import { faTrashCan } from '@fortawesome/free-solid-svg-icons/faTrashCan'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'

import { Button } from '../../components/ui/button'
import { IconButton } from '../../components/ui/icon-button'
import { Input } from '../../components/ui/input'
import { Textarea } from '../../components/ui/textarea'
import type { VariableRowDraft } from './stage-variable-utils'

export function VariableRows({
  addLabel,
  errors,
  fieldIdPrefix,
  jsonLabel,
  keyLabel,
  onAddRow,
  onChangeKey,
  onChangeValue,
  onRemoveRow,
  rows,
}: {
  addLabel: string
  errors: Record<string, string>
  fieldIdPrefix: string
  jsonLabel: string
  keyLabel: string
  onAddRow: () => void
  onChangeKey: (rowId: string, value: string) => void
  onChangeValue: (rowId: string, value: string) => void
  onRemoveRow: (rowId: string) => void
  rows: VariableRowDraft[]
}) {
  return (
    <div className="space-y-3">
      <div className="hidden grid-cols-[12rem,minmax(0,1fr),auto] gap-3 px-1 text-xs uppercase tracking-[0.12em] text-[var(--color-text-muted)] md:grid">
        <span>{keyLabel}</span>
        <span>{jsonLabel}</span>
        <span className="sr-only">Actions</span>
      </div>

      <div className="space-y-3">
        {rows.map((row) => {
          const keyFieldId = `${fieldIdPrefix}-${row.id}-key`
          const valueFieldId = `${fieldIdPrefix}-${row.id}-value`

          return (
            <div className="space-y-2" key={row.id}>
              <div className="grid gap-3 md:grid-cols-[12rem,minmax(0,1fr),auto] md:items-start">
                <Input
                  aria-label={keyLabel}
                  id={keyFieldId}
                  name={keyFieldId}
                  onChange={(event) => {
                    onChangeKey(row.id, event.target.value)
                  }}
                  placeholder={keyLabel}
                  value={row.key}
                />
                <Textarea
                  aria-label={jsonLabel}
                  className="min-h-[3.25rem] font-mono text-xs leading-6"
                  id={valueFieldId}
                  name={valueFieldId}
                  onChange={(event) => {
                    onChangeValue(row.id, event.target.value)
                  }}
                  placeholder="null"
                  value={row.valueText}
                />
                <div className="flex justify-end md:pt-1">
                  <IconButton
                    icon={<FontAwesomeIcon icon={faTrashCan} />}
                    label="Delete variable"
                    onClick={() => {
                      onRemoveRow(row.id)
                    }}
                    size="sm"
                    variant="ghost"
                  />
                </div>
              </div>
              {errors[row.id] ? (
                <p className="text-sm text-[var(--color-state-error)]">{errors[row.id]}</p>
              ) : null}
            </div>
          )
        })}
      </div>

      <Button onClick={onAddRow} size="sm" variant="secondary">
        <FontAwesomeIcon className="mr-2 text-xs" icon={faPlus} />
        {addLabel}
      </Button>
    </div>
  )
}
