import { RefObject } from 'react'
import { create } from 'zustand'

import { SingleTaskQueue } from '@/utils/Queue'

import { WorkspaceViewRef } from './WorkspaceView'

export interface WorkspaceViewStore {
  openedWorkingCopy: Map<string, RefObject<WorkspaceViewRef | null>>
  openWorkingCopyQueue: SingleTaskQueue
  addOpendWorkingCopy: (path: string, view: RefObject<WorkspaceViewRef | null>) => void
  removeOpenedWorkingCopy: (path: string) => void
}

export const useWorkingspaceViewStore = create<WorkspaceViewStore>()((set) => {
  const queue = new SingleTaskQueue()
  queue.single = false
  return {
    openedWorkingCopy: new Map(),
    addOpendWorkingCopy: (path, view) => {
      set((state) => {
        const next = new Map(state.openedWorkingCopy)
        next.set(path, view)
        return { openedWorkingCopy: next }
      })
    },
    openWorkingCopyQueue: queue,
    removeOpenedWorkingCopy(path) {
      set((state) => {
        const next = new Map(state.openedWorkingCopy)
        next.delete(path)
        return { openedWorkingCopy: next }
      })
    },
  }
})
