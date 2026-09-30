import { useState } from 'react'

import { SubversionSSHEvent } from '@/bindings/SubversionSSHEvent'
import PureInput from '@/components/PureInput'
import { replySuccess } from '@/context/Functions'
import { useCurrentModal } from '@/lib/multi-modal'

import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

export type Passphrase = Extract<SubversionSSHEvent, { passphrase: any }>['passphrase']

export function NiceSSHPassphraseDialog(props: Passphrase) {
  const modal = useCurrentModal()
  return (
    <SSHPassphraseDialog
      {...props}
      onOk={() => {
        modal.resolve(true)
        modal.hide()
      }}
      onCancel={() => {
        modal.resolve(false)
        modal.hide()
      }}
      afterClose={modal.remove}
      visible={modal.visible}
    ></SSHPassphraseDialog>
  )
}

export interface SSHPassphraseDialogProps extends Passphrase {
  visible: boolean
  afterClose?: () => void
  onOk?: () => void
  onCancel?: () => void
}

export default function SSHPassphraseDialog(props: SSHPassphraseDialogProps) {
  const [passphrase, setPassphrase] = useState('')

  // 空口令与取消都回复 null：后端按 OpenSSH 的
  // "no passphrase given, try next key" 跳过这把钥匙，连接继续。
  const onOk = async () => {
    await replySuccess(props.id, passphrase === '' ? null : passphrase)
    props.onOk?.()
  }

  const onCancel = async () => {
    await replySuccess(props.id, null)
    props.onCancel?.()
  }

  return (
    <Dialog
      title="Passphrase"
      afterClose={props.afterClose}
      onCancel={onCancel}
      onOk={onOk}
      visible={props.visible}
    >
      <div>
        {props.wrong
          ? 'Incorrect passphrase, please try again'
          : 'This key is protected by a passphrase.'}
      </div>
      {props.username != null && props.username !== '' && <div>Username: {props.username}</div>}
      <div>{props.path}</div>
      <DialogFormItem title="Passphrase:">
        <PureInput mode="password" value={passphrase} onChange={setPassphrase}></PureInput>
      </DialogFormItem>
    </Dialog>
  )
}
