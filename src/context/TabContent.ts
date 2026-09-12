import { createContext, useContext } from 'react'

export type Identity = string | number

interface TabContentContext {
  identity: Identity
}

export const TabContentContextProvider = createContext<TabContentContext | null>(null)

export function useTabContent(): TabContentContext {
  const context = useContext(TabContentContextProvider)

  if (context === null) {
    throw new Error('No subversion context provided')
  }

  return context
}
