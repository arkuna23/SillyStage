export const appPaths = {
  apis: '/workspace/apis',
  characters: '/workspace/characters',
  dashboard: '/workspace/dashboard',
  lorebooks: '/workspace/lorebooks',
  playerProfiles: '/workspace/player-profiles',
  presets: '/workspace/presets',
  root: '/',
  schemas: '/workspace/schemas',
  stage: '/stage',
  stageRoot: '/stage',
  stories: '/workspace/stories',
  storyResources: '/workspace/story-resources',
  workspace: '/workspace/dashboard',
  workspaceRoot: '/workspace',
} as const

export const backendPaths = {
  download(resourceId: string, fileId: string) {
    return `/download/${encodeURIComponent(resourceId)}/${encodeURIComponent(fileId)}`
  },
  healthz: '/healthz',
  rpc: '/rpc',
  upload(resourceId: string, fileId: string) {
    return `/upload/${encodeURIComponent(resourceId)}/${encodeURIComponent(fileId)}`
  },
} as const
