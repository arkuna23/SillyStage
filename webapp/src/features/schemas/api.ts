import { rpcRequest } from '../../lib/rpc'
import type { StateFieldSchema } from '../../lib/state-schema'
import type {
  SchemaDeletedResult,
  SchemaResource,
  SchemasListedResult,
} from './types'

export async function listSchemas(signal?: AbortSignal) {
  const result = await rpcRequest<Record<string, never>, SchemasListedResult>(
    'schema.list',
    {},
    { signal },
  )

  return result.schemas
}

export async function getSchema(schemaId: string, signal?: AbortSignal) {
  return rpcRequest<{ schema_id: string }, SchemaResource>(
    'schema.get',
    { schema_id: schemaId },
    { signal },
  )
}

export async function createSchema(
  params: {
    display_name: string
    fields: Record<string, StateFieldSchema>
    schema_id: string
    tags: string[]
  },
  signal?: AbortSignal,
) {
  return rpcRequest<typeof params, SchemaResource>('schema.create', params, { signal })
}

export async function updateSchema(
  params: {
    display_name?: string
    fields?: Record<string, StateFieldSchema>
    schema_id: string
    tags?: string[]
  },
  signal?: AbortSignal,
) {
  return rpcRequest<typeof params, SchemaResource>('schema.update', params, { signal })
}

export async function deleteSchema(schemaId: string, signal?: AbortSignal) {
  return rpcRequest<{ schema_id: string }, SchemaDeletedResult>(
    'schema.delete',
    { schema_id: schemaId },
    { signal },
  )
}
