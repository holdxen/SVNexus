import { generateKeyBetween } from 'fractional-indexing'
import * as uuid from 'uuid'
import { create } from 'zustand'

import { WorkspaceGroup } from '@/bindings/WorkspaceGroup'
import { WorkspaceItem } from '@/bindings/WorkspaceItem'
import { invokeMessagePack } from '@/utils/MessagePack'
import { workspaceItemIdentity } from '@/utils/WorkspaceItem'

type WorkspaceItemState = {
  isModified?: boolean
  isConflicted?: boolean
  isInvalid?: boolean
  isLocked?: boolean
  isClean?: boolean
  revision?: { min: number; max: number }
}

export interface Database {
  workspaceItemStates: Map<string, WorkspaceItemState>
  workspaceGroups: WorkspaceGroup[]
  workspaceItems: WorkspaceItem[]

  loaded: boolean

  load: (force?: boolean) => Promise<void>

  addWorkspaceGroup: (name: string, members: string[]) => Promise<string>
  updateWorkspaceGroup: (group: WorkspaceGroup) => Promise<void>
  deleteWorkspaceGroup: (identity: string) => Promise<void>

  moveWorkspaceGroup: (from: number, to: number) => Promise<void>

  addWorkspaceItem: (
    builder: (order: string) => Promise<WorkspaceItem>,
    update?: boolean,
  ) => Promise<WorkspaceItem>
  deleteWorkspaceItem: (identity: string) => Promise<void>
  updateWorkspaceItem: (item: WorkspaceItem) => Promise<void>

  setWorkspaceItemState: (identity: string, state: WorkspaceItemState) => void
}

function workspaceItemOrder(item: WorkspaceItem): string {
  if ('workingCopy' in item) {
    return item.workingCopy.order
  } else if ('repository' in item) {
    return item.repository.order
  } else {
    throw new Error('Unexpected workspace item')
  }
}

export const useDatabase = create<Database>()((set, get) => {
  return {
    workspaceItemStates: new Map<string, WorkspaceItemState>(),
    workspaceGroups: [],
    workspaceItems: [],
    loaded: false,

    async load(force?: boolean) {
      if (get().loaded && !(force ?? false)) {
        return
      }

      const groups = await invokeMessagePack<WorkspaceGroup[]>('database_workspace_groups')

      groups.sort((first, second) => {
        if (first.order < second.order) return -1
        if (first.order > second.order) return 1
        return 0
      })

      const items = await invokeMessagePack<WorkspaceItem[]>('database_workspace_items')

      items.sort((first, second) => {
        const a = workspaceItemOrder(first)
        const b = workspaceItemOrder(second)
        if (a < b) return -1
        if (a > b) return 1
        return 0
      })

      set({
        workspaceGroups: groups,
        workspaceItems: items,
        loaded: true,
      })
    },
    async addWorkspaceGroup(name: string, members: string[]) {
      const identity = uuid.v4()

      const groups = get().workspaceGroups

      // const order = groups.length === 0 ? 10000 : groups[groups.length - 1].order + 10000
      //

      const last = groups.length === 0 ? null : groups[groups.length - 1].order
      const order = generateKeyBetween(last, null)

      const group: WorkspaceGroup = {
        identity,
        name,
        members,
        order,
      }

      await invokeMessagePack<string>('database_add_workspace_group', {
        group,
      })

      set((state) => ({
        workspaceGroups: [...state.workspaceGroups, group],
      }))

      return identity
    },

    async updateWorkspaceGroup(group: WorkspaceGroup) {
      await invokeMessagePack('database_update_workspace_group', {
        group,
      })

      set((state) => ({
        workspaceGroups: state.workspaceGroups.map((e) =>
          e.identity === group.identity ? group : e,
        ),
      }))
    },

    async deleteWorkspaceGroup(identity) {
      await invokeMessagePack('database_delete_workspace_group', {
        identity,
      })

      set((state) => ({
        workspaceGroups: state.workspaceGroups.filter((e) => e.identity !== identity),
      }))
    },

    async moveWorkspaceGroup(from, to) {
      const groups = get().workspaceGroups
      if (groups.length - 1 < from) {
        console.error('No such index:', from)
        return
      }
      if (groups.length - 1 < to) {
        console.error('No such index:', to)
        return
      }
      if (from === to) {
        return
      }

      let order: string | null = null
      if (to === 0) {
        order = generateKeyBetween(null, groups[0].order)
      } else if (to === groups.length - 1) {
        order = generateKeyBetween(groups[to].order, null)
      } else {
        if (from < to) {
          order = generateKeyBetween(groups[to].order, groups[to + 1].order)
        } else {
          order = generateKeyBetween(groups[to - 1].order, groups[to].order)
        }
      }

      const group: WorkspaceGroup = {
        ...groups[from],
        order,
      }

      await invokeMessagePack('database_update_workspace_group', {
        group,
      })

      groups[from] = group

      groups.sort((first, second) => {
        if (first.order < second.order) return -1
        if (first.order > second.order) return 1
        return 0
      })

      set({
        workspaceGroups: [...groups],
      })
    },

    async addWorkspaceItem(builder, update) {
      const items = get().workspaceItems
      const first = items.length === 0 ? null : workspaceItemOrder(items[0])

      const order = generateKeyBetween(null, first)
      const item = await builder(order)

      await invokeMessagePack('database_add_workspace_item', { item })
      if (update) {
        set((state) => ({
          workspaceItems: [
            item,
            ...state.workspaceItems.filter(
              (i) => workspaceItemIdentity(i) !== workspaceItemIdentity(item),
            ),
          ],
        }))
      } else {
        set((state) => ({
          workspaceItems: [item, ...state.workspaceItems],
        }))
      }
      return item
    },
    async deleteWorkspaceItem(identity) {
      await invokeMessagePack('database_delete_workspace_item', { identity })

      set((state) => ({
        workspaceItems: state.workspaceItems.filter((i) => workspaceItemIdentity(i) !== identity),
      }))
    },
    async updateWorkspaceItem(item) {
      await invokeMessagePack('database_add_workspace_item', { item })
      set((state) => {
        const items = [
          item,
          ...state.workspaceItems.filter(
            (e) => workspaceItemIdentity(e) !== workspaceItemIdentity(item),
          ),
        ]
        items.sort((first, second) => {
          if (workspaceItemOrder(first) < workspaceItemOrder(second)) return -1
          if (workspaceItemOrder(first) > workspaceItemOrder(second)) return 1
          return 0
        })
        return {
          workspaceItems: items,
        }
      })
    },
    // setWorkspaceItemState(identity, itemState) {
    //   set((state) => ({
    //     workspaceItemStates: {
    //       ...state.workspaceItemStates,
    //       [identity]: itemState
    //     },
    //   }))
    // },
    setWorkspaceItemState(identity, itemState) {
      set((state) => {
        const next = new Map(state.workspaceItemStates)
        next.set(identity, itemState)
        return {
          workspaceItemStates: next,
        }
      })
    },
  }
})
