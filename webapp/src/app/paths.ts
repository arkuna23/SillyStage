export const appPaths = {
  apis: '/workspace/apis',
  characters: '/workspace/characters',
  dashboard: '/workspace/dashboard',
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
  healthz: '/healthz',
  rpc: '/rpc',
} as const
