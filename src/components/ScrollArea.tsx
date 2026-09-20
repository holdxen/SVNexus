import { cx } from '@linaria/core'
import { RefObject } from 'react'

interface ScrollAreaProps {
  children: React.ReactNode
  className?: string
  contentClassName?: string
  overflow?: { x: 'hidden' | 'auto' | 'scroll'; y: 'hidden' | 'auto' | 'scroll' }
  ref?: RefObject<HTMLDivElement | null>
}

export function ScrollArea({
  ref,
  children,
  className,
  contentClassName,
  overflow,
}: ScrollAreaProps) {
  const overflowX = overflow?.x ?? 'hidden'
  const overflowY = overflow?.y ?? 'auto'

  return (
    <div ref={ref} className={cx(className)} style={{ overflowX, overflowY }}>
      <div className={contentClassName}>{children}</div>
    </div>
  )
}
