import { Descriptions, Divider, Tooltip } from '@douyinfe/semi-ui'
import CopyFromIcon from '@icons/CopyFrom.svg?react'
import { css, cx } from '@linaria/core'
import { useState } from 'react'

import { LogEntry } from '@/bindings/LogEntry'
import { RevisionPropertyName } from '@/bindings/RevisionPropertyName'
import ChangeActionIcon from '@/components/subversion/ChangeActionIcon'
import FileKindIcon from '@/components/subversion/FileKindIcon'
import VirtualList from '@/components/VirtualList'
import {
  border_box,
  flex,
  flex_1,
  flex_col,
  gap_x_1,
  items_center,
  m_2,
  min_h_0,
  min_w_0,
  my_1,
  overflow_hidden,
  px_1,
  py_2,
  whitespace_nowrap,
} from '@/styles/Classes'
import { list_item, list_item_selected } from '@/styles/Components'
import simplifyPath from '@/utils/Path'

export interface DetailViewProps {
  entry: LogEntry
  className?: string
  relateTo: string
  root: string
  location: string
}

const changeActionIcon = css`
  svg {
    width: 20px;
    height: 20px;
  }
`

const table = css`
  table {
    min-width: 0px;
  }
  .semi-descriptions-value {
    word-break: break-all;
    word-wrap: break-word;
  }
`

export default function DetailView(props: DetailViewProps) {
  const authorKey: RevisionPropertyName = 'svn:author'
  const dateKey: RevisionPropertyName = 'svn:date'
  const messageKey: RevisionPropertyName = 'svn:log'
  const data = [
    {
      key: 'Author:',
      value: props.entry.revisionProperties?.[authorKey] ?? '',
    },
    {
      key: 'Revision:',
      value: props.entry.revision === null ? '' : String(props.entry.revision),
    },
    {
      key: 'Date:',
      value: props.entry.revisionProperties?.[dateKey] ?? '',
    },
    {
      key: 'Message:',
      value: props.entry.revisionProperties?.[messageKey] ?? '',
    },
    {
      key: 'Root:',
      value: props.root,
    },
    {
      key: 'Location:',
      value: props.location,
    },
  ]

  const [selected, setSelected] = useState<number | null>(null)

  const changes = Object.entries(props.entry.changedPathEntries)
  return (
    <div className={cx(min_h_0, min_w_0, flex, flex_col, props.className)}>
      <VirtualList
        autoMeasure={true}
        getItemKey={(index) => {
          if (index === 0 || index === 1) {
            return index
          }
          return changes[index - 2][0]
        }}
        className={cx(flex_1, border_box, m_2)}
        itemHeight={35}
        itemRender={(row) => {
          let index = row.index

          if (index === 0) {
            return <Descriptions className={cx(flex_1, min_w_0, table)} data={data}></Descriptions>
          } else if (index === 1) {
            return <Divider className={cx(flex_1, border_box, my_1)}></Divider>
          }

          index = index - 2

          let [path, entry] = changes[index]
          path = simplifyPath(props.relateTo, path)
          return (
            <div
              style={{ height: 35 }}
              onClick={() => setSelected(index)}
              className={cx(
                changeActionIcon,
                flex_1,
                flex,
                min_w_0,
                py_2,
                px_1,
                border_box,
                items_center,
                gap_x_1,
                list_item,
                overflow_hidden,
                index === selected && list_item_selected,
              )}
            >
              <Tooltip content={entry.action}>
                <ChangeActionIcon action={entry.action}></ChangeActionIcon>
              </Tooltip>
              <FileKindIcon kind={entry.nodeKind}></FileKindIcon>
              {entry.copyFromPath !== null && entry.copyFromRevision !== null ? (
                <Tooltip content={`Copy from ${entry.copyFromPath}(r${entry.copyFromRevision})`}>
                  <CopyFromIcon></CopyFromIcon>
                </Tooltip>
              ) : (
                <></>
              )}
              <div className={cx(flex_1, min_w_0, whitespace_nowrap)}>{path}</div>
            </div>
          )
        }}
        count={changes.length + 2}
      ></VirtualList>
      {/*<div className={cx(flex_1, flex, flex_col, border_box, m_2, changeActionIcon)}>
        {changes.map(([path, entry], index) => {
          path = simplifyPath(props.relateTo, path)
          return (
            <div
              onClick={() => setSelected(index)}
              key={index}
              className={cx(
                flex,
                min_w_0,
                py_2,
                px_1,
                border_box,
                items_center,
                gap_x_1,
                list_item,
                index === selected && list_item_selected,
              )}
            >
              <Tooltip content={entry.action}>
                <ChangeActionIcon action={entry.action}></ChangeActionIcon>
              </Tooltip>
              <FileKindIcon kind={entry.nodeKind}></FileKindIcon>
              {entry.copyFromPath !== null && entry.copyFromRevision !== null ? (
                <Tooltip content={`Copy from ${entry.copyFromPath}(r${entry.copyFromRevision})`}>
                  <CopyFromIcon></CopyFromIcon>
                </Tooltip>
              ) : (
                <></>
              )}
              <div className={cx(flex_1, min_w_0, break_all, break_word)}>{path}</div>
            </div>
          )
        })}
      </div>*/}
    </div>
  )
}
