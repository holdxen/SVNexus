import { IconChevronRight } from '@douyinfe/semi-icons'
import { cx } from '@linaria/core'
import * as Menu from '@radix-ui/react-context-menu'
import React, { HTMLAttributes, useState } from 'react'

import {
  context_menu_content,
  context_menu_item,
  context_menu_separator,
} from '@/styles/Components'

export type ContextMenuItemModel =
  | {
      separator: {
        className?: string
      }
    }
  | {
      item: {
        onSelect?: (event: Event) => void
        content?: React.ReactNode
        disabled?: boolean
        className?: string
      }
    }
  | {
      subitem: {
        content?: React.ReactNode
        className?: string
        disabled?: boolean
        items: ContextMenuItemModel[]
      }
    }

// export interface ContextMenuItemProps {
//   onSelect?: (event: Event) => void
//   identity: string | number
//   isSeparator: boolean
//   className?: string
//   disable?: boolean
//   children?: React.ReactNode
// }

export interface ContextMenuProps extends HTMLAttributes<HTMLSpanElement> {
  menu?: ContextMenuItemModel[]
  asChild?: boolean
}

export function ContextMenu(props: ContextMenuProps) {
  const { menu, ...others } = props
  const [open, setOpen] = useState(false)

  const itemRender = (e: ContextMenuItemModel, index: number) => {
    if ('item' in e) {
      return (
        <Menu.Item
          key={index}
          disabled={e.item.disabled}
          className={cx(context_menu_item, e.item.className)}
          onSelect={e.item.onSelect}
        >
          {e.item.content}
        </Menu.Item>
      )
    }

    if ('separator' in e) {
      return (
        <Menu.Separator
          key={index}
          className={cx(context_menu_separator, e.separator.className)}
        ></Menu.Separator>
      )
    }

    if ('subitem' in e) {
      return (
        <Menu.Sub key={index}>
          <Menu.SubTrigger
            disabled={e.subitem.disabled}
            className={cx(context_menu_item, e.subitem.className)}
          >
            {e.subitem.content}
            <IconChevronRight style={{ marginLeft: 'auto' }} />
          </Menu.SubTrigger>
          <Menu.Portal>
            <Menu.SubContent className={context_menu_content}>
              {e.subitem.items.map((item, index) => itemRender(item, index))}
            </Menu.SubContent>
          </Menu.Portal>
        </Menu.Sub>
      )
    }
    return <React.Fragment key={index}></React.Fragment>
  }

  return (
    <Menu.Root
      open={open}
      onOpenChange={(e) => {
        if (e) {
          if (!props.menu || props.menu.length === 0) {
            return
          }
        }
        setOpen(e)
      }}
    >
      <Menu.Trigger asChild={props.asChild} {...others}></Menu.Trigger>
      <Menu.Portal>
        <Menu.Content className={context_menu_content}>
          {menu?.map((e, index) => itemRender(e, index))}
        </Menu.Content>
      </Menu.Portal>
    </Menu.Root>
  )
}
