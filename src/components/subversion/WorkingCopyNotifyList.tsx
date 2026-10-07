import { css, cx } from '@linaria/core'
import { useVirtualizer } from '@tanstack/react-virtual'
import { ReactNode, useEffect, useRef } from 'react'

import { WorkingCopyNotifyEntry } from '@/components/subversion/workingCopyNotify'
import WorkingCopyNotifyRow from '@/components/subversion/WorkingCopyNotifyRow'
import { useT } from '@/i18n'
import { flex, flex_col, items_center, justify_center, text_center } from '@/styles/Classes'

const ITEM_ESTIMATE = 26
const OVERSCAN = 5
const BOTTOM_THRESHOLD = 8

const scroll = css`
  height: 100%;
  min-height: 0;
  overflow-x: hidden;
  overflow-y: auto;
`

const inner = css`
  position: relative;
  width: 100%;
`

const item = css`
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
`

const empty = css`
  height: 100%;
  color: var(--semi-color-text-3);
  font-size: 12px;
`

export interface WorkingCopyNotifyListProps {
  /** 要展示的条目。归并与去重由调用方（NotifyLog store）负责，这里只做展示 */
  entries: WorkingCopyNotifyEntry[]
  className?: string
  /** 新条目到达且用户已在底部时自动跟随滚动，默认 true */
  autoScroll?: boolean
  /** 行初始是否展开详情，默认 false */
  defaultExpanded?: boolean
  emptyPlaceholder?: ReactNode
}

export default function WorkingCopyNotifyList(props: WorkingCopyNotifyListProps) {
  'use no memo'
  const t = useT()
  const { entries, autoScroll = true } = props

  const scrollRef = useRef<HTMLDivElement>(null)
  const followRef = useRef(true)

  const virtualizer = useVirtualizer({
    count: entries.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => ITEM_ESTIMATE,
    overscan: OVERSCAN,
    getItemKey: (index) => entries[index].key,
    // 内容变高时把当前可见位置锚定住，避免追加条目引起视口跳动
    anchorTo: 'end',
  })

  // 不要直接写 scrollTop：虚拟列表在测量完成后会按它自己记录的偏移再调一次 scrollTo，
  // 把外部写入的滚动位置覆盖掉，必须让它自己滚并记住目标位置。
  // 也不用库内置的 followOnAppend：它只在自己已经位于底部时才跟随，
  // 列表第一次长过视口后就会停止跟随，而进度日志需要的是"除非用户主动上滚，否则一直跟随"。
  useEffect(() => {
    if (entries.length === 0 || !autoScroll || !followRef.current) {
      return
    }
    virtualizer.scrollToEnd()
  }, [entries.length, autoScroll, virtualizer])

  const onScroll = () => {
    const element = scrollRef.current
    if (element === null) {
      return
    }
    const distanceToBottom = element.scrollHeight - element.scrollTop - element.clientHeight
    followRef.current = distanceToBottom < BOTTOM_THRESHOLD
  }

  if (entries.length === 0) {
    return (
      <div className={cx(flex, flex_col, items_center, justify_center, empty, props.className)}>
        <span className={text_center}>
          {props.emptyPlaceholder ?? t('components.notify.empty')}
        </span>
      </div>
    )
  }

  return (
    <div ref={scrollRef} onScroll={onScroll} className={cx(scroll, props.className)}>
      <div className={inner} style={{ height: virtualizer.getTotalSize() }}>
        {virtualizer.getVirtualItems().map((virtualRow) => (
          <div
            key={virtualRow.key}
            data-index={virtualRow.index}
            ref={virtualizer.measureElement}
            className={item}
            style={{ transform: `translateY(${virtualRow.start}px)` }}
          >
            <WorkingCopyNotifyRow
              notify={entries[virtualRow.index].notify}
              count={entries[virtualRow.index].count}
              defaultExpanded={props.defaultExpanded}
            />
          </div>
        ))}
      </div>
    </div>
  )
}
