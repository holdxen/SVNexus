import { Spin, Tooltip } from '@douyinfe/semi-ui'
import { cx } from '@linaria/core'
import React, { useState } from 'react'

import { IconButton } from '@/icons/IconButton'
import { flex } from '@/styles/Classes'

import { useHoverTooltip } from './HoverTooltip'

export interface OperationIconProps {
  tooltip?: string
  onClick?: () => void | Promise<any>
  enable: boolean
  children: React.ReactNode

  size?: number
  sync?: boolean
}

function OperationIcon(props: OperationIconProps) {
  const [isLoading, setIsLoading] = useState(false)

  const onClick = () => {
    if (!props.enable || props.onClick === undefined) {
      return
    }
    if (props.sync ?? true) {
      props.onClick()
    } else {
      const call = async () => {
        setIsLoading(true)
        try {
          await props.onClick!()
        } finally {
          setIsLoading(false)
        }
      }
      call()
    }
  }

  const { hover, handler, tooltip } = useHoverTooltip()

  const icon = (
    <IconButton
      onAnimationStart={handler}
      size={props.size}
      onClick={onClick}
      className={cx(!props.enable && 'disabled', props.tooltip !== undefined && tooltip)}
    >
      {props.children}
    </IconButton>
  )
  const loading = (
    <IconButton className="inactive" size={props.size}>
      <Spin spinning></Spin>
    </IconButton>
  )
  const content = isLoading ? loading : icon
  return (
    <Tooltip
      visible={hover}
      trigger="custom"
      content={props.tooltip}
      condition={props.tooltip !== undefined && props.tooltip !== ''}
    >
      {content}
    </Tooltip>
  )
}

export interface OperationBarProps {
  icons: OperationIconProps[]
  size?: number
  className?: string
}

export default function OperationBar(props: OperationBarProps) {
  if (props.size !== undefined) {
    return (
      <div className={cx(flex, props.className)}>
        {props.icons.map((e, index) => {
          return <OperationIcon {...e} size={props.size} key={index}></OperationIcon>
        })}
      </div>
    )
  } else {
    return (
      <div className={cx(flex, props.className)}>
        {props.icons.map((e, index) => {
          return <OperationIcon {...e} key={index}></OperationIcon>
        })}
      </div>
    )
  }
}
