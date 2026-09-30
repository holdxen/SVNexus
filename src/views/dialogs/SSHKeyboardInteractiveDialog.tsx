import { useState } from 'react'

import { SubversionSSHEvent } from '@/bindings/SubversionSSHEvent'
import PureInput from '@/components/PureInput'
import { replyFailure, replySuccess } from '@/context/Functions'
import { useCurrentModal } from '@/lib/multi-modal'

import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

export type KeyboardInteractive = Extract<
  SubversionSSHEvent,
  { keyboardInteractive: any }
>['keyboardInteractive']

export function NiceSSHKeyboardInteractiveDialog(props: KeyboardInteractive) {
  const modal = useCurrentModal()
  return (
    <SSHKeyboardInteractiveDialog
      {...props}
      onOk={() => {
        modal.resolve(true)
        modal.hide()
      }}
      onCancel={() => {
        modal.resolve(false)
        modal.hide()
      }}
      visible={modal.visible}
    ></SSHKeyboardInteractiveDialog>
  )
}

export interface SSHKeyboardInteractiveDialogProps extends KeyboardInteractive {
  visible: boolean
  afterClose?: () => void
  onOk?: () => void
  onCancel?: () => void
}

export default function SSHKeyboardInteractiveDialog(props: SSHKeyboardInteractiveDialogProps) {
  const [answers, setAnsers] = useState(Array(props.prompts.length).fill(''))
  const onOk = async () => {
    await replySuccess(props.id, answers)
    props.onOk?.()
  }
  const onCancel = async () => {
    await replyFailure(props.id, {
      unexpectedError: { detail: 'Cancelled by user' },
    })
    props.onCancel?.()
  }
  return (
    <Dialog afterClose={props.afterClose} onCancel={onCancel} onOk={onOk} visible={props.visible}>
      {props.prompts.map(([name, echo], index) => {
        return (
          <DialogFormItem key={index} title={name}>
            <PureInput
              mode={echo ? undefined : 'password'}
              value={answers[index]}
              onChange={(value) => {
                setAnsers((v) => v.map((e, i) => (i === index ? value : e)))
              }}
            ></PureInput>
          </DialogFormItem>
        )
      })}
    </Dialog>
  )
}
