import { Button } from '@douyinfe/semi-ui'
import { Theme, Type } from '@douyinfe/semi-ui/lib/es/button'
import React, { useState } from 'react'

export interface AutoLoadingButtonProps {
  autoLoading: boolean
  onClick: () => Promise<any>
  children?: React.ReactNode
  theme?: Theme

  type?: Type

  className?: string
}

export default function AutoLoadingButton(props: AutoLoadingButtonProps) {
  const [isLoading, setIsLoading] = useState(false)
  const click = props.autoLoading
    ? async () => {
        setIsLoading(true)
        try {
          await props.onClick()
        } finally {
          setIsLoading(false)
        }
      }
    : props.onClick
  return (
    <Button
      className={props.className}
      type={props.type}
      theme={props.theme}
      loading={isLoading}
      onClick={click}
    >
      {props.children}
    </Button>
  )
}
