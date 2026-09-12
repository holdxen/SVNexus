import { IconChevronDown, IconTick } from '@douyinfe/semi-icons'
import { Dropdown } from '@douyinfe/semi-ui'
import { css, cx } from '@linaria/core'
import { useState } from 'react'

import { cursor_pointer, flex, flex_1, items_center } from '@/styles/Classes'

export interface IconSelectItem {
  value: string
  label: string
  icon?: React.ReactNode
}

export interface IconSelectProps {
  items: IconSelectItem[]
  value?: string
  onChange?: (value: string) => void
  className?: string
}

const trigger = css`
  display: inline-flex;
  align-items: center;
  column-gap: 8px;
  padding: 5px 8px;
  border: 1px solid var(--semi-color-border);
  border-radius: 8px;
  background-color: var(--semi-color-bg-0);
  cursor: pointer;

  &:hover {
    background-color: var(--semi-color-fill-0);
  }

  &:active {
    background-color: var(--semi-color-fill-1);
  }
`

const trigger_icon = css`
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;

  & > svg,
  & > img {
    width: 100%;
    height: 100%;
  }
`

const trigger_divider = css`
  width: 1px;
  height: 14px;
  background-color: var(--semi-color-border);
`

const trigger_chevron = css`
  color: var(--semi-color-text-2);
`

const panel = css`
  padding: 6px;
  min-width: 200px;
`

const row = css`
  padding: 8px 10px;
  border-radius: 8px;
  column-gap: 12px;

  &:hover {
    background-color: var(--semi-color-fill-1);
  }

  &:active {
    background-color: var(--semi-color-fill-2);
  }
`

const row_icon = css`
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  width: 28px;
  height: 28px;
  border-radius: 6px;

  & > svg,
  & > img {
    width: 100%;
    height: 100%;
    border-radius: 6px;
  }
`

const row_label = css`
  font-size: 15px;
  color: var(--semi-color-text-0);
`

const row_check = css`
  flex-shrink: 0;
  color: var(--semi-color-text-0);
`

export default function IconSelect(props: IconSelectProps) {
  const [visible, setVisible] = useState(false)

  const selected = props.items.find((item) => item.value === props.value)

  return (
    <Dropdown
      trigger="click"
      clickToHide
      visible={visible}
      onVisibleChange={setVisible}
      position="bottomLeft"
      contentClassName={panel}
      render={
        <div>
          {props.items.map((item) => (
            <div
              key={item.value}
              className={cx(flex, items_center, cursor_pointer, row)}
              onClick={() => {
                props.onChange?.(item.value)
                setVisible(false)
              }}
            >
              {item.icon && <div className={row_icon}>{item.icon}</div>}
              <span className={cx(flex_1, row_label)}>{item.label}</span>
              {item.value === props.value && <IconTick className={row_check} />}
            </div>
          ))}
        </div>
      }
    >
      <div className={cx(trigger, props.className)}>
        {selected?.icon && <div className={trigger_icon}>{selected.icon}</div>}
        <div className={trigger_divider}></div>
        <IconChevronDown className={trigger_chevron} />
      </div>
    </Dropdown>
  )
}
