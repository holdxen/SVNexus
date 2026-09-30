import Descriptions, { Data } from '@douyinfe/semi-ui/lib/es/descriptions'

import { SubversionSSHEvent } from '@/bindings/SubversionSSHEvent'
import { replySuccess } from '@/context/Functions'
import { useCurrentModal } from '@/lib/multi-modal'

import { Dialog } from './Dialog'

export function NiceSSHVerifyNewHostKeyDialog(props: VerifyNewHostKey) {
  const modal = useCurrentModal()
  return (
    <SSHVerifyNewHostKeyDialog
      {...props}
      onOK={() => {
        modal.resolve(true)
        modal.hide()
      }}
      onCancel={() => {
        modal.resolve(false)
        modal.hide()
      }}
      afterClose={modal.remove}
      visible={modal.visible}
    ></SSHVerifyNewHostKeyDialog>
  )
}

export type VerifyNewHostKey = Extract<
  SubversionSSHEvent,
  { verifyNewHostKey: any }
>['verifyNewHostKey']

export interface SSHVerifyNewHostKeyDialogProps extends VerifyNewHostKey {
  visible: boolean
  afterClose?: () => void
  onCancel?: () => void
  onOK?: () => void
}

export default function SSHVerifyNewHostKeyDialog(props: SSHVerifyNewHostKeyDialogProps) {
  const data: Data[] = [
    {
      key: 'Host:',
      value: props.host,
    },
    {
      key: 'IP:',
      value: props.ip,
      hidden: props.ip === null,
    },
    {
      key: 'Key type:',
      value: props.keyType,
    },
    {
      key: 'Fingerprint:',
      value: props.fingerprint,
    },
  ]

  const onOk = async () => {
    await replySuccess(props.id, true)
    props.onOK?.()
  }

  const onCancel = async () => {
    await replySuccess(props.id, false)
    props.onCancel?.()
  }

  return (
    <Dialog afterClose={props.afterClose} onOk={onOk} onCancel={onCancel} visible={props.visible}>
      <Descriptions data={data}></Descriptions>
    </Dialog>
  )
}
