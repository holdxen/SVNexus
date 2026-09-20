import { IconRefresh } from '@douyinfe/semi-icons'
import { Button, Spin, Typography } from '@douyinfe/semi-ui'
import { css, cx } from '@linaria/core'
import React from 'react'

import { flex, flex_1, flex_col, gap_y_1, select_none } from '@/styles/Classes'

export type LoadingState = 'none' | 'loading' | 'error'

interface LoadingLayerProps {
  children?: React.ReactNode
  state: LoadingState
  errorMessage?: string
  retry?: () => void
  className?: string
  contentClassName?: string
}

const copyable = css`
  user-select: text;
`

export default function LoadingLayer({
  className,
  state,
  errorMessage,
  retry,
  children,
  contentClassName,
}: LoadingLayerProps) {
  const loading = state === 'loading'
  const error = state === 'error'
  const layer = loading || error

  const errorComponent = error ? (
    <div className={cx(flex, flex_col, gap_y_1)}>
      <Typography.Text className={cx(copyable)} type="danger">
        {errorMessage}
      </Typography.Text>
      <div className={cx(flex)}>
        <div className={cx(flex_1)}></div>
        <Button icon={<IconRefresh></IconRefresh>} onClick={retry}>
          Retry
        </Button>
        <div className={cx(flex_1)}></div>
      </div>
    </div>
  ) : (
    <></>
  )

  const loadingComponent = loading ? <Spin spinning></Spin> : <></>

  return (
    <div style={{ position: 'relative' }} className={cx(flex, className)}>
      {layer && (
        <>
          {/* 居中的 Button */}
          <div
            style={{
              position: 'absolute',
              top: '50%',
              left: '50%',
              transform: 'translate(-50%, -50%)',
              zIndex: 2,
            }}
          >
            {errorComponent}
            {loadingComponent}
          </div>
          {/* 透明遮罩，拦截事件 */}
          <div
            style={{
              position: 'absolute',
              inset: 0,
              zIndex: 1,
            }}
          />
        </>
      )}
      {/* 子内容 — spinning 时降低透明度 */}
      <div
        className={cx(flex_1, flex, contentClassName, layer && select_none)}
        style={{
          opacity: layer ? 0.5 : 1,
          filter: layer ? 'blur(2px)' : 'none',
        }}
      >
        {children}
      </div>
    </div>
  )
}
