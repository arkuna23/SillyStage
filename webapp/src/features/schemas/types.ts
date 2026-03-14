import type { StateFieldSchema } from '../../lib/state-schema'

export type SchemaResource = {
  display_name: string
  fields: Record<string, StateFieldSchema>
  schema_id: string
  tags: string[]
  type: 'schema'
}

export type SchemasListedResult = {
  schemas: SchemaResource[]
  type: 'schemas_listed'
}

export type SchemaDeletedResult = {
  schema_id: string
  type: 'schema_deleted'
}
