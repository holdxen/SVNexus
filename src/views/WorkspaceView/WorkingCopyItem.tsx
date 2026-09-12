import { Spin, Tooltip } from '@douyinfe/semi-ui'
import { Typography } from '@douyinfe/semi-ui'
import { css, cx } from '@linaria/core'
import {
  flex,
  items_center,
  hidden,
  grid,
  min_w_0,
  flex_1,
  grid_cols_auto_minmax_0_1fr,
  gap_x_1,
  px_2px,
  py_5px,
  overflow_hidden,
} from '@/styles/Classes'

import FileKindIcon from '@/components/subversion/FileKindIcon'
import { localPath } from '@/utils/Path'

import { NodeKind } from '../../bindings/NodeKind'
import { StatusEntry } from '../../bindings/StatusEntry'
import { WorkingCopyStatus } from '../../bindings/WorkingCopyStatus'
import Container from '../../components/Container'
import ChangelistIcon from '../../icons/Changelist.svg?react'
import StarIcon from '../../icons/Star.svg?react'
import StatusAddedIcon from '../../icons/StatusAdded.svg?react'
import StatusConflictedIcon from '../../icons/StatusConflicted.svg?react'
import StatusDeletedIcon from '../../icons/StatusDeleted.svg?react'
import StatusExternalIcon from '../../icons/StatusExternal.svg?react'
import StatusIgnoredIcon from '../../icons/StatusIgnored.svg?react'
import StatusIncompleteIcon from '../../icons/StatusIncomplete.svg?react'
import StatusLockedIcon from '../../icons/StatusLocked.svg?react'
import StatusMergedIcon from '../../icons/StatusMerged.svg?react'
import StatusMissingIcon from '../../icons/StatusMissing.svg?react'
import StatusModifiedIcon from '../../icons/StatusModified.svg?react'
import StatusNormalIcon from '../../icons/StatusNormal.svg?react'
import StatusObstructedIcon from '../../icons/StatusObstructed.svg?react'
import StatusReplacedIcon from '../../icons/StatusReplaced.svg?react'
import StatusUnversionedIcon from '../../icons/StatusUnversioned.svg?react'
import CopyFromIcon from '../../icons/CopyFrom.svg?react'
import ExchangeIcon from '../../icons/Exchange.svg?react'
import HoverTooltip from '@/components/HoverTooltip'

const { Text } = Typography

export interface WorkingCopyItemProps extends WorkingCopyItemModel {
  className?: string
  onClick?: React.MouseEventHandler<HTMLDivElement>
  onContextMenu?: React.MouseEventHandler<HTMLDivElement>
  showRelativeDirectory?: boolean
}

export interface WorkingCopyPathItemModel extends WorkingCopyItemModel {
  path: string
}

export interface WorkingCopyItemModel {
  kind: NodeKind
  textToolTip: string
  fileName: string
  relativeDirectory: string
  isLocked: boolean
  isLoading: boolean
  status: WorkingCopyStatus
  changelist: string | null
  className?: string
  switched: boolean
  copied: boolean
}

function StatusIcon({ status, ...any }: { status: WorkingCopyStatus } & any) {
  switch (status) {
    case 'none':
    case 'normal':
      return <StatusNormalIcon {...any} />
    case 'added':
      return <StatusAddedIcon {...any} />
    case 'conflicted':
      return <StatusConflictedIcon {...any} />
    case 'deleted':
      return <StatusDeletedIcon {...any} />
    case 'external':
      return <StatusExternalIcon {...any} />
    case 'ignored':
      return <StatusIgnoredIcon {...any} />
    case 'incomplete':
      return <StatusIncompleteIcon {...any} />
    case 'merged':
      return <StatusMergedIcon {...any} />
    case 'missing':
      return <StatusMissingIcon {...any} />
    case 'modified':
      return <StatusModifiedIcon {...any} />
    case 'obstructed':
      return <StatusObstructedIcon {...any} />
    case 'replaced':
      return <StatusReplacedIcon {...any} />
    case 'unversioned':
      return <StatusUnversionedIcon {...any} />
  }
}

function stripPrefix(str: string, prefix: string): string {
  return str.startsWith(prefix) ? str.slice(prefix.length) : str
}

export function fromStatusEntry(
  entry: StatusEntry,
  absolute?: boolean,
  relateTo?: string,
): WorkingCopyItemModel {
  let fileName = ''
  if (absolute) {
    fileName = entry.path
  } else {
    fileName = entry.path === relateTo ? '/' : (localPath.getFileName(entry.path) ?? '')
  }

  let relativeDirectory = ''

  if (!absolute) {
    if (relateTo === undefined || relateTo === '') {
      relativeDirectory = localPath.getParent(entry.path) ?? ''
    } else if (entry.path === relateTo) {
      relativeDirectory = ''
    } else {
      let path = stripPrefix(entry.path, relateTo)
      relativeDirectory = localPath.getParent(path) ?? ''
    }
  }

  return {
    kind: entry.nodeKind,
    textToolTip: entry.path,
    fileName: fileName,
    relativeDirectory: relativeDirectory,
    isLocked: entry.lock !== null,
    status: entry.nodeStatus,
    changelist: entry.changelist,
    isLoading: false,
    switched: entry.switched,
    copied: entry.copied
  }
}

export const WorkingCopyIconStyle = css`
  svg {
    width: 20px;
    height: 20px;
  }
`


export function WorkingCopyItem(props: WorkingCopyItemProps) {
  // grid grid-cols-[auto_auto_minmax(0,1fr)_auto_auto_auto]
  // grid grid-cols-[auto_minmax(0,1fr)]

  return (
    <div
      onClick={props.onClick}
      onContextMenu={props.onContextMenu}
      className={cx(flex, items_center, min_w_0, py_5px, px_2px, WorkingCopyIconStyle, '_____________workitem', props.className)}
    >
      <Container>
        <Spin
          style={{ display: props.isLoading ? 'block' : 'none' }}
          spinning={props.isLoading}
        ></Spin>
        <Tooltip content={props.status}>
          <div className={cx(props.isLoading && hidden, flex, items_center)}>
            <StatusIcon status={props.status}></StatusIcon>
          </div>
        </Tooltip>
      </Container>
      <FileKindIcon kind={props.kind}></FileKindIcon>
      <div className={cx(grid, min_w_0, flex_1, grid_cols_auto_minmax_0_1fr, gap_x_1, overflow_hidden)}>
        <Text delete={props.status === 'deleted'}>{props.fileName}</Text>
        <Text type="quaternary" className={cx(!props.showRelativeDirectory && hidden)}>
          {props.relativeDirectory}
        </Text>
      </div>
      <HoverTooltip wrapperClassName={cx(flex, items_center, !props.copied && hidden)} content="Copied">
        <CopyFromIcon></CopyFromIcon>
      </HoverTooltip>
      <HoverTooltip content={props.changelist ?? ''} wrapperClassName={cx(flex, items_center, props.changelist === null && hidden)}>
        <ChangelistIcon></ChangelistIcon>
      </HoverTooltip>
      <StarIcon className={hidden}></StarIcon>
      <HoverTooltip wrapperClassName={cx(flex, items_center, !props.switched && hidden)} content={'Switched'}>
        <ExchangeIcon></ExchangeIcon>
      </HoverTooltip>
      <StatusLockedIcon className={cx(!props.isLocked && hidden)}></StatusLockedIcon>
    </div>
  )
}
