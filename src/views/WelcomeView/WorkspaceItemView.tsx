import { Typography } from '@douyinfe/semi-ui'
import EditIcon from '@icons/Edit.svg?react'
import { cx } from '@linaria/core'

import { IconButton } from '@/icons/IconButton'
import { flex, flex_1, flex_col, gap_x_2, items_center } from '@/styles/Classes'

export interface WorkspaceItemViewProps {
  name: string
  title1: string
  title2: string
  pin: boolean
  star: boolean
  className?: string
  tags?: React.ReactNode[]
  ref?: React.RefObject<HTMLDivElement | null>
  onEditClick?: () => void
}

export default function WorkspaceItemView({ ref, ...props }: WorkspaceItemViewProps) {
  return (
    <div ref={ref} className={cx(flex, flex_col, props.className)}>
      <div className={cx(flex, items_center, gap_x_2)}>
        <Typography.Title heading={6}>{props.name}</Typography.Title>
        {props.tags?.map((node, index) => {
          return <div key={index}>{node}</div>
        })}
        <div className={cx(flex_1)}></div>
        <IconButton onClick={props.onEditClick} onDoubleClick={(e) => e.stopPropagation()}>
          <EditIcon></EditIcon>
        </IconButton>
      </div>
      <Typography.Text type="tertiary">{props.title1}</Typography.Text>
      <Typography.Text type="tertiary">{props.title2}</Typography.Text>
    </div>
  )
}
