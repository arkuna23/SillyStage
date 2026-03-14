import { useOutletContext } from 'react-router-dom'

export type WorkspaceRailStat = {
  label: string
  value: number | string
}

export type WorkspaceRailContent = {
  description?: string
  stats: ReadonlyArray<WorkspaceRailStat>
  title: string
}

export type WorkspaceLayoutContextValue = {
  setRailContent: (content: WorkspaceRailContent | null) => void
}

export function useWorkspaceLayoutContext() {
  return useOutletContext<WorkspaceLayoutContextValue>()
}
