import { IconButton } from '@/icons/IconButton'
import { Tooltip } from '@douyinfe/semi-ui'
import { TooltipProps } from '@douyinfe/semi-ui/lib/es/tooltip'
import { css, cx } from '@linaria/core'
import { useState } from 'react'

const tooltip = css`
  &:hover {
    animation: hover-enter 1ms;
  }

  &:not(:hover) {
    animation: hover-leave 1ms;
  }
`

export interface HoverTooltipProps extends Omit<TooltipProps, 'trigger' | 'visible'> {
  wrapperClassName?: string
}

export function useHoverTooltip() {
  const [hover, setHover] = useState(false)

  const handler = (e: React.AnimationEvent) => {
    if (e.animationName === 'hover-enter') {
      setHover(true)
    }

    if (e.animationName === 'hover-leave') {
      setHover(false)
    }
  }

  return {
    hover,
    handler,
    tooltip,
  }
}

export function IconTooltipButton(
  props: React.HTMLAttributes<HTMLDivElement> & { tooltipConent: string },
) {
  const { hover, tooltip, handler } = useHoverTooltip()
  const { className, tooltipConent, ...others } = props
  return (
    <Tooltip visible={hover} trigger="custom" content={tooltipConent}>
      <IconButton
        {...others}
        onAnimationStart={handler}
        className={cx(tooltip, className)}
      ></IconButton>
    </Tooltip>
  )
}

export default function HoverTooltip(props: HoverTooltipProps) {
  // const [hover, setHover] = useState(false)
  const { children, wrapperClassName, ...others } = props
  const { hover, handler, tooltip } = useHoverTooltip()
  return (
    <Tooltip {...others} trigger="custom" visible={hover}>
      <div onAnimationStart={handler} className={cx(tooltip, wrapperClassName)}>
        {children}
      </div>
    </Tooltip>
  )
}
