import { IconChevronDown, IconTick } from '@douyinfe/semi-icons'
import { Dropdown } from '@douyinfe/semi-ui'
import { css, cx } from '@linaria/core'
import { useState } from 'react'

import {
  border_box,
  flex,
  items_center,
  overflow_hidden,
  px_2,
  py_1,
  visibility_hidden,
} from '@/styles/Classes'

export interface IconSelectItem {
  value: string
  label: string
  icon?: React.ReactNode
  onClick?: () => void
}

export interface IconSelectProps {
  items: IconSelectItem[]
  value?: string
  onChange?: (value: string) => void
  className?: string
  onSelect?: () => void
}

const trigger = css`
  display: inline-flex;
  align-items: center;
  border: 1px solid var(--semi-color-border);
  border-radius: 8px;
  cursor: pointer;
`

const trigger_icon = css`
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 20px;

  & > svg {
    width: 1em;
    height: 1em;
  }
  &:hover {
    background-color: var(--semi-color-fill-0);
  }

  &:active {
    background-color: var(--semi-color-fill-1);
  }
`

const trigger_divider = css`
  width: 1px;
  height: 14px;
  background-color: var(--semi-color-border);
`

const hover = css`
  &:hover {
    background-color: var(--semi-color-fill-0);
  }

  &:active {
    background-color: var(--semi-color-fill-1);
  }
`

const icon = css`
  svg {
    font-size: 20px;
    width: 1em;
    height: 1em;
    border-radius: 6px;
  }
`

export default function IconSelect(props: IconSelectProps) {
  const [visible, setVisible] = useState(false)

  const selected = props.items.find((item) => item.value === props.value)

  const dropDown = (
    <Dropdown.Menu>
      {props.items.map((item, index) => {
        return (
          <Dropdown.Item
            className={cx(icon)}
            icon={item.icon}
            onClick={() => {
              item.onClick?.()
              if (parent === null) {
                return
              }
              props.onChange?.(item.value)
              setVisible(false)
            }}
            key={index}
          >
            {item.label}
            <span
              className={cx(
                border_box,
                css`
                  margin-left: 0.5rem;
                  margin-right: 0.5rem;
                `,
                props.value !== item.value && visibility_hidden,
              )}
            >
              <IconTick></IconTick>
            </span>
          </Dropdown.Item>
        )
      })}
    </Dropdown.Menu>
  )

  return (
    <Dropdown
      trigger="click"
      clickToHide
      visible={visible}
      onVisibleChange={setVisible}
      position="bottomLeft"
      render={dropDown}
    >
      <div
        onClick={(e) => {
          if (visible) {
            setVisible(false)
            e.stopPropagation()
          }
        }}
        className={cx(trigger, overflow_hidden, props.className)}
      >
        {selected?.icon && (
          <div
            onClick={(e) => {
              e.stopPropagation()
              setVisible(false)
              props.onSelect?.()
            }}
            className={cx(trigger_icon, border_box, py_1, px_2)}
          >
            {selected.icon}
          </div>
        )}
        <div className={trigger_divider}></div>
        <div className={cx(hover, flex, items_center, border_box, py_1, px_2)}>
          <IconChevronDown size="large" />
        </div>
      </div>
    </Dropdown>
  )
}
