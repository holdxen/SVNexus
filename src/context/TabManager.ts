import { createContext, useContext } from 'react'

import { TabModel } from '@/tab/Tab'

import { Identity } from './TabContent'

export interface TabViewModel extends TabModel {
  content: React.ReactNode
}

export interface TabManagerContext {
  add: (model: TabViewModel, jump: boolean) => void
  goTo: (id: Identity) => void
  goToLast: () => void
  setTitle: (id: Identity, title: string) => void
  setTooltip: (id: Identity, tooltip?: string) => void
  close: (id: Identity) => void
  closeOnly: (id: Identity) => void
  openWorkingCopy: (from: Identity, path: string) => void
  reload: (id: Identity) => void
}

export const TabManager = createContext<TabManagerContext | null>(null)

export function useTabManager(): TabManagerContext {
  const context = useContext(TabManager)

  if (context === null) {
    throw new Error('No TabManager context provided')
  }

  return context
}
