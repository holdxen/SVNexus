import { cx } from '@linaria/core'
import { useVirtualizer, VirtualItem } from '@tanstack/react-virtual'
import { Key, memo, useCallback, useEffect, useLayoutEffect, useRef, useState } from 'react'

import { overflow_y_auto } from '@/styles/Classes'

export interface LazyVirtualListProps {
  itemRender: (row: VirtualItem, isVisible: boolean) => React.ReactNode
  count: number
  itemHeight: number
  className?: string
  getItemKey?: (index: number) => Key
  autoMeasure?: boolean
  containerProps?: Record<string, any>
  renderDeps?: any[]
  startIndex?: number
  onLoadMoreTop?: () => void
  onLoadMoreBottom?: () => void
  endReachedThreshold?: number
}

interface VirtualItemWrapperProps {
  virtualRow: VirtualItem
  isVisible: boolean
  itemRender: (row: VirtualItem, isVisible: boolean) => React.ReactNode
  autoMeasure?: boolean
  measureElement: (el: HTMLElement) => void
  onRef: (index: number, el: HTMLElement | null) => void
  renderDeps?: any[]
}

const VirtualItemWrapper = memo(
  ({
    virtualRow,
    isVisible,
    itemRender,
    autoMeasure,
    measureElement,
    onRef,
  }: VirtualItemWrapperProps) => {
    return (
      <div
        data-index={virtualRow.index}
        ref={(el) => {
          onRef(virtualRow.index, el)
          if (el && autoMeasure) {
            measureElement(el)
          }
        }}
        style={{
          width: '100%',
          position: 'absolute',
          top: 0,
          left: 0,
          transform: `translateY(${virtualRow.start}px)`,
        }}
      >
        {itemRender(virtualRow, isVisible)}
      </div>
    )
  },
  (prev, next) => {
    if (prev.virtualRow.index !== next.virtualRow.index) return false
    if (prev.isVisible !== next.isVisible) return false
    if (prev.autoMeasure !== next.autoMeasure) return false

    const prevDeps = prev.renderDeps
    const nextDeps = next.renderDeps

    if (!prevDeps && !nextDeps) return true
    if (!prevDeps || !nextDeps) return false
    if (prevDeps.length !== nextDeps.length) return false

    for (let i = 0; i < prevDeps.length; i++) {
      if (!Object.is(prevDeps[i], nextDeps[i])) return false
    }

    return true
  },
)

VirtualItemWrapper.displayName = 'VirtualItemWrapper'

export default function LazyVirtualList(props: LazyVirtualListProps) {
  'use no memo'
  const scrollRef = useRef<HTMLDivElement>(null)
  const [isReady, setIsReady] = useState(false)
  const [visibleIndices, setVisibleIndices] = useState<Set<number>>(new Set())
  const itemRefs = useRef<Map<number, HTMLElement>>(new Map())
  const sentinelTopRef = useRef<HTMLDivElement>(null)
  const sentinelBottomRef = useRef<HTMLDivElement>(null)
  const prevStartIndexRef = useRef(props.startIndex ?? 0)
  const loadingTopRef = useRef(false)
  const loadingBottomRef = useRef(false)

  const virtualizer = useVirtualizer({
    count: props.count,
    getScrollElement: () => {
      if (!isReady) return null
      return scrollRef.current
    },
    estimateSize: () => props.itemHeight,
    overscan: 5,
    getItemKey: props.getItemKey,
  })

  useEffect(() => {
    setIsReady(true)
  }, [])

  useEffect(() => {
    if (!isReady || !scrollRef.current) return

    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            const index = Number(entry.target.getAttribute('data-index'))
            setVisibleIndices((prev) => {
              if (prev.has(index)) return prev
              const next = new Set(prev)
              next.add(index)
              return next
            })
          }
        })
      },
      {
        root: scrollRef.current,
        threshold: 0,
      },
    )

    itemRefs.current.forEach((el) => observer.observe(el))

    return () => observer.disconnect()
  }, [isReady, virtualizer.getVirtualItems()])

  useLayoutEffect(() => {
    const currentIndex = props.startIndex ?? 0
    const prevIndex = prevStartIndexRef.current
    if (currentIndex < prevIndex && scrollRef.current) {
      const itemsAdded = prevIndex - currentIndex
      const heightAdded = itemsAdded * props.itemHeight
      scrollRef.current.scrollTop += heightAdded
    }
    prevStartIndexRef.current = currentIndex
  }, [props.startIndex, props.itemHeight])

  useEffect(() => {
    if (!isReady || !scrollRef.current) return
    if (!props.onLoadMoreTop && !props.onLoadMoreBottom) return

    const threshold = props.endReachedThreshold ?? 200

    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (!entry.isIntersecting) return
          const target = entry.target as HTMLElement
          if (target.dataset.sentinel === 'top') {
            if (props.onLoadMoreTop && !loadingTopRef.current) {
              loadingTopRef.current = true
              props.onLoadMoreTop()
              setTimeout(() => {
                loadingTopRef.current = false
              }, 300)
            }
          } else if (target.dataset.sentinel === 'bottom') {
            if (props.onLoadMoreBottom && !loadingBottomRef.current) {
              loadingBottomRef.current = true
              props.onLoadMoreBottom()
              setTimeout(() => {
                loadingBottomRef.current = false
              }, 300)
            }
          }
        })
      },
      {
        root: scrollRef.current,
        rootMargin: `${threshold}px 0px ${threshold}px 0px`,
        threshold: 0,
      },
    )

    if (sentinelTopRef.current && props.onLoadMoreTop) {
      observer.observe(sentinelTopRef.current)
    }
    if (sentinelBottomRef.current && props.onLoadMoreBottom) {
      observer.observe(sentinelBottomRef.current)
    }

    return () => observer.disconnect()
  }, [isReady, props.onLoadMoreTop, props.onLoadMoreBottom, props.endReachedThreshold])

  const handleRef = useCallback((index: number, el: HTMLElement | null) => {
    if (el) {
      itemRefs.current.set(index, el)
    } else {
      itemRefs.current.delete(index)
    }
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
        {props.onLoadMoreTop && (
          <div
            ref={sentinelTopRef}
            data-sentinel="top"
            style={{ position: 'absolute', top: 0, left: 0, width: '100%', height: 1 }}
          />
        )}
        {virtualizer.getVirtualItems().map((virtualRow) => (
          <VirtualItemWrapper
            key={virtualRow.key}
            virtualRow={virtualRow}
            isVisible={visibleIndices.has(virtualRow.index)}
            itemRender={props.itemRender}
            autoMeasure={props.autoMeasure}
            measureElement={virtualizer.measureElement}
            onRef={handleRef}
            renderDeps={props.renderDeps}
          />
        ))}
        {props.onLoadMoreBottom && (
          <div
            ref={sentinelBottomRef}
            data-sentinel="bottom"
            style={{ position: 'absolute', bottom: 0, left: 0, width: '100%', height: 1 }}
          />
        )}
      </div>
    </div>
  )
}
