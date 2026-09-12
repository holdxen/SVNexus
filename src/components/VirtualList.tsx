import { css, cx } from '@linaria/core'
import { useVirtualizer, VirtualItem } from '@tanstack/react-virtual'
import type { OverlayScrollbars } from 'overlayscrollbars'
import { OverlayScrollbarsComponent, OverlayScrollbarsComponentRef } from 'overlayscrollbars-react'
import { Key, useCallback, useRef, useState } from 'react'

import { border_box } from '@/styles/Classes'

export interface VirtualListProps {
  itemRender: (row: VirtualItem) => React.ReactNode
  count: number
  itemHeight: number
  className?: string
  getItemKey?: (index: number) => Key
  autoMeasure?: boolean
}

const scrollAreaPadding = css`
  padding-right: var(--scrollbar-size);
`

export default function VirtualList(props: VirtualListProps) {
  'use no memo'
  const scrollRef = useRef<OverlayScrollbarsComponentRef>(null)
  const [isReady, setIsReady] = useState(false)
  const [hasYOverflow, setHasYOverflow] = useState(false)

  const syncOverflow = useCallback((instance: OverlayScrollbars) => {
    setHasYOverflow(instance.state().hasOverflow.y)
  }, [])

  const virtualizer = useVirtualizer({
    count: props.count, // 总共多少项
    getScrollElement: () => {
      if (!isReady) return null
      return scrollRef.current?.osInstance()?.elements().viewport ?? null
    },
    estimateSize: () => props.itemHeight, // 每项预估高度
    overscan: 5, // 上下多渲染几个，减少白屏
    getItemKey: props.getItemKey,
  })

  return (
    <OverlayScrollbarsComponent
      events={{
        initialized(instance) {
          setIsReady(true)
          syncOverflow(instance)
        },
        updated: syncOverflow,
      }}
      className={cx(props.className, border_box, hasYOverflow && scrollAreaPadding)}
      ref={scrollRef}
      options={{ overflow: { x: 'hidden', y: 'scroll' } }}
    >
      <div style={{ height: virtualizer.getTotalSize(), position: 'relative' }}>
        {virtualizer.getVirtualItems().map((virtualRow) => {
          // const item = items[virtualRow.index]
          return (
            <div
              key={virtualRow.key}
              data-index={virtualRow.index}
              ref={props.autoMeasure ? virtualizer.measureElement : undefined}
              style={{
                width: '100%',
                position: 'absolute',
                top: 0,
                left: 0,
                transform: `translateY(${virtualRow.start}px)`,
              }}
            >
              {props.itemRender(virtualRow)}
            </div>
          )
        })}
      </div>
    </OverlayScrollbarsComponent>
  )
}
