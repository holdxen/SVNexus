import { Checkbox, TextArea, Toast, Input } from '@douyinfe/semi-ui'
import { cx } from '@linaria/core'
import { useState } from 'react'

import { MkdirOptions } from '@/bindings/MkdirOptions'
import { Subversion, useSubversion } from '@/context/Subversion'
import { useT } from '@/i18n'
import { useCurrentModal } from '@/lib/multi-modal'
import {
  border_box,
  break_all,
  break_word,
  flex,
  flex_1,
  flex_col,
  gap_x_4,
  gap_y_1,
  gap_y_2,
  hidden,
  min_h_0,
} from '@/styles/Classes'

import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

export interface MkdirDialogProps {
  afterClose?: () => void
  onCancel?: () => void
  onOK?: () => void
  visible: boolean

  parent: string

  needCommitMessage: boolean
}

export function NiceMkdirDialog(props: { parent: string; needCommitMessage: boolean }) {
  const modal = useCurrentModal()

  return (
    <MkdirDialog
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
      parent={props.parent}
      needCommitMessage={props.needCommitMessage}
    ></MkdirDialog>
  )
}

export default function MkdirDialog(props: MkdirDialogProps) {
  const t = useT()
  const [commitMessage, setCommitMessage] = useState('')
  const [makeParents, setMakeParents] = useState(false)
  const [name, setName] = useState('')
  const subversion = useSubversion()

  const onOk = async () => {
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
        if (name === '') {
          Toast.error({
            content: t('shared.error.nameRequired'),
            stack: true,
          })
          return
        }
        const options: MkdirOptions = {
          paths: [`${props.parent}/${name}`],
          makeParents,
          revisionPropertyTable: null,
          commitMessage,
        }
        await context.mkdir(options)
        props.onOK?.()
      },
    })
  }

  return (
    <Dialog
      afterClose={props.afterClose}
      onCancel={props.onCancel}
      onOk={onOk}
      title={t('shared.action.mkdir')}
      visible={props.visible}
    >
      <div className={cx(flex, flex_1, flex_col, min_h_0, gap_x_4, border_box, gap_y_2)}>
        {/*<span>{`Target: ${props.parent}`}</span>*/}
        <DialogFormItem title={t('shared.field.target')}>
          <span className={cx(break_all, break_word)}>{props.parent}</span>
        </DialogFormItem>
        <DialogFormItem title={t('shared.field.name')}>
          <Input className={cx(flex_1)} value={name} onChange={setName}></Input>
        </DialogFormItem>
        <div className={cx(flex, flex_col, min_h_0, gap_y_1)}>
          <div className={cx(flex_1, min_h_0)}></div>
          <DialogFormItem
            className={cx(!props.needCommitMessage && hidden)}
            title={t('shared.field.commitMessage')}
          >
            <TextArea value={commitMessage} onChange={(e) => setCommitMessage(e)}></TextArea>
          </DialogFormItem>
          <DialogFormItem title={t('shared.field.options')}>
            <div className={cx(flex_1, flex, flex_col)}>
              <Checkbox
                checked={makeParents}
                onChange={(e) => setMakeParents(e.target.checked ?? false)}
              >
                {t('shared.option.makeParents')}
              </Checkbox>
            </div>
          </DialogFormItem>
        </div>
      </div>
    </Dialog>
  )
}
