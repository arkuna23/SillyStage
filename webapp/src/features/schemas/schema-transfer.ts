import type { SchemaResource } from './types'

export type SchemaBundle = {
  schemas: SchemaResource[]
  type: 'schema_bundle'
  version: 1
}

function isObject(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null
}

function isStringArray(value: unknown): value is string[] {
  return Array.isArray(value) && value.every((item) => typeof item === 'string')
}

function isSchemaResource(value: unknown): value is SchemaResource {
  if (!isObject(value) || value.type !== 'schema') {
    return false
  }

  return (
    typeof value.schema_id === 'string' &&
    typeof value.display_name === 'string' &&
    isObject(value.fields) &&
    isStringArray(value.tags)
  )
}

export function createSchemaBundle(schemas: ReadonlyArray<SchemaResource>): SchemaBundle {
  return {
    schemas: [...schemas],
    type: 'schema_bundle',
    version: 1,
  }
}

export function isSchemaBundle(value: unknown): value is SchemaBundle {
  if (!isObject(value) || value.type !== 'schema_bundle' || value.version !== 1) {
    return false
  }

  return Array.isArray(value.schemas) && value.schemas.every(isSchemaResource)
}
