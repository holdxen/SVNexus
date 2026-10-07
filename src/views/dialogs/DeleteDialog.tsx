import { Card, Checkbox, TextArea, Button, Toast } from '@douyinfe/semi-ui'
import { cx } from '@linaria/core'
import { useState } from 'react'

import { DeleteOptions } from '@/bindings/DeleteOptions'
import { Subversion, useSubversion } from '@/context/Subversion'
import { useT } from '@/i18n'
import { useCurrentModal } from '@/lib/multi-modal'
import {
  border_box,
  flex,
  flex_1,
  flex_col,
  flex_row_reverse,
  gap_x_2,
  gap_x_4,
  gap_y_1,
  gap_y_2,
  hidden,
  min_h_0,
  min_w_0,
  overflow_hidden,
  pb_3,
  whitespace_nowrap,
} from '@/styles/Classes'
import { list_item, list_item_selected } from '@/styles/Components'

import { WorkingCopyItem, WorkingCopyPathItemModel } from '../WorkspaceView/WorkingCopyItem'
import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

export interface DeleteDialogProps {
  afterClose?: () => void
  onCancel?: () => void
  onOK?: () => void
  visible: boolean
  items: WorkingCopyPathItemModel[]

  needCommitMessage: boolean
}

export function NiceDeleteDialog(props: {
  items: WorkingCopyPathItemModel[]
  needCommitMessage: boolean
}) {
  const modal = useCurrentModal()

  return (
    <DeleteDialog
      afterClose={modal.remove}
      visible={modal.visible}
      onCancel={() => {
        modal.resolve(false)
        modal.hide()
      }}
      onOK={() => {
        modal.resolve(true)
        modal.hide()
      }}
      items={props.items}
      needCommitMessage={props.needCommitMessage}
    ></DeleteDialog>
  )
}

export default function DeleteDialog(props: DeleteDialogProps) {
  const t = useT()
  const [selected, setSelected] = useState<string | null>(null)
  const [commitMessage, setCommitMessage] = useState('')
  const [keepLocal, setKeepLocal] = useState(false)
  const [force, setForce] = useState(false)
  const subversion = useSubversion()
  const [isRunning, setIsRunning] = useState(false)

  const onOk = async () => {
    setIsRunning(true)
    await Subversion.callOnce({
      factory: subversion,
      async call(context) {
        if (props.needCommitMessage && commitMessage === '') {
          Toast.error({
            content: t('shared.error.commitMessageRequired'),
            stack: true,
          })
          return
        }
        const options: DeleteOptions = {
          path: props.items.map((e) => e.path),
          force,
          keepLocal,
          revisionPropertyTable: null,
          commitMessage,
        }

        await context.delete(options)
        props.onOK?.()
      },
    })
    setIsRunning(false)
  }

  return (
    <Dialog
      size="medium"
      afterClose={props.afterClose}
      onCancel={props.onCancel}
      footer={<></>}
      title={t('shared.action.delete')}
      visible={props.visible}
      closeIconDisabled={isRunning}
      maskClosable={!isRunning}
      closeOnEsc={!isRunning}
    >
      <div className={cx(flex, flex_1, min_h_0, gap_x_4, border_box, pb_3)}>
        <div className={cx(flex, flex_col, gap_y_2, min_h_0, flex_1)}>
          <Card
            style={{ flex: '1 1 300px' }}
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
        </div>
        <div className={cx(flex, flex_col, min_h_0, gap_y_1)}>
          <div className={cx(flex_1, min_h_0)}></div>
          <DialogFormItem title={t('shared.field.options')}>
            <div className={cx(flex_1, flex, flex_col)}>
              <Checkbox
                checked={keepLocal}
                onChange={(e) => setKeepLocal(e.target.checked ?? false)}
              >
                {t('shared.option.keepLocal')}
              </Checkbox>
              <Checkbox checked={force} onChange={(e) => setForce(e.target.checked ?? false)}>
                {t('shared.option.force')}
              </Checkbox>
            </div>
          </DialogFormItem>
          <DialogFormItem
            className={cx(!props.needCommitMessage && hidden)}
            title={t('shared.field.commitMessage')}
          >
            <TextArea value={commitMessage} onChange={(e) => setCommitMessage(e)}></TextArea>
          </DialogFormItem>
          <DialogFormItem title={t('shared.field.actions')}>
            <div className={cx(flex, flex_1, flex_row_reverse, gap_x_2)}>
              <Button loading={isRunning} onClick={onOk} theme="solid" type="primary">
                {t('shared.action.delete')}
              </Button>
              <Button disabled={isRunning} onClick={props.onCancel}>
                {t('common.cancel')}
              </Button>
            </div>
          </DialogFormItem>
        </div>
      </div>
    </Dialog>
  )
}
