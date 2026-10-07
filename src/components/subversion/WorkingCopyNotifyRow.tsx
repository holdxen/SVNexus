import { IconChevronRight } from '@douyinfe/semi-icons'
import PropertyViewIcon from '@icons/PropertyView.svg?react'
import StatusAddedIcon from '@icons/StatusAdded.svg?react'
import StatusConflictedIcon from '@icons/StatusConflicted.svg?react'
import StatusDeletedIcon from '@icons/StatusDeleted.svg?react'
import StatusIgnoredIcon from '@icons/StatusIgnored.svg?react'
import StatusIncompleteIcon from '@icons/StatusIncomplete.svg?react'
import StatusLockedIcon from '@icons/StatusLocked.svg?react'
import StatusMergedIcon from '@icons/StatusMerged.svg?react'
import StatusModifiedIcon from '@icons/StatusModified.svg?react'
import StatusNormalIcon from '@icons/StatusNormal.svg?react'
import { css, cx } from '@linaria/core'
import dayjs from 'dayjs'
import { Fragment, ReactNode, useState } from 'react'

import type { WorkingCopyNotify } from '@/bindings/WorkingCopyNotify'
import HoverTooltip from '@/components/HoverTooltip'
import FileKindIcon from '@/components/subversion/FileKindIcon'
import { useT } from '@/i18n'
import {
  flex_1,
  gap_x_1,
  grid,
  grid_cols_auto_minmax_0_1fr,
  min_w_0,
  overflow_hidden,
} from '@/styles/Classes'
import { localPath } from '@/utils/Path'

import {
  actionGroup,
  actionLabel,
  errorCodeText,
  isPatchAction,
  lockTimeToMillis,
  NOTIFY_GROUP_COLOR,
  NOTIFY_KIND_LABEL,
  NOTIFY_LOCK_STATE_META,
  NOTIFY_STATE_META,
  NotifyActionGroup,
  notifyDisplayTarget,
} from './workingCopyNotify'

const row = css`
  display: flex;
  flex-direction: column;
  min-width: 0;
  padding: 0 2px;
`

const main = css`
  display: flex;
  align-items: center;
  gap: 6px;
  min-width: 0;
  padding: 3px;
  background: none;
  border: none;
  font: inherit;
  color: inherit;
  text-align: left;
  cursor: pointer;

  &:hover {
    background: var(--semi-color-fill-0);
    border-radius: 3px;
  }
`

const chevron = css`
  flex: none;
  color: var(--semi-color-text-3);
  transition: transform 0.15s ease;
`

const chevronOpen = css`
  transform: rotate(90deg);
`

const groupIcon = css`
  flex: none;
  width: 16px;
  height: 16px;
`

const badge = css`
  flex: none;
  padding: 1px 6px;
  border-radius: 4px;
  background: var(--semi-color-fill-0);
  font-size: 12px;
  line-height: 18px;
  white-space: nowrap;
`

const occurrences = css`
  flex: none;
  color: var(--semi-color-text-3);
  font-size: 12px;
`

const kindIcon = css`
  flex: none;
  width: 16px;
  height: 16px;
`

const fileName = css`
  color: var(--semi-color-text-0);
  white-space: nowrap;
`

const directoryText = css`
  color: var(--semi-color-text-2);
  white-space: nowrap;
`

const trailing = css`
  flex: none;
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--semi-color-text-2);
  font-size: 12px;
  white-space: nowrap;
`

const stateText = css`
  font-size: 12px;
`

const markIcon = css`
  flex: none;
  width: 14px;
  height: 14px;
`

const detail = css`
  display: grid;
  grid-template-columns: max-content minmax(0, 1fr);
  gap: 3px 12px;
  margin: 0 0 6px 22px;
  padding: 6px 10px;
  border: 1px solid var(--semi-color-border);
  border-radius: 6px;
  background: var(--semi-color-fill-0);
  font-size: 12px;
  line-height: 1.5;
`

const detailLabel = css`
  color: var(--semi-color-text-2);
  white-space: nowrap;
`

const detailValue = css`
  min-width: 0;
  color: var(--semi-color-text-0);
  overflow-wrap: anywhere;
`

const detailValueBlock = css`
  white-space: pre-wrap;
`

function GroupIcon({ group, color }: { group: NotifyActionGroup; color: string }) {
  const props = { className: groupIcon, style: { color } }
  switch (group) {
    case 'added':
      return <StatusAddedIcon {...props} />
    case 'deleted':
      return <StatusDeletedIcon {...props} />
    case 'modified':
      return <StatusModifiedIcon {...props} />
    case 'conflict':
      return <StatusConflictedIcon {...props} />
    case 'failed':
      return <StatusIncompleteIcon {...props} />
    case 'skipped':
      return <StatusIgnoredIcon {...props} />
    case 'locked':
      return <StatusLockedIcon {...props} />
    case 'resolve':
      return <StatusMergedIcon {...props} />
    case 'progress':
      return <StatusNormalIcon {...props} />
    default:
      return <></>
  }
}

interface DetailEntry {
  key: string
  label: string
  value: ReactNode
  block?: boolean
}

function buildDetail(notify: WorkingCopyNotify, t: ReturnType<typeof useT>): DetailEntry[] {
  const entries: DetailEntry[] = []

  const push = (key: string, label: string, value: ReactNode, block?: boolean) => {
    entries.push({ key, label, value, block })
  }

  push('path', t('components.notify.detail.path'), notify.path)
  push('action', t('components.notify.detail.action'), notify.action)
  push('kind', t('components.notify.detail.kind'), t(NOTIFY_KIND_LABEL[notify.kind]))

  if (notify.url !== null) {
    push('url', t('components.notify.detail.url'), notify.url)
  }
  if (notify.pathPrefix !== null) {
    push('pathPrefix', t('components.notify.detail.pathPrefix'), notify.pathPrefix)
  }
  if (notify.mimeType !== null) {
    push('mimeType', t('components.notify.detail.mimeType'), notify.mimeType)
  }

  push(
    'contentState',
    t('components.notify.field.contentState'),
    t(NOTIFY_STATE_META[notify.contentState].label),
  )
  push(
    'propertyState',
    t('components.notify.field.propertyState'),
    t(NOTIFY_STATE_META[notify.propertyState].label),
  )
  push(
    'lockState',
    t('components.notify.field.lockState'),
    t(NOTIFY_LOCK_STATE_META[notify.lockState].label),
  )

  if (notify.revision !== null) {
    push('revision', t('components.notify.detail.revision'), notify.revision)
  }
  if (notify.oldRevision !== null) {
    push('oldRevision', t('components.notify.detail.oldRevision'), notify.oldRevision)
  }
  if (notify.changelistName !== null) {
    push('changelistName', t('components.notify.detail.changelistName'), notify.changelistName)
  }
  if (notify.propertyName !== null) {
    push('propertyName', t('components.notify.detail.propertyName'), notify.propertyName)
  }

  if (notify.mergeRange !== null) {
    const { start, end, inheritable } = notify.mergeRange
    push('mergeRange', t('components.notify.detail.mergeRange'), `${start} - ${end}`)
    push(
      'mergeInheritable',
      t('components.notify.detail.mergeInheritable'),
      inheritable ? t('components.notify.yes') : t('components.notify.no'),
    )
  }

  if (notify.lock !== null) {
    const { lock } = notify
    const created = lockTimeToMillis(lock.creationDate)
    const expires = lockTimeToMillis(lock.expirationDate)

    push('lockOwner', t('components.notify.detail.lockOwner'), lock.owner)
    push('lockToken', t('components.notify.detail.lockToken'), lock.token)
    if (lock.comment !== null && lock.comment !== '') {
      push('lockComment', t('components.notify.detail.lockComment'), lock.comment)
    }
    push(
      'lockCreated',
      t('components.notify.detail.lockCreated'),
      created === null
        ? t('components.notify.unknownTime')
        : dayjs(created).format('YYYY-MM-DD HH:mm:ss'),
    )
    push(
      'lockExpires',
      t('components.notify.detail.lockExpires'),
      expires === null
        ? t('components.notify.neverExpires')
        : dayjs(expires).format('YYYY-MM-DD HH:mm:ss'),
    )
  }

  if (notify.err !== null) {
    const { err } = notify
    push('errMsg', t('components.notify.detail.errMsg'), err.msg, true)
    push('errCode', t('components.notify.detail.errCode'), errorCodeText(err.code))
    push('errStatus', t('components.notify.detail.errStatus'), err.status)
    if (err.info.length > 0) {
      const lines = err.info
        .map((info) => `${info.file}:${info.line} ${errorCodeText(info.code)} ${info.msg}`)
        .join('\n')
      push('errInfo', t('components.notify.detail.errInfo'), lines, true)
    }
  }

  const revisionProperties = Object.entries(notify.revisionProperties)
  push(
    'revisionProperties',
    t('components.notify.detail.revisionProperties'),
    revisionProperties.length === 0
      ? t('components.notify.detail.none')
      : revisionProperties.map(([key, value]) => `${key}=${value}`).join('\n'),
    revisionProperties.length > 0,
  )

  if (isPatchAction(notify.action)) {
    push(
      'hunkOriginalStart',
      t('components.notify.detail.hunkOriginalStart'),
      notify.hunkOriginalStart,
    )
    push(
      'hunkOriginalLength',
      t('components.notify.detail.hunkOriginalLength'),
      notify.hunkOriginalLength,
    )
    push(
      'hunkModifiedStart',
      t('components.notify.detail.hunkModifiedStart'),
      notify.hunkModifiedStart,
    )
    push(
      'hunkModifiedLength',
      t('components.notify.detail.hunkModifiedLength'),
      notify.hunkModifiedLength,
    )
    push('hunkMatchedLine', t('components.notify.detail.hunkMatchedLine'), notify.hunkMatchedLine)
    push('hunkFuzz', t('components.notify.detail.hunkFuzz'), notify.hunkFuzz)
  }

  return entries
}

export interface WorkingCopyNotifyRowProps {
  notify: WorkingCopyNotify
  /** 该路径累计收到的通知次数，大于 1 时在动作标签后显示，让"合并过"这件事可见 */
  count?: number
  defaultExpanded?: boolean
  className?: string
}

export default function WorkingCopyNotifyRow(props: WorkingCopyNotifyRowProps) {
  const t = useT()
  const [expanded, setExpanded] = useState(props.defaultExpanded ?? false)

  const { notify } = props
  const group = actionGroup(notify.action)
  const color = NOTIFY_GROUP_COLOR[group]

  const target = notifyDisplayTarget(notify)
  const name = localPath.getFileName(target) ?? target
  const directory = localPath.getParent(target) ?? ''

  const contentState = NOTIFY_STATE_META[notify.contentState]
  const propertyState = NOTIFY_STATE_META[notify.propertyState]
  const lockState = NOTIFY_LOCK_STATE_META[notify.lockState]

  const revisionText =
    notify.oldRevision !== null && notify.revision !== null
      ? t('components.notify.revisionRange', { from: notify.oldRevision, to: notify.revision })
      : notify.revision !== null
        ? t('components.notify.revisionSingle', { revision: notify.revision })
        : null

  return (
    <div className={cx(row, props.className)}>
      <button
        type="button"
        className={main}
        aria-expanded={expanded}
        title={t(expanded ? 'components.notify.collapse' : 'components.notify.expand')}
        onClick={() => setExpanded((value) => !value)}
      >
        <IconChevronRight className={cx(chevron, expanded && chevronOpen)} />
        <GroupIcon group={group} color={color} />
        <span className={badge} style={{ color }}>
          {t(actionLabel(notify.action))}
        </span>
        {(props.count ?? 1) > 1 && (
          <span className={occurrences}>
            {t('components.notify.occurrences', { count: props.count ?? 1 })}
          </span>
        )}
        <FileKindIcon kind={notify.kind} className={kindIcon} />
        <div
          className={cx(
            grid,
            min_w_0,
            flex_1,
            grid_cols_auto_minmax_0_1fr,
            gap_x_1,
            overflow_hidden,
          )}
        >
          <span className={fileName}>{name}</span>
          <span className={directoryText}>{directory}</span>
        </div>
        <div className={trailing}>
          {revisionText !== null && <span>{revisionText}</span>}
          {contentState.color !== null && (
            <HoverTooltip
              content={t('components.notify.stateTooltip', {
                field: t('components.notify.field.contentState'),
                state: t(contentState.label),
              })}
            >
              <span className={stateText} style={{ color: contentState.color }}>
                {t(contentState.label)}
              </span>
            </HoverTooltip>
          )}
          {propertyState.color !== null && (
            <HoverTooltip
              content={t('components.notify.stateTooltip', {
                field: t('components.notify.field.propertyState'),
                state: t(propertyState.label),
              })}
            >
              <span className={stateText} style={{ color: propertyState.color }}>
                <PropertyViewIcon className={markIcon} /> {t(propertyState.label)}
              </span>
            </HoverTooltip>
          )}
          {lockState.color !== null && (
            <HoverTooltip
              content={t('components.notify.stateTooltip', {
                field: t('components.notify.field.lockState'),
                state: t(lockState.label),
              })}
            >
              <span className={stateText} style={{ color: lockState.color }}>
                {t(lockState.label)}
              </span>
            </HoverTooltip>
          )}
          {notify.lock !== null && (
            <HoverTooltip content={notify.lock.owner}>
              <StatusLockedIcon
                className={markIcon}
                style={{ color: 'var(--semi-color-primary)' }}
              />
            </HoverTooltip>
          )}
          {notify.err !== null && (
            <HoverTooltip content={notify.err.msg}>
              <StatusIncompleteIcon
                className={markIcon}
                style={{ color: 'var(--semi-color-danger)' }}
              />
            </HoverTooltip>
          )}
        </div>
      </button>
      {expanded && (
        <div className={detail}>
          {buildDetail(notify, t).map((entry) => (
            <Fragment key={entry.key}>
              <span className={detailLabel}>{entry.label}</span>
              <span className={cx(detailValue, entry.block && detailValueBlock)}>
                {entry.value}
              </span>
            </Fragment>
          ))}
        </div>
      )}
    </div>
  )
}
