import { cx } from '@linaria/core'
import React from 'react'

import { flex, flex_1, flex_col, px_2px } from '@/styles/Classes'

export default function DialogFormItem({
  title,
  children,
  className,
  wrapperClassName,
  titleClassName,
  defaultPadding,
}: {
  title: React.ReactNode
  children?: React.ReactNode
  className?: string
  wrapperClassName?: string
  titleClassName?: string
  defaultPadding?: boolean
}) {
  return (
    <div className={cx(flex, flex_col, className)}>
      <div
        className={cx((defaultPadding ?? false) && px_2px, titleClassName)}
        style={{ color: 'rgba(var(--semi-grey-9), 1)' }}
      >
        {title}
      </div>
      <div className={cx(flex, flex_1, wrapperClassName)}>{children}</div>
    </div>
  )
}
