const STORAGE_KEY = 'sillystage.character-folders'

function normalizeFolderName(folder: string) {
  return folder.trim().replace(/[\\/]+/g, ' ')
}

function sortFolders(folders: string[]) {
  return [...folders].sort((left, right) => left.localeCompare(right))
}

export function loadCharacterFolderRegistry() {
  if (typeof window === 'undefined') {
    return [] as string[]
  }

  try {
    const rawValue = window.localStorage.getItem(STORAGE_KEY)
    if (!rawValue) {
      return []
    }

    const parsed = JSON.parse(rawValue)
    if (!Array.isArray(parsed)) {
      return []
    }

    return sortFolders(
      parsed
        .filter((value): value is string => typeof value === 'string')
        .map(normalizeFolderName)
        .filter((value, index, values) => value.length > 0 && values.indexOf(value) === index),
    )
  } catch {
    return []
  }
}

export function saveCharacterFolderRegistry(folders: string[]) {
  if (typeof window === 'undefined') {
    return
  }

  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(sortFolders(folders)))
}

export function addCharacterFolderRegistryEntry(folders: string[], folder: string) {
  const normalized = normalizeFolderName(folder)
  if (!normalized) {
    return sortFolders(folders)
  }

  if (folders.includes(normalized)) {
    return sortFolders(folders)
  }

  return sortFolders([...folders, normalized])
}

export function renameCharacterFolderRegistryEntry(
  folders: string[],
  currentFolder: string,
  nextFolder: string,
) {
  const normalizedCurrent = normalizeFolderName(currentFolder)
  const normalizedNext = normalizeFolderName(nextFolder)

  const nextFolders = folders.filter((folder) => folder !== normalizedCurrent)
  if (!normalizedNext) {
    return sortFolders(nextFolders)
  }

  if (nextFolders.includes(normalizedNext)) {
    return sortFolders(nextFolders)
  }

  return sortFolders([...nextFolders, normalizedNext])
}

export function removeCharacterFolderRegistryEntry(folders: string[], folder: string) {
  const normalized = normalizeFolderName(folder)
  return sortFolders(folders.filter((item) => item !== normalized))
}

export function normalizeCharacterFolderRegistryName(folder: string) {
  return normalizeFolderName(folder)
}
