import { Radio, RadioGroup, Toast } from '@douyinfe/semi-ui'
import { cx } from '@linaria/core'
import { useState } from 'react'

import { SSHAuthetication } from '@/bindings/SSHAuthetication'
import { SubversionSSHEvent } from '@/bindings/SubversionSSHEvent'
import PathInput from '@/components/PathInput'
import PureInput from '@/components/PureInput'
import { replyFailure, replySuccess } from '@/context/Functions'
import { useCurrentModal } from '@/lib/multi-modal'
import { flex, flex_1, flex_col, hidden } from '@/styles/Classes'

import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

export type Authenticate = Extract<SubversionSSHEvent, { authenticate: any }>['authenticate']

export function NiceSSHAuthenticateDialog(props: Authenticate) {
  const modal = useCurrentModal()
  return (
    <SSHAuthenticateDialog
      {...props}
      visible={modal.visible}
      afterClose={modal.remove}
      onOk={() => {
        modal.resolve(true)
        modal.hide()
      }}
      onCancel={() => {
        modal.resolve(false)
        modal.hide()
      }}
    ></SSHAuthenticateDialog>
  )
}

export interface SSHAuthenticateDialogProps extends Authenticate {
  visible: boolean
  afterClose?: () => void
  onOk?: () => void
  onCancel?: () => void
}

type AuthenticateType = 'password' | 'publicKey' | 'keyboardInteractive'

export default function SSHAuthenticateDialog(props: SSHAuthenticateDialogProps) {
  let t: AuthenticateType | undefined = undefined

  if (props.password) {
    t = 'password'
  } else if (props.publicKey) {
    t = 'publicKey'
  } else if (props.keyboardInteractive) {
    t = 'keyboardInteractive'
  }

  const [authenticateType, setAuthenticateType] = useState<AuthenticateType | undefined>(t)
  const [username, setUsername] = useState(props.username ?? '')
  const [password, setPassword] = useState('')
  const [passphrase, setPassphrase] = useState('')

  const [key, setKey] = useState('')

  const onCancel = async () => {
    await replyFailure(props.id, {
      unexpectedError: {
        detail: 'Rejected by user',
      },
    })
    props.onCancel?.()
  }

  const onOk = async () => {
    if (username === '') {
      Toast.error({
        content: 'Username must not be empty',
        stack: true,
      })
      return
    }
    let value: SSHAuthetication | null = null
    if (authenticateType === 'password') {
      value = {
        password: {
          password,
          username,
        },
      }
    } else if (authenticateType === 'publicKey') {
      value = {
        publicKey: {
          file: key,
          username,
          passphrase: passphrase === '' ? null : passphrase,
        },
      }
    } else if (authenticateType === 'keyboardInteractive') {
      value = {
        keyboardInteractive: {
          username,
        },
      }
    }
    if (value === null) {
      return
    }
    await replySuccess(props.id, value)
    props.onOk?.()
  }

  return (
    <Dialog
      onOk={onOk}
      onCancel={onCancel}
      title="Authenticate"
      afterClose={props.afterClose}
      visible={props.visible}
    >
      <div className={cx(flex_1, flex, flex_col)}>
        <div>
          <RadioGroup
            value={authenticateType}
            onChange={(e) => setAuthenticateType(e.target.value)}
            type="button"
          >
            {props.password && <Radio value={'password'}>Password</Radio>}
            {props.publicKey && <Radio value={'publicKey'}>Key</Radio>}
            {props.keyboardInteractive && (
              <Radio value={'keyboardInteractive'}>Keyboard Interactive</Radio>
            )}
          </RadioGroup>
        </div>
        <div className={cx(flex_1, flex, flex_col, authenticateType !== 'password' && hidden)}>
          <DialogFormItem title="Username:">
            <PureInput value={username} onChange={setUsername}></PureInput>
          </DialogFormItem>
          <DialogFormItem title="Password:">
            <PureInput mode="password" value={password} onChange={setPassword}></PureInput>
          </DialogFormItem>
        </div>
        <div className={cx(flex_1, flex, flex_col, authenticateType !== 'publicKey' && hidden)}>
          <DialogFormItem title="Username:">
            <PureInput value={username} onChange={setUsername}></PureInput>
          </DialogFormItem>
          <DialogFormItem title="Key:">
            <PathInput onSelected={setKey} value={key} onChange={setKey}></PathInput>
          </DialogFormItem>
          <DialogFormItem title="Passphrase:">
            <PureInput mode="password" value={passphrase} onChange={setPassphrase}></PureInput>
          </DialogFormItem>
        </div>
        <div
          className={cx(
            flex_1,
            flex,
            flex_col,
            authenticateType !== 'keyboardInteractive' && hidden,
          )}
        >
          <DialogFormItem title="Username:">
            <PureInput value={username} onChange={setUsername}></PureInput>
          </DialogFormItem>
        </div>
      </div>
    </Dialog>
  )
}
