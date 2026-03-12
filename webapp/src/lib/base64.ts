export function bytesToBase64(bytes: Uint8Array) {
  let binary = ''
  const chunkSize = 0x8000

  for (let index = 0; index < bytes.length; index += chunkSize) {
    const chunk = bytes.subarray(index, index + chunkSize)
    binary += String.fromCharCode(...chunk)
  }

  return btoa(binary)
}

export function base64ToBytes(base64: string) {
  const binary = atob(base64)
  const bytes = new Uint8Array(binary.length)

  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index)
  }

  return bytes
}

export async function fileToBytes(file: File) {
  return new Uint8Array(await file.arrayBuffer())
}

export async function fileToBase64(file: File) {
  return bytesToBase64(await fileToBytes(file))
}

export async function sha256Hex(bytes: Uint8Array) {
  const digestInput = new Uint8Array(bytes.byteLength)

  digestInput.set(bytes)

  const digest = await crypto.subtle.digest('SHA-256', digestInput)
  const hashBytes = new Uint8Array(digest)

  return Array.from(hashBytes, (byte) => byte.toString(16).padStart(2, '0')).join('')
}

export function toDataUrl(base64: string, mimeType: string) {
  return `data:${mimeType};base64,${base64}`
}

export function downloadBase64File(args: {
  base64: string
  contentType: string
  fileName: string
}) {
  const bytes = base64ToBytes(args.base64)
  const blob = new Blob([bytes], { type: args.contentType })
  const objectUrl = URL.createObjectURL(blob)
  const anchor = document.createElement('a')

  anchor.href = objectUrl
  anchor.download = args.fileName
  anchor.click()

  window.setTimeout(() => {
    URL.revokeObjectURL(objectUrl)
  }, 0)
}
