import { Select } from '@douyinfe/semi-ui'
import { cx } from '@linaria/core'

import { Depth } from '@/bindings/Depth'

export interface DepthSelectProps {
  value?: string
  className?: string
  onChange?: (depth: Depth) => void
  disable?: boolean
}

export default function DepthSelect(props: DepthSelectProps) {
  return (
    <Select
      disabled={props.disable}
      onChange={(e) => props.onChange?.(e as Depth)}
      clickToHide
      value={props.value ?? 'empty'}
      className={cx(props.className)}
    >
      <Select.Option value={'empty'}>Empty</Select.Option>
      <Select.Option value={'files'}>Files</Select.Option>
      <Select.Option value={'immediates'}>Immediates</Select.Option>
      <Select.Option value={'infinity'}>Infinity</Select.Option>
    </Select>
  )
}
