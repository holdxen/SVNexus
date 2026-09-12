import { Card, Checkbox } from '@douyinfe/semi-ui'
import { cx } from '@linaria/core'
import { Ref, useImperativeHandle, useRef, useState } from 'react'

import { WorkspaceGroup } from '@/bindings/WorkspaceGroup'
import { ScrollArea } from '@/components/ScrollArea'
import { useDatabase } from '@/context/Database'
import { useCurrentModal } from '@/lib/multi-modal'
import { border_box, flex, flex_1, flex_col, min_h_0, p_1, px_1 } from '@/styles/Classes'
import { workspaceItemIdentity } from '@/utils/WorkspaceItem'

import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

interface WorkspaceGroupViewRef {
  exec: (item: string) => Promise<void>
}

function WorkspaceGroupView({
  ref,
  ...props
}: {
  ref: Ref<WorkspaceGroupViewRef | null>
  defaultChecked: boolean
  group: WorkspaceGroup
}) {
  const [checked, setChecked] = useState(props.defaultChecked)
  const updateWorkspaceGroup = useDatabase((state) => state.updateWorkspaceGroup)
  useImperativeHandle(ref, () => ({
    exec: async (item: string) => {
      if (checked) {
        await updateWorkspaceGroup({
          ...props.group,
          members: [item, ...props.group.members.filter((e) => e !== item)],
        })
      } else {
        await updateWorkspaceGroup({
          ...props.group,
          members: props.group.members.filter((e) => e !== item),
        })
      }
    },
  }))
  return (
    <Checkbox
      checked={checked}
      onChange={(e) => setChecked(e.target.checked ?? false)}
      defaultChecked={props.defaultChecked}
    >
      {props.group.name}
    </Checkbox>
  )
}

export const NiceEditWorkspaceItemDialog = (props: { item: string }) => {
  const modal = useCurrentModal()
  return (
    <EditWorkspaceItemDialog
      onCancel={() => {
        modal.resolve(false)
        modal.hide()
      }}
      onOk={() => {
        modal.resolve(true)
        modal.hide()
      }}
      afterClose={modal.remove}
      visible={modal.visible}
      item={props.item}
    ></EditWorkspaceItemDialog>
  )
}

export interface EditWorkspaceItemDialogProps {
  item: string
  visible: boolean
  onOk?: () => void
  onCancel?: () => void
  afterClose?: () => void
}

export default function EditWorkspaceItemDialog(props: EditWorkspaceItemDialogProps) {
  const workspaceItems = useDatabase((state) => state.workspaceItems)
  const workspaceGroups = useDatabase((state) => state.workspaceGroups)

  const item = workspaceItems.find((e) => workspaceItemIdentity(e) === props.item)

  const elements = useRef(new Map<string, WorkspaceGroupViewRef>())

  const onOk = async () => {
    if (!item) {
      return
    }

    const values = Array.from(elements.current.values())

    for (let i of values) {
      await i.exec(workspaceItemIdentity(item))
    }
    props.onOk?.()
  }

  if (!item) {
    return <></>
  }

  return (
    <Dialog
      afterClose={props.afterClose}
      onOk={onOk}
      onCancel={props.onCancel}
      title="Edit"
      visible={props.visible}
    >
      <div className={cx(flex, flex_1, min_h_0)}>
        <DialogFormItem
          title="Groups:"
          titleClassName={cx(px_1)}
          className={cx(min_h_0, flex_1)}
          wrapperClassName={cx(flex, min_h_0)}
        >
          <Card className={cx(flex, flex_1)} bodyStyle={{ flex: 1, display: 'flex' }}>
            <ScrollArea
              className={cx(flex, border_box, p_1, flex_1, min_h_0)}
              contentClassName={cx(flex, flex_col)}
            >
              {workspaceGroups.map((e) => {
                return (
                  <WorkspaceGroupView
                    key={e.identity}
                    ref={(obj) => {
                      if (obj) {
                        elements.current.set(e.identity, obj)
                      } else {
                        elements.current.delete(e.identity)
                      }
                    }}
                    defaultChecked={e.members.indexOf(props.item) >= 0}
                    group={e}
                  ></WorkspaceGroupView>
                )
              })}
            </ScrollArea>
          </Card>
        </DialogFormItem>
      </div>
    </Dialog>
  )
}
