import { Button, Card, Checkbox, TextArea } from '@douyinfe/semi-ui'
import { cx } from '@linaria/core'
import { useState } from 'react'

import { LockOptions } from '@/bindings/LockOptions'
import { Subversion, useSubversion } from '@/context/Subversion'
import { useCurrentModal } from '@/lib/multi-modal'
import { list_item, list_item_selected } from '@/styles/Components'
import {
  border_box,
  flex,
  flex_1,
  flex_col,
  flex_row_reverse,
  gap_x_2,
  gap_y_2,
  min_h_0,
  min_w_0,
  overflow_hidden,
  pb_3,
  whitespace_nowrap,
} from '@/styles/Classes'

import { WorkingCopyItem, WorkingCopyPathItemModel } from '../WorkspaceView/WorkingCopyItem'
import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

export function NiceLockDialog(props: { items: WorkingCopyPathItemModel[] }) {
  const modal = useCurrentModal()

  return (
    <LockDialog
      onCancel={() => {
        modal.resolve(false)
        modal.hide()
      }}
      onOK={() => {
        modal.resolve(true)
        modal.hide()
      }}
      afterClose={modal.remove}
      visible={modal.visible}
      items={props.items}
    ></LockDialog>
  )
}

export interface LockDialogProps {
  visible: boolean
  items: WorkingCopyPathItemModel[]

  onOK: () => void
  onCancel: () => void
  afterClose?: () => void
}

export default function LockDialog(props: LockDialogProps) {
  const [selected, setSelected] = useState<string | null>(null)
  const [hasComment, setHasComment] = useState(false)
  const [comment, setComment] = useState('')
  const [stealLock, setStealLock] = useState(false)
  const subversion = useSubversion()
  const [isRunning, setIsRunning] = useState(false)

  const onOk = async () => {
    setIsRunning(true)
    await Subversion.callOnce({
      factory: subversion,
      async call(context) {
        const options: LockOptions = {
          targets: props.items.map((e) => e.path),
          comment: hasComment ? comment : null,
          stealLock,
        }
        await context.lock(options)
        props.onOK()
      },
    })
    setIsRunning(false)
  }

  return (
    <Dialog
      size="medium"
      onCancel={props.onCancel}
      title="Lock"
      afterClose={props.afterClose}
      visible={props.visible}
      footer={<></>}
      closeIconDisabled={isRunning}
      maskClosable={!isRunning}
      closeOnEsc={!isRunning}
    >
      <div className={cx(flex, flex_1, min_h_0, gap_x_2, border_box, pb_3)}>
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
        <div className={cx(flex, flex_col, min_h_0, gap_y_2, min_w_0)}>
          <div className={cx(flex_1, min_h_0)}></div>
          <DialogFormItem title="Options:">
            <Checkbox checked={stealLock} onChange={(e) => setStealLock(e.target.checked ?? false)}>
              Steal lock
            </Checkbox>
          </DialogFormItem>
          <DialogFormItem
            defaultPadding={false}
            title={
              <Checkbox
                checked={hasComment}
                onChange={(e) => setHasComment(e.target.checked ?? false)}
              >
                Comment:
              </Checkbox>
            }
          >
            <TextArea onChange={(e) => setComment(e)} disabled={!hasComment}></TextArea>
          </DialogFormItem>
          <DialogFormItem wrapperClassName={cx(flex_row_reverse, gap_x_2)} title="Action:">
            <Button loading={isRunning} onClick={onOk} type="primary" theme="solid">
              确定
            </Button>
            <Button disabled={isRunning} onClick={props.onCancel}>
              取消
            </Button>
          </DialogFormItem>
        </div>
      </div>
    </Dialog>
  )
}
