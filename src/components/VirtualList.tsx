import { cx } from '@linaria/core'
import { useVirtualizer, VirtualItem } from '@tanstack/react-virtual'
import { Key, useEffect, useRef, useState } from 'react'

import { overflow_y_auto } from '@/styles/Classes'

export interface VirtualListProps {
  itemRender: (row: VirtualItem) => React.ReactNode
  count: number
  itemHeight: number
  className?: string
  getItemKey?: (index: number) => Key
  autoMeasure?: boolean
  // 附加到内部列表容器上的属性（例如 headless-tree 的 getContainerProps）
  containerProps?: Record<string, any>
}

export default function VirtualList(props: VirtualListProps) {
  'use no memo'
  const scrollRef = useRef<HTMLDivElement>(null)
  const [isReady, setIsReady] = useState(false)

  const virtualizer = useVirtualizer({
    count: props.count, // 总共多少项
    getScrollElement: () => {
      if (!isReady) return null
      return scrollRef.current
    },
    estimateSize: () => props.itemHeight, // 每项预估高度
    overscan: 5, // 上下多渲染几个，减少白屏
    getItemKey: props.getItemKey,
  })

  useEffect(() => {
    setIsReady(true)
  }, [])

  return (
    <div
      ref={scrollRef}
      className={cx(props.className, overflow_y_auto)}
      style={{ overflowX: 'hidden' }}
    >
      <div
        {...props.containerProps}
        style={{
          height: virtualizer.getTotalSize(),
          position: 'relative',
          ...props.containerProps?.style,
        }}
      >
        {virtualizer.getVirtualItems().map((virtualRow) => {
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
    </div>
  )
}
