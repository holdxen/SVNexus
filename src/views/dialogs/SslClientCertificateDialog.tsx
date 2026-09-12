import { IconFolderStroked } from '@douyinfe/semi-icons'
import { Checkbox } from '@douyinfe/semi-ui'
import { css, cx } from '@linaria/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useState } from 'react'

import { ClientCertificate } from '@/bindings/ClientCertificate'
import PureInput from '@/components/PureInput'
import { replySuccess } from '@/context/Functions'
import { useCurrentModal } from '@/lib/multi-modal'
import { cursor_pointer, hidden } from '@/styles/Classes'

import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

export function NiceSslClientCertificateDialog(props: {
  id: number
  realm: string
  maySave: boolean
}) {
  const modal = useCurrentModal()
  return (
    <SslClientCertificateDialog
      visible={modal.visible}
      onCancel={() => {
        modal.resolve(false)
        modal.hide()
      }}
      onOk={() => {
        modal.resolve(true)
        modal.hide()
      }}
      afterClose={modal.remove}
      {...props}
    ></SslClientCertificateDialog>
  )
}

export interface SslClientCertificateProps {
  visible: boolean
  realm: string
  maySave: boolean
  id: number
  onOk?: () => void
  onCancel?: () => void
  afterClose?: () => void
}

const inputIcon = css`
  .semi-input-append {
    cursor: default;
  }
`

const button = css`
  color: var(--semi-color-primary);
  &:hover {
    color: var(--semi-color-primary-hover);
  }
  &:active {
    color: var(--semi-color-primary-active);
  }
`

export default function SslClientCertificateDialog(props: SslClientCertificateProps) {
  const [file, setFile] = useState('')
  const [save, setSave] = useState(false)

  const selectFile = async () => {
    const file = await open({
      multiple: false,
      directory: false,
    })
    if (file === null) {
      return
    }
    setFile(file)
  }

  const onOk = async () => {
    const value: ClientCertificate = {
      file,
      save,
    }
    // const message: ReplyMessage = {
    //   success: value,
    // }
    await replySuccess(props.id, value)
    props.onOk?.()
  }

  const onCancel = async () => {
    // const message: ReplyMessage = {
    //   success: null,
    // }
    await replySuccess(props.id, null)
    props.onCancel?.()
  }

  return (
    <Dialog
      onOk={onOk}
      onCancel={onCancel}
      afterClose={props.afterClose}
      visible={props.visible}
      title="Client certificate"
    >
      <DialogFormItem title="Realm:">{props.realm}</DialogFormItem>
      <DialogFormItem title="Certificate:">
        <PureInput
          className={inputIcon}
          value={file}
          onChange={(value) => setFile(value)}
          addonAfter={
            <IconFolderStroked onClick={selectFile} className={cx(cursor_pointer, button)} />
          }
        ></PureInput>
      </DialogFormItem>
      <Checkbox
        className={cx(!props.maySave && hidden)}
        checked={save}
        onChange={(e) => setSave(e.target.checked ?? false)}
      >
        Save
      </Checkbox>
    </Dialog>
  )
}
