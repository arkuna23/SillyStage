type StoryResourceDisplayTarget = {
  display_name: string
  resource_id: string
}

function truncateLabel(value: string, maxLength: number) {
  if (value.length <= maxLength) {
    return value
  }

  return `${value.slice(0, Math.max(0, maxLength - 3)).trimEnd()}...`
}

export function getStoryResourceDisplayName(resource: StoryResourceDisplayTarget) {
  const displayName = resource.display_name.trim()

  return displayName.length > 0 ? displayName : resource.resource_id
}

export function getStoryResourceOptionLabel(resource: StoryResourceDisplayTarget) {
  const displayName = truncateLabel(getStoryResourceDisplayName(resource), 48)

  return displayName === resource.resource_id
    ? displayName
    : `${displayName} · ${resource.resource_id}`
}
