import { backendPaths } from '../app/paths'

export const heroMetricItems = [
  { value: 'ZH / EN', labelKey: 'home.highlights.metrics.locales' },
  { value: `POST ${backendPaths.rpc}`, labelKey: 'home.highlights.metrics.rpc' },
  { value: 'Radix + Tailwind', labelKey: 'home.highlights.metrics.ui' },
] as const

export const stagePrincipleItems = [
  { id: 'fidelity', key: 'home.principles.items.fidelity' },
  { id: 'thin-client', key: 'home.principles.items.thinClient' },
  { id: 'composable', key: 'home.principles.items.composable' },
] as const

export const rehearsalMoodOptions = [
  { value: 'intrigue', labelKey: 'home.rehearsal.moods.intrigue' },
  { value: 'wonder', labelKey: 'home.rehearsal.moods.wonder' },
  { value: 'conflict', labelKey: 'home.rehearsal.moods.conflict' },
] as const

export const transportSurfaceItems = [
  {
    value: 'rpc',
    method: 'POST',
    target: backendPaths.rpc,
    tabLabelKey: 'home.transport.items.rpc.tabLabel',
    titleKey: 'home.transport.items.rpc.title',
    summaryKey: 'home.transport.items.rpc.summary',
    detailKeys: [
      'home.transport.items.rpc.details.0',
      'home.transport.items.rpc.details.1',
      'home.transport.items.rpc.details.2',
    ],
  },
  {
    value: 'stream',
    method: 'SSE',
    target: 'turn execution',
    tabLabelKey: 'home.transport.items.stream.tabLabel',
    titleKey: 'home.transport.items.stream.title',
    summaryKey: 'home.transport.items.stream.summary',
    detailKeys: [
      'home.transport.items.stream.details.0',
      'home.transport.items.stream.details.1',
      'home.transport.items.stream.details.2',
    ],
  },
  {
    value: 'health',
    method: 'GET',
    target: backendPaths.healthz,
    tabLabelKey: 'home.transport.items.health.tabLabel',
    titleKey: 'home.transport.items.health.title',
    summaryKey: 'home.transport.items.health.summary',
    detailKeys: [
      'home.transport.items.health.details.0',
      'home.transport.items.health.details.1',
      'home.transport.items.health.details.2',
    ],
  },
] as const

export const stageKitItems = [
  {
    id: 'button',
    titleKey: 'home.stageKit.items.button.title',
    descriptionKey: 'home.stageKit.items.button.description',
  },
  {
    id: 'input',
    titleKey: 'home.stageKit.items.input.title',
    descriptionKey: 'home.stageKit.items.input.description',
  },
  {
    id: 'card',
    titleKey: 'home.stageKit.items.card.title',
    descriptionKey: 'home.stageKit.items.card.description',
  },
  {
    id: 'select',
    titleKey: 'home.stageKit.items.select.title',
    descriptionKey: 'home.stageKit.items.select.description',
  },
] as const
