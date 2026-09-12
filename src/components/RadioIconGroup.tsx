import { Tooltip } from '@douyinfe/semi-ui'
import { RenderContent } from '@douyinfe/semi-ui/lib/es/tooltip'
import React from 'react'

import { IconButton } from '@/icons/IconButton'

export interface RadioIconGroupProps {
  icons: {
    icon: React.ReactNode
    identity: string | number
    onClick: () => void
    tooltip?: React.ReactNode | RenderContent
  }[]
  active: string | number | undefined
}

export default function RadioIconGroup(props: RadioIconGroupProps) {
  return (
    <>
      {props.icons.map((item) => {
        const icon = (
          <IconButton
            onClick={item.onClick}
            color={props.active === item.identity ? 'var(--semi-color-primary)' : undefined}
            key={item.identity}
          >
            {item.icon}
          </IconButton>
        )
        if (item.tooltip) {
          return (
            <Tooltip key={item.identity} content={item.tooltip}>
              {icon}
            </Tooltip>
          )
        }
        return <>{icon}</>
      })}
    </>
  )
}
