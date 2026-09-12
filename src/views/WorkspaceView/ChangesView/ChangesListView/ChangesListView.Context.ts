// import { createContext, useContext } from 'react'

// import { createChangesListViewStore } from './ChangesListView.Store'

// export type ChangesListViewContext = ReturnType<typeof createChangesListViewStore>

// export interface ChangesListViewContext {
//   store: ReturnType<typeof createChangesListViewStore>
// }

// export const ChangesListViewContextProvider = createContext<ChangesListViewContext | null>(null)

// export function useChangesListViewContext(): ChangesListViewContext {
//   const provider = useContext(ChangesListViewContextProvider)

//   if (provider === null) {
//     throw new Error('No context provided')
//   }

//   return provider
// }
