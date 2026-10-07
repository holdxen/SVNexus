import { Tag, Toast } from '@douyinfe/semi-ui'
import { cx } from '@linaria/core'
import * as ContextMenu from '@radix-ui/react-context-menu'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { useRef, useState } from 'react'

import PureInput from '@/components/PureInput'
import { useDatabase } from '@/context/Database'
import { useT } from '@/i18n'
import { useCurrentModal, useModal } from '@/lib/multi-modal'
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
import { Dialog } from '../dialogs/Dialog'
import DialogFormItem from '../dialogs/DialogFormItem'

export interface WorkspaceGroupViewProps {
  item: WorkspaceGroup
  onClick?: () => void
  onDelete?: () => void
  className?: string
}

function RenameWorkspaceGroupDialog(props: { identity: string }) {
  const t = useT()
  const modal = useCurrentModal()

  const updateWorkspaceGroup = useDatabase((status) => status.updateWorkspaceGroup)

  const workspaceGroups = useDatabase((status) => status.workspaceGroups)

  const item = workspaceGroups.find((i) => i.identity === props.identity)

  const [name, setName] = useState(item?.name ?? '')

  const onOk = async () => {
    if (item === undefined) {
      Toast.error({
        content: t('shared.error.groupNotExists'),
        stack: true,
      })
      return
    }

    if (workspaceGroups.findIndex((i) => i.identity != props.identity && i.name === name) >= 0) {
      Toast.error({
        content: t('shared.error.nameExists'),
        stack: true,
      })
      return
    }

    const group: WorkspaceGroup = {
      ...item,
      name,
    }

    await updateWorkspaceGroup(group)

    modal.resolve(true)
    modal.hide()
  }

  const input = useRef(null)

  return (
    <Dialog
      initialFocusRef={input}
      title={t('shared.action.renameGroup')}
      visible={modal.visible}
      onOk={onOk}
      onCancel={() => {
        modal.resolve(true)
        modal.hide()
      }}
    >
      <DialogFormItem title={t('shared.field.name')}>
        <PureInput ref={input} value={name} onChange={setName} />
      </DialogFormItem>
    </Dialog>
  )
}

export function WorkspaceGroupView({
  item,
  onClick,
  onDelete,
  className,
}: WorkspaceGroupViewProps) {
  const t = useT()
  const modal = useModal()
  const dialogOpenedFromMenu = useRef(false)
  return (
    <div
      onClick={onClick}
      onContextMenu={onClick}
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
          <ContextMenu.Content
            className={context_menu_content}
            onCloseAutoFocus={(e) => {
              if (dialogOpenedFromMenu.current) {
                e.preventDefault()
                dialogOpenedFromMenu.current = false
              }
            }}
          >
            <ContextMenu.Item
              onSelect={() => {
                writeText(item.name)
              }}
              className={context_menu_item}
            >
              {t('shared.action.copyName')}
            </ContextMenu.Item>

            <ContextMenu.Item
              onSelect={() => {
                dialogOpenedFromMenu.current = true
                modal.show(RenameWorkspaceGroupDialog, {
                  identity: item.identity,
                })
              }}
              className={context_menu_item}
            >
              {t('shared.action.rename')}
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
              {t('shared.action.delete')}
            </ContextMenu.Item>
          </ContextMenu.Content>
        </ContextMenu.Portal>
      </ContextMenu.Root>
    </div>
  )
}
