export const stateValueTypes = [
  'bool',
  'int',
  'float',
  'string',
  'array',
  'object',
  'null',
] as const

export type StateValueType = (typeof stateValueTypes)[number]

export type JsonValue =
  | boolean
  | null
  | number
  | string
  | JsonValue[]
  | { [key: string]: JsonValue }

export type StateFieldSchema = {
  default?: JsonValue
  description?: string | null
  value_type: StateValueType
}
