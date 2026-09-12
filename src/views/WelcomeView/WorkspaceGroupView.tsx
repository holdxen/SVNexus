import { Tag } from '@douyinfe/semi-ui'
import { cx } from '@linaria/core'
import * as ContextMenu from '@radix-ui/react-context-menu'
import {
  font_normal,
  text_12px,
  flex_1,
  px_6px,
  py_7px,
  flex,
  items_center,
  break_all,
  break_word,
  min_w_0,
} from '@/styles/Classes'

import { WorkspaceGroup } from '../../bindings/WorkspaceGroup'
import {
  context_menu_content,
  context_menu_item,
  context_menu_item_danger,
  context_menu_separator,
} from '../../styles/Components'

export interface WorkspaceGroupViewProps {
  item: WorkspaceGroup
  onClick?: () => void
  onDelete?: () => void
  className?: string
}

export function WorkspaceGroupView({
  item,
  onClick,
  onDelete,
  className,
}: WorkspaceGroupViewProps) {
  return (
    <div
      onClick={onClick}
      // onRightClick={onClick}
      style={{ padding: '0px' }}
      className={cx(className, font_normal, text_12px)}
    >
      <ContextMenu.Root>
        <ContextMenu.Trigger className={cx(flex_1, flex, min_w_0)}>
          <div className={cx(flex, min_w_0, items_center, px_6px, py_7px, flex_1)}>
            <div className={cx(break_all, break_word, min_w_0)}>{item.name}</div>
            <div className={cx(flex_1)}></div>
            <Tag color="indigo" shape="circle">
              {item.members.length}
            </Tag>
          </div>
        </ContextMenu.Trigger>
        <ContextMenu.Portal>
          <ContextMenu.Content className={context_menu_content}>
            <ContextMenu.Item className={context_menu_item} onSelect={() => console.log('复制')}>
              Copy name
            </ContextMenu.Item>

            <ContextMenu.Item className={context_menu_item} onSelect={() => console.log('重命名')}>
              Rename
            </ContextMenu.Item>

            <ContextMenu.Separator className={context_menu_separator} />

            <ContextMenu.Item
              className={cx(context_menu_item, context_menu_item_danger)}
              onSelect={() => {
                if (onDelete) {
                  onDelete()
                }
              }}
            >
              Delete
            </ContextMenu.Item>
          </ContextMenu.Content>
        </ContextMenu.Portal>
      </ContextMenu.Root>
    </div>
  )
}
