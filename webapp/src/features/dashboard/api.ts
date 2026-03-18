import { rpcRequest } from '../../lib/rpc'
import {
  downloadBinaryResource,
  triggerBlobDownload,
  uploadBinaryResource,
} from '../../lib/binary-resource'
import type {
  DashboardPayload,
  DataPackageExportPrepareParams,
  DataPackageExportPreparedResult,
  DataPackageImportCommittedResult,
  DataPackageImportPreparedResult,
  ResourceFile,
  ResourceFileRef,
} from './types'

export async function getDashboard(signal?: AbortSignal) {
  return rpcRequest<Record<string, never>, DashboardPayload>('dashboard.get', {}, { signal })
}

export async function prepareDataPackageExport(
  params: DataPackageExportPrepareParams,
  signal?: AbortSignal,
) {
  return rpcRequest<DataPackageExportPrepareParams, DataPackageExportPreparedResult>(
    'data_package.export_prepare',
    params,
    { signal },
  )
}

export async function prepareDataPackageImport(signal?: AbortSignal) {
  return rpcRequest<Record<string, never>, DataPackageImportPreparedResult>(
    'data_package.import_prepare',
    {},
    { signal },
  )
}

export async function commitDataPackageImport(importId: string, signal?: AbortSignal) {
  return rpcRequest<{ import_id: string }, DataPackageImportCommittedResult>(
    'data_package.import_commit',
    { import_id: importId },
    { signal },
  )
}

export async function uploadDataPackageArchive(args: {
  archive: ResourceFileRef
  file: File
  signal?: AbortSignal
}) {
  return uploadBinaryResource<ResourceFile>({
    body: args.file,
    contentType: args.file.type || 'application/zip',
    fileId: args.archive.file_id,
    fileName: args.file.name,
    resourceId: args.archive.resource_id,
    signal: args.signal,
  })
}

export async function downloadDataPackageArchive(args: {
  archive: ResourceFileRef
  fallbackFileName: string
  signal?: AbortSignal
}) {
  const result = await downloadBinaryResource({
    fileId: args.archive.file_id,
    resourceId: args.archive.resource_id,
    signal: args.signal,
  })

  triggerBlobDownload({
    blob: result.blob,
    fileName: result.fileName ?? args.fallbackFileName,
  })

  return result
}
