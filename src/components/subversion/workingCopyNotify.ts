import type { NodeKind } from '@/bindings/NodeKind'
import type { SubversionErrorCode } from '@/bindings/SubversionErrorCode'
import type { WorkingCopyNotify } from '@/bindings/WorkingCopyNotify'
import type { WorkingCopyNotifyAction } from '@/bindings/WorkingCopyNotifyAction'
import type { WorkingCopyNotifyLockState } from '@/bindings/WorkingCopyNotifyLockState'
import type { WorkingCopyNotifyState } from '@/bindings/WorkingCopyNotifyState'
import type { MessageKey } from '@/i18n'

/**
 * 动作分组。除了决定配色和图标，它还是列表合并策略的依据：
 * `progress` 是"又发生了一次"的过程事件，必须逐条追加；
 * 其余分组描述的是某个路径的最终结果，同路径的新事件应当覆盖旧事件。
 */
export type NotifyActionGroup =
  | 'added'
  | 'deleted'
  | 'modified'
  | 'conflict'
  | 'failed'
  | 'skipped'
  | 'locked'
  | 'resolve'
  | 'progress'

export interface NotifyActionMeta {
  group: NotifyActionGroup
  label: MessageKey
  /**
   * 覆盖分组的默认合并策略。分组的默认规则是"终止型按路径合并、progress 逐条追加"，
   * 少数动作语义上是逐次发生的，但配色上不该退化成中性灰，就用这个字段单独声明。
   */
  episodic?: boolean
}

/** 覆盖全部 90 个 WorkingCopyNotifyAction；漏一个 TS 会直接报错 */
export const NOTIFY_ACTION_META: Record<WorkingCopyNotifyAction, NotifyActionMeta> = {
  Add: { group: 'added', label: 'components.notify.action.Add' },
  Copy: { group: 'added', label: 'components.notify.action.Copy' },
  Delete: { group: 'deleted', label: 'components.notify.action.Delete' },
  Restore: { group: 'modified', label: 'components.notify.action.Restore' },
  Revert: { group: 'modified', label: 'components.notify.action.Revert' },
  FailedRevert: { group: 'failed', label: 'components.notify.action.FailedRevert' },
  Resolved: { group: 'resolve', label: 'components.notify.action.Resolved' },
  Skip: { group: 'skipped', label: 'components.notify.action.Skip' },
  UpdateDelete: { group: 'deleted', label: 'components.notify.action.UpdateDelete' },
  UpdateAdd: { group: 'added', label: 'components.notify.action.UpdateAdd' },
  UpdateUpdate: { group: 'modified', label: 'components.notify.action.UpdateUpdate' },
  UpdateCompleted: { group: 'progress', label: 'components.notify.action.UpdateCompleted' },
  UpdateExternal: { group: 'progress', label: 'components.notify.action.UpdateExternal' },
  StatusCompleted: { group: 'progress', label: 'components.notify.action.StatusCompleted' },
  StatusExternal: { group: 'progress', label: 'components.notify.action.StatusExternal' },
  CommitModified: { group: 'modified', label: 'components.notify.action.CommitModified' },
  CommitAdded: { group: 'added', label: 'components.notify.action.CommitAdded' },
  CommitDeleted: { group: 'deleted', label: 'components.notify.action.CommitDeleted' },
  CommitReplaced: { group: 'modified', label: 'components.notify.action.CommitReplaced' },
  CommitPostfixTxDelta: {
    group: 'progress',
    label: 'components.notify.action.CommitPostfixTxDelta',
  },
  BlameRevision: { group: 'progress', label: 'components.notify.action.BlameRevision' },
  Locked: { group: 'locked', label: 'components.notify.action.Locked' },
  Unlocked: { group: 'locked', label: 'components.notify.action.Unlocked' },
  FailedLock: { group: 'failed', label: 'components.notify.action.FailedLock' },
  FailedUnlock: { group: 'failed', label: 'components.notify.action.FailedUnlock' },
  Exists: { group: 'skipped', label: 'components.notify.action.Exists' },
  ChangelistSet: { group: 'modified', label: 'components.notify.action.ChangelistSet' },
  ChangelistClear: { group: 'modified', label: 'components.notify.action.ChangelistClear' },
  ChangelistMoved: { group: 'modified', label: 'components.notify.action.ChangelistMoved' },
  MergeBegin: { group: 'progress', label: 'components.notify.action.MergeBegin' },
  ForeignMergeBegin: { group: 'progress', label: 'components.notify.action.ForeignMergeBegin' },
  UpdateReplace: { group: 'modified', label: 'components.notify.action.UpdateReplace' },
  PropertyAdded: { group: 'added', label: 'components.notify.action.PropertyAdded' },
  PropertyModified: { group: 'modified', label: 'components.notify.action.PropertyModified' },
  PropertyDeleted: { group: 'deleted', label: 'components.notify.action.PropertyDeleted' },
  PropertyDeletedNonexistent: {
    group: 'skipped',
    label: 'components.notify.action.PropertyDeletedNonexistent',
  },
  RevisionPropertySet: { group: 'added', label: 'components.notify.action.RevisionPropertySet' },
  RevisionPropertyDeleted: {
    group: 'deleted',
    label: 'components.notify.action.RevisionPropertyDeleted',
  },
  MergeCompleted: { group: 'progress', label: 'components.notify.action.MergeCompleted' },
  TreeConflict: { group: 'conflict', label: 'components.notify.action.TreeConflict' },
  FailedExternal: { group: 'failed', label: 'components.notify.action.FailedExternal' },
  UpdateStarted: { group: 'progress', label: 'components.notify.action.UpdateStarted' },
  UpdateSkipObstruction: {
    group: 'skipped',
    label: 'components.notify.action.UpdateSkipObstruction',
  },
  UpdateSkipWorkingOnly: {
    group: 'skipped',
    label: 'components.notify.action.UpdateSkipWorkingOnly',
  },
  UpdateSkipAccessDenied: {
    group: 'skipped',
    label: 'components.notify.action.UpdateSkipAccessDenied',
  },
  UpdateExternalRemoved: {
    group: 'deleted',
    label: 'components.notify.action.UpdateExternalRemoved',
  },
  UpdateShadowedAdd: { group: 'skipped', label: 'components.notify.action.UpdateShadowedAdd' },
  UpdateShadowedUpdate: {
    group: 'skipped',
    label: 'components.notify.action.UpdateShadowedUpdate',
  },
  UpdateShadowedDelete: {
    group: 'skipped',
    label: 'components.notify.action.UpdateShadowedDelete',
  },
  MergeRecordInfo: { group: 'progress', label: 'components.notify.action.MergeRecordInfo' },
  UpgradedPath: { group: 'modified', label: 'components.notify.action.UpgradedPath' },
  MergeRecordInfoBegin: {
    group: 'progress',
    label: 'components.notify.action.MergeRecordInfoBegin',
  },
  MergeElideInfo: { group: 'progress', label: 'components.notify.action.MergeElideInfo' },
  Patch: { group: 'progress', label: 'components.notify.action.Patch' },
  // 每个 hunk 各回调一次，同路径反复出现；合并会把 hunk 字段压成一条而失去意义
  PatchAppliedHunk: {
    group: 'modified',
    label: 'components.notify.action.PatchAppliedHunk',
    episodic: true,
  },
  PatchRejectHunk: {
    group: 'failed',
    label: 'components.notify.action.PatchRejectHunk',
    episodic: true,
  },
  PatchHunkAlreadyApplied: {
    group: 'skipped',
    label: 'components.notify.action.PatchHunkAlreadyApplied',
    episodic: true,
  },
  CommitCopied: { group: 'added', label: 'components.notify.action.CommitCopied' },
  CommitCopiedReplaced: {
    group: 'added',
    label: 'components.notify.action.CommitCopiedReplaced',
  },
  UrlRedirect: { group: 'progress', label: 'components.notify.action.UrlRedirect' },
  PathNonexistent: { group: 'failed', label: 'components.notify.action.PathNonexistent' },
  Exclude: { group: 'skipped', label: 'components.notify.action.Exclude' },
  FailedConflict: { group: 'failed', label: 'components.notify.action.FailedConflict' },
  FailedMissing: { group: 'failed', label: 'components.notify.action.FailedMissing' },
  FailedOutOfDate: { group: 'failed', label: 'components.notify.action.FailedOutOfDate' },
  FailedNoParent: { group: 'failed', label: 'components.notify.action.FailedNoParent' },
  FailedLocked: { group: 'failed', label: 'components.notify.action.FailedLocked' },
  FailedForbiddenByServer: {
    group: 'failed',
    label: 'components.notify.action.FailedForbiddenByServer',
  },
  SkipConflicted: { group: 'skipped', label: 'components.notify.action.SkipConflicted' },
  UpdateBrokenLock: { group: 'failed', label: 'components.notify.action.UpdateBrokenLock' },
  FailedObstruction: { group: 'failed', label: 'components.notify.action.FailedObstruction' },
  ConflictResolverStarting: {
    group: 'conflict',
    label: 'components.notify.action.ConflictResolverStarting',
  },
  ConflictResolverDone: {
    group: 'conflict',
    label: 'components.notify.action.ConflictResolverDone',
  },
  LeftLocalModifications: {
    group: 'skipped',
    label: 'components.notify.action.LeftLocalModifications',
  },
  ForeignCopyBegin: { group: 'progress', label: 'components.notify.action.ForeignCopyBegin' },
  MoveBroken: { group: 'failed', label: 'components.notify.action.MoveBroken' },
  CleanupExternal: { group: 'progress', label: 'components.notify.action.CleanupExternal' },
  FailedRequiresTarget: {
    group: 'failed',
    label: 'components.notify.action.FailedRequiresTarget',
  },
  InfoExternal: { group: 'progress', label: 'components.notify.action.InfoExternal' },
  CommitFinalizing: { group: 'progress', label: 'components.notify.action.CommitFinalizing' },
  ResolvedText: { group: 'resolve', label: 'components.notify.action.ResolvedText' },
  ResolvedProp: { group: 'resolve', label: 'components.notify.action.ResolvedProp' },
  ResolvedTree: { group: 'resolve', label: 'components.notify.action.ResolvedTree' },
  BeginSearchTreeConflictDetails: {
    group: 'progress',
    label: 'components.notify.action.BeginSearchTreeConflictDetails',
  },
  TreeConflictDetailsProgress: {
    group: 'progress',
    label: 'components.notify.action.TreeConflictDetailsProgress',
  },
  EndSearchTreeConflictDetails: {
    group: 'progress',
    label: 'components.notify.action.EndSearchTreeConflictDetails',
  },
  HydratingStart: { group: 'progress', label: 'components.notify.action.HydratingStart' },
  HydratingFile: { group: 'progress', label: 'components.notify.action.HydratingFile' },
  HydratingEnd: { group: 'progress', label: 'components.notify.action.HydratingEnd' },
  Warning: { group: 'failed', label: 'components.notify.action.Warning' },
}

const EPISODIC_GROUPS: ReadonlySet<NotifyActionGroup> = new Set<NotifyActionGroup>(['progress'])

const PATCH_ACTIONS: ReadonlySet<WorkingCopyNotifyAction> = new Set<WorkingCopyNotifyAction>([
  'Patch',
  'PatchAppliedHunk',
  'PatchRejectHunk',
  'PatchHunkAlreadyApplied',
])

export function actionGroup(action: WorkingCopyNotifyAction): NotifyActionGroup {
  return NOTIFY_ACTION_META[action].group
}

export function actionLabel(action: WorkingCopyNotifyAction): MessageKey {
  return NOTIFY_ACTION_META[action].label
}

export function isEpisodicAction(action: WorkingCopyNotifyAction): boolean {
  const meta = NOTIFY_ACTION_META[action]
  return meta.episodic === true || EPISODIC_GROUPS.has(meta.group)
}

/** SVN 只在 patch 相关动作上填 hunk 字段，其余动作全是 0，展示出来只会误导 */
export function isPatchAction(action: WorkingCopyNotifyAction): boolean {
  return PATCH_ACTIONS.has(action)
}

export const NOTIFY_GROUP_COLOR: Record<NotifyActionGroup, string> = {
  added: 'var(--semi-color-success)',
  resolve: 'var(--semi-color-success)',
  deleted: 'var(--semi-color-danger)',
  failed: 'var(--semi-color-danger)',
  conflict: 'var(--semi-color-warning)',
  locked: 'var(--semi-color-primary)',
  modified: 'var(--semi-color-primary)',
  skipped: 'var(--semi-color-text-2)',
  progress: 'var(--semi-color-text-1)',
}

export interface NotifyStateMeta {
  label: MessageKey
  /** null 表示这个取值不携带信息，行内不必展示 */
  color: string | null
}

const INERT_STATE: NotifyStateMeta = { label: 'components.notify.state.inapplicable', color: null }

export const NOTIFY_STATE_META: Record<WorkingCopyNotifyState, NotifyStateMeta> = {
  Inapplicable: INERT_STATE,
  Unknown: { label: 'components.notify.state.unknown', color: null },
  Unchanged: { label: 'components.notify.state.unchanged', color: null },
  Missing: { label: 'components.notify.state.missing', color: 'var(--semi-color-danger)' },
  Obstructed: { label: 'components.notify.state.obstructed', color: 'var(--semi-color-danger)' },
  Changed: { label: 'components.notify.state.changed', color: 'var(--semi-color-primary)' },
  Merged: { label: 'components.notify.state.merged', color: 'var(--semi-color-warning)' },
  Conflicted: { label: 'components.notify.state.conflicted', color: 'var(--semi-color-danger)' },
  SourceMissing: {
    label: 'components.notify.state.sourceMissing',
    color: 'var(--semi-color-danger)',
  },
}

export const NOTIFY_LOCK_STATE_META: Record<WorkingCopyNotifyLockState, NotifyStateMeta> = {
  InApplicable: INERT_STATE,
  Unknown: { label: 'components.notify.state.unknown', color: null },
  Unchanged: { label: 'components.notify.state.unchanged', color: null },
  Locked: { label: 'components.notify.lockState.locked', color: 'var(--semi-color-primary)' },
  Unlocked: { label: 'components.notify.lockState.unlocked', color: 'var(--semi-color-text-2)' },
}

export const NOTIFY_KIND_LABEL: Record<NodeKind, MessageKey> = {
  none: 'components.notify.kind.none',
  file: 'components.notify.kind.file',
  directory: 'components.notify.kind.directory',
  symlink: 'components.notify.kind.symlink',
  unknown: 'components.notify.kind.unknown',
}

/**
 * 通知里的 path 既可能是绝对路径，也可能相对当前工作目录。
 * `pathPrefix` 是 SVN 专门给出、用于从路径上剥掉的公共前缀，剥掉后更适合展示。
 */
export function notifyDisplayPath(notify: WorkingCopyNotify): string {
  const { path, pathPrefix } = notify
  if (pathPrefix !== null && pathPrefix !== '' && path.startsWith(pathPrefix)) {
    return path.slice(pathPrefix.length)
  }
  return path
}

/**
 * 真实目标：当通知描述的是一个 URL 时，`path` 只会是 "."，目标在 `url` 里。
 * 见 svn_wc.h 对 svn_wc_notify_t.path 的说明。
 */
export function notifyDisplayTarget(notify: WorkingCopyNotify): string {
  const path = notifyDisplayPath(notify)
  if ((path === '.' || path === '') && notify.url !== null) {
    return notify.url
  }
  return path
}

/** SVN 的 lock 时间是 apr_time_t，即微秒；0 表示没有设置过期时间 */
export function lockTimeToMillis(value: number): number | null {
  return value > 0 ? Math.floor(value / 1000) : null
}

/**
 * SubversionErrorCode 的 Unknown(i32) 变体在 TS 侧是 `{ unknown: number }` 而不是字符串，
 * 直接塞进 React 树会报错，也会出现 "[object Object]"。
 */
export function errorCodeText(code: SubversionErrorCode): string {
  return typeof code === 'string' ? code : `unknown(${code.unknown})`
}

export interface WorkingCopyNotifyEntry {
  /** 终止型条目用 path 做 key，同路径的后续事件原地替换，因此展开状态不会丢 */
  key: string
  notify: WorkingCopyNotify
  /** 该路径累计收到的通知次数 */
  count: number
  episodic: boolean
}

export interface WorkingCopyNotifyListState {
  entries: WorkingCopyNotifyEntry[]
  /** 过程型条目的唯一编号，避免同路径重复出现时 key 冲突 */
  serial: number
}

export const EMPTY_NOTIFY_LIST_STATE: WorkingCopyNotifyListState = { entries: [], serial: 0 }

/**
 * SVN 会对同一个路径反复回调（属性变了、正文变了、每个 hunk 各一次）。
 * 描述"最终结果"的动作按路径合并成一条，描述"又发生了一次"的过程动作逐条追加。
 */
export function appendNotify(
  state: WorkingCopyNotifyListState,
  notify: WorkingCopyNotify,
  maxItems: number,
): WorkingCopyNotifyListState {
  if (isEpisodicAction(notify.action)) {
    const serial = state.serial + 1
    const entry: WorkingCopyNotifyEntry = {
      key: `${notify.path}#${serial}`,
      notify,
      count: 1,
      episodic: true,
    }
    return trim({ entries: [...state.entries, entry], serial }, maxItems)
  }

  const index = state.entries.findIndex(
    (entry) => !entry.episodic && entry.notify.path === notify.path,
  )

  if (index < 0) {
    const entry: WorkingCopyNotifyEntry = {
      key: notify.path,
      notify,
      count: 1,
      episodic: false,
    }
    return trim({ entries: [...state.entries, entry], serial: state.serial }, maxItems)
  }

  const previous = state.entries[index]
  const entries = state.entries.slice()
  entries[index] = { ...previous, notify, count: previous.count + 1 }
  return { entries, serial: state.serial }
}

function trim(state: WorkingCopyNotifyListState, maxItems: number): WorkingCopyNotifyListState {
  if (state.entries.length <= maxItems) {
    return state
  }
  return { entries: state.entries.slice(state.entries.length - maxItems), serial: state.serial }
}
