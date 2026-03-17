import type { IconDefinition } from '@fortawesome/fontawesome-svg-core'
import { faBookAtlas } from '@fortawesome/free-solid-svg-icons/faBookAtlas'
import { faDiagramProject } from '@fortawesome/free-solid-svg-icons/faDiagramProject'
import { faFileLines } from '@fortawesome/free-solid-svg-icons/faFileLines'
import { faGaugeHigh } from '@fortawesome/free-solid-svg-icons/faGaugeHigh'
import { faIdCard } from '@fortawesome/free-solid-svg-icons/faIdCard'
import { faPlug } from '@fortawesome/free-solid-svg-icons/faPlug'
import { faSliders } from '@fortawesome/free-solid-svg-icons/faSliders'
import { faBookOpen } from '@fortawesome/free-solid-svg-icons/faBookOpen'
import { faUserGroup } from '@fortawesome/free-solid-svg-icons/faUserGroup'
import type { TFunction } from 'i18next'

import { appPaths } from './paths'

export type WorkspaceNavigationKey =
  | 'apis'
  | 'characters'
  | 'dashboard'
  | 'lorebooks'
  | 'playerProfiles'
  | 'presets'
  | 'schemas'
  | 'stories'
  | 'storyResources'

export type WorkspaceNavigationItem = {
  icon: IconDefinition
  key: WorkspaceNavigationKey
  label: string
  to: string
}

export function getWorkspaceNavigationItems(t: TFunction): ReadonlyArray<WorkspaceNavigationItem> {
  return [
    {
      icon: faGaugeHigh,
      key: 'dashboard',
      label: t('workspace.sidebar.items.dashboard.label'),
      to: appPaths.dashboard,
    },
    {
      icon: faPlug,
      key: 'apis',
      label: t('workspace.sidebar.items.apis.label'),
      to: appPaths.apis,
    },
    {
      icon: faSliders,
      key: 'presets',
      label: t('workspace.sidebar.items.presets.label'),
      to: appPaths.presets,
    },
    {
      icon: faDiagramProject,
      key: 'schemas',
      label: t('workspace.sidebar.items.schemas.label'),
      to: appPaths.schemas,
    },
    {
      icon: faBookAtlas,
      key: 'lorebooks',
      label: t('workspace.sidebar.items.lorebooks.label'),
      to: appPaths.lorebooks,
    },
    {
      icon: faIdCard,
      key: 'playerProfiles',
      label: t('workspace.sidebar.items.playerProfiles.label'),
      to: appPaths.playerProfiles,
    },
    {
      icon: faUserGroup,
      key: 'characters',
      label: t('workspace.sidebar.items.characters.label'),
      to: appPaths.characters,
    },
    {
      icon: faFileLines,
      key: 'storyResources',
      label: t('workspace.sidebar.items.storyResources.label'),
      to: appPaths.storyResources,
    },
    {
      icon: faBookOpen,
      key: 'stories',
      label: t('workspace.sidebar.items.stories.label'),
      to: appPaths.stories,
    },
  ]
}

export function getWorkspaceNavigationValue(
  pathname: string,
  items: ReadonlyArray<WorkspaceNavigationItem>,
): string {
  const matchedItem = items.find((item) => pathname.startsWith(item.to))

  return matchedItem?.to ?? appPaths.dashboard
}
