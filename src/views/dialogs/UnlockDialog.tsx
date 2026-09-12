import { Card, Checkbox } from '@douyinfe/semi-ui'
import { cx } from '@linaria/core'
import { useState } from 'react'

import { UnlockOptions } from '@/bindings/UnlockOptions'
import { Subversion, useSubversion } from '@/context/Subversion'
import { useCurrentModal } from '@/lib/multi-modal'
import { list_item, list_item_selected } from '@/styles/Components'
import {
  flex,
  flex_1,
  flex_col,
  gap_y_2,
  min_h_0,
  min_w_0,
  overflow_hidden,
  whitespace_nowrap,
} from '@/styles/Classes'

import { WorkingCopyItem, WorkingCopyPathItemModel } from '../WorkspaceView/WorkingCopyItem'
import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

export function NiceUnlockDialog(props: { items: WorkingCopyPathItemModel[] }) {
  const modal = useCurrentModal()
  return (
    <UnlockDialog
      afterClose={modal.remove}
      onOk={() => {
        modal.resolve(true)
        modal.hide()
      }}
      onCancel={() => {
        modal.resolve(false)
        modal.hide()
      }}
      items={props.items}
      visible={modal.visible}
    ></UnlockDialog>
  )
}

export interface UnlockDialogProps {
  visible: boolean
  items: WorkingCopyPathItemModel[]

  onCancel: () => void
  onOk: () => void
  afterClose: () => void
}

export default function UnlockDialog(props: UnlockDialogProps) {
  const [selected, setSelected] = useState<string | null>(null)
  const [breakLock, setBreakLock] = useState(false)

  const subversion = useSubversion()

  const onOk = async () => {
    await Subversion.callOnce({
      factory: subversion,
      async call(context) {
        const options: UnlockOptions = {
          targets: props.items.map((e) => e.path),
          breakLock,
        }
        await context.unlock(options)
        props.onOk()
      },
    })
  }

  return (
    <Dialog
      onOk={onOk}
      onCancel={props.onCancel}
      title="Lock"
      afterClose={props.afterClose}
      visible={props.visible}
    >
      <div className={cx(flex, flex_col, flex_1, min_h_0, gap_y_2)}>
        <DialogFormItem
          className={cx(flex_1, min_h_0)}
          wrapperClassName={cx(min_h_0)}
          title="Targets:"
        >
          <Card
            className={cx(min_h_0, flex_1, flex)}
            headerStyle={{ padding: '0.25rem' }}
            bodyStyle={{ padding: '0.5rem', minHeight: '0px', overflowY: 'auto', flex: '1' }}
          >
            <div className={cx(min_w_0, min_h_0)}>
              {props.items.map((item) => {
                return (
                  <WorkingCopyItem
                    key={item.path}
                    showRelativeDirectory={true}
                    onClick={() => {
                      setSelected(item.path)
                    }}
                    {...item}
                    className={cx(
                      list_item,
                      overflow_hidden,
                      whitespace_nowrap,
                      selected === item.path && list_item_selected,
                    )}
                  ></WorkingCopyItem>
                )
              })}
            </div>
          </Card>
        </DialogFormItem>
        <DialogFormItem title="Options:">
          <Checkbox checked={breakLock} onChange={(e) => setBreakLock(e.target.checked ?? false)}>
            Break lock
          </Checkbox>
        </DialogFormItem>
      </div>
    </Dialog>
  )
}
