import { isRpcConflict } from './rpc'

export type BatchDeleteResult<T> = {
  conflictCount: number
  deleted: T[]
  failed: T[]
}

export async function runBatchDelete<T>(
  targets: ReadonlyArray<T>,
  removeTarget: (target: T) => Promise<void>,
): Promise<BatchDeleteResult<T>> {
  const deleted: T[] = []
  const failed: T[] = []
  let conflictCount = 0

  for (const target of targets) {
    try {
      await removeTarget(target)
      deleted.push(target)
    } catch (error) {
      failed.push(target)

      if (isRpcConflict(error)) {
        conflictCount += 1
      }
    }
  }

  return {
    conflictCount,
    deleted,
    failed,
  }
}
