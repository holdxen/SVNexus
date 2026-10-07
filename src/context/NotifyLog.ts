import { create } from 'zustand'

import type { WorkingCopyNotify } from '@/bindings/WorkingCopyNotify'
import {
  appendNotify,
  EMPTY_NOTIFY_LIST_STATE,
  WorkingCopyNotifyEntry,
  WorkingCopyNotifyListState,
} from '@/components/subversion/workingCopyNotify'

import type { Identity } from './TabContent'

/** 每个 tab 保留的通知条数上限，超出后丢弃最旧的 */
export const NOTIFY_LOG_LIMIT = 500

export interface NotifyLogBucket extends WorkingCopyNotifyListState {
  /** 打开面板之前累积的新通知数，用于工具栏的小红点 */
  unread: number
}

interface NotifyLogStore {
  buckets: Map<Identity, NotifyLogBucket>
  append: (identity: Identity, notify: WorkingCopyNotify) => void
  markSeen: (identity: Identity) => void
  clear: (identity: Identity) => void
  remove: (identity: Identity) => void
}

const EMPTY_BUCKET: NotifyLogBucket = { ...EMPTY_NOTIFY_LIST_STATE, unread: 0 }

function bucketOf(buckets: Map<Identity, NotifyLogBucket>, identity: Identity): NotifyLogBucket {
  return buckets.get(identity) ?? EMPTY_BUCKET
}

export const useNotifyLogStore = create<NotifyLogStore>()((set) => {
  /** 只替换目标 tab 的桶，其它桶保持原引用，这样订阅别的 tab 的组件不会重渲染 */
  const replace = (
    buckets: Map<Identity, NotifyLogBucket>,
    identity: Identity,
    bucket: NotifyLogBucket,
  ) => {
    const next = new Map(buckets)
    next.set(identity, bucket)
    return next
  }

  return {
    buckets: new Map(),

    append: (identity, notify) =>
      set((state) => {
        const bucket = bucketOf(state.buckets, identity)
        const appended = appendNotify(bucket, notify, NOTIFY_LOG_LIMIT)
        return {
          buckets: replace(state.buckets, identity, {
            ...appended,
            unread: bucket.unread + 1,
          }),
        }
      }),

    markSeen: (identity) =>
      set((state) => {
        const bucket = state.buckets.get(identity)
        if (bucket === undefined || bucket.unread === 0) {
          return state
        }
        return { buckets: replace(state.buckets, identity, { ...bucket, unread: 0 }) }
      }),

    clear: (identity) =>
      set((state) => {
        if (!state.buckets.has(identity)) {
          return state
        }
        return {
          buckets: replace(state.buckets, identity, { ...EMPTY_NOTIFY_LIST_STATE, unread: 0 }),
        }
      }),

    remove: (identity) =>
      set((state) => {
        if (!state.buckets.has(identity)) {
          return state
        }
        const next = new Map(state.buckets)
        next.delete(identity)
        return { buckets: next }
      }),
  }
})

const NO_ENTRIES: WorkingCopyNotifyEntry[] = []

export function useNotifyLogEntries(identity: Identity | null): WorkingCopyNotifyEntry[] {
  return useNotifyLogStore((state) =>
    identity === null ? NO_ENTRIES : (state.buckets.get(identity)?.entries ?? NO_ENTRIES),
  )
}

/** 返回布尔值而不是计数：工具栏只关心"有没有未读"，避免每条通知都重渲染整个顶栏 */
export function useNotifyLogUnread(identity: Identity | null): boolean {
  return useNotifyLogStore(
    (state) => identity !== null && (state.buckets.get(identity)?.unread ?? 0) > 0,
  )
}
