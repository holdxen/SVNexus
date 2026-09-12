import { css, cx } from '@linaria/core'
import type { OverlayScrollbars, OverflowBehavior } from 'overlayscrollbars'
import { OverlayScrollbarsComponent, OverlayScrollbarsComponentRef } from 'overlayscrollbars-react'
import { RefObject, useCallback, useState } from 'react'

import { border_box } from '@/styles/Classes'

interface ScrollAreaProps {
  children: React.ReactNode
  className?: string
  contentClassName?: string
  overflow?: { x: OverflowBehavior; y: OverflowBehavior }
  ref?: RefObject<OverlayScrollbarsComponentRef | null>
}

const scrollArea = css`
  --scrollbar-size: 10px;
`

const scrollAreaPadding = css`
  padding-right: var(--scrollbar-size);
`

/*
.scroll-area {
  --scrollbar-size: 10px;
}

.scroll-area-theme.os-scrollbar {
  --os-size: var(--scrollbar-size);

  --os-handle-bg: rgba(0, 0, 0, 0.25);
  --os-handle-bg-hover: rgba(0, 0, 0, 0.4);
  --os-handle-bg-active: rgba(0, 0, 0, 0.5);

  --os-handle-border-radius: 999px;
}

.scroll-area-content {
  box-sizing: border-box;
}

.scroll-area-content[data-has-y-overflow] {
  padding-right: var(--scrollbar-size);
}
*/

export function ScrollArea({
  ref,
  children,
  className,
  contentClassName,
  overflow,
}: ScrollAreaProps) {
  const [hasYOverflow, setHasYOverflow] = useState(false)

  const syncOverflow = useCallback((instance: OverlayScrollbars) => {
    setHasYOverflow(instance.state().hasOverflow.y)
  }, [])

  return (
    <OverlayScrollbarsComponent
      ref={ref}
      className={cx(scrollArea, className, '____scrollArea____')}
      options={{
        overflow: overflow ?? {
          x: 'hidden',
          y: 'scroll',
        },

        scrollbars: {
          // 先直接使用官方主题
          theme: 'os-theme-dark',

          // 内容溢出才显示
          visibility: 'auto',

          // 一旦需要滚动条，就一直显示
          autoHide: 'never',
        },
      }}
      events={{
        initialized: syncOverflow,
        updated: syncOverflow,
      }}
    >
      <div
        className={cx(
          contentClassName,
          border_box,
          hasYOverflow && scrollAreaPadding,
          '______scrollContent______',
        )}
      >
        {children}
      </div>
    </OverlayScrollbarsComponent>
  )
}
