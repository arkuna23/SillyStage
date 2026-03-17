export function createJsonExportFileName(prefix: string) {
  const stamp = new Date().toISOString().replaceAll(':', '-').replaceAll('.', '-')
  return `${prefix}-${stamp}.json`
}

export function downloadJsonFile(fileName: string, payload: unknown) {
  const blob = new Blob([`${JSON.stringify(payload, null, 2)}\n`], {
    type: 'application/json',
  })
  const url = URL.createObjectURL(blob)
  const anchor = document.createElement('a')

  anchor.href = url
  anchor.download = fileName
  anchor.click()

  window.setTimeout(() => {
    URL.revokeObjectURL(url)
  }, 0)
}

export async function readJsonFile(file: File) {
  const raw = await file.text()
  return JSON.parse(raw) as unknown
}
