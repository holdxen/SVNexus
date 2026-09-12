import { Checkbox, Toast } from '@douyinfe/semi-ui'
import { Typography } from '@douyinfe/semi-ui'
import { cx } from '@linaria/core'
import { useState } from 'react'

import { Authentication } from '@/bindings/Authentication'
import { SubversionEvent } from '@/bindings/SubversionEvent'
import PureInput from '@/components/PureInput'
import { replySuccess } from '@/context/Functions'
import { flex, flex_col, gap_y_1, hidden } from '@/styles/Classes'

import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

const { Text } = Typography

export const title = 'Authenticate'

type Authenticate = Extract<SubversionEvent, { authenticate: any }>['authenticate']

export interface AuthenticateDialogProps extends Authenticate {
  onClose?: () => void
  afterClose?: () => void
}

export default function AuthenticateDialog(props: AuthenticateDialogProps) {
  const [username, setUsername] = useState(props.username)
  const [password, setPassword] = useState('')
  const [save, setSave] = useState(false)
  const [visible, setVisible] = useState(true)

  // useImperativeHandle(props.ref, () => {
  //   return {
  //     show() {
  //       setVisible(true)
  //     },
  //   }
  // })

  const close = () => {
    setVisible(false)
    props.onClose?.()
  }

  const onOk = async () => {
    if (username === '') {
      Toast.error({
        content: 'Username must not be empty',
        stack: true,
      })
      return
    }
    // let msg: ReplyMessage = {
    //   success: {
    //     username,
    //     password,
    //     save,
    //   } as Authentication,
    // }
    const value: Authentication = {
      username,
      password,
      save,
    }
    await replySuccess(props.id, value)
    close()
  }
  const onCancel = async () => {
    // let msg: ReplyMessage = {
    //   success: null,
    // }
    await replySuccess(props.id, null)
    close()
  }

  return (
    <Dialog afterClose={props.afterClose} visible={visible} onOk={onOk} onCancel={onCancel}>
      <div className={cx(flex, flex_col, gap_y_1)}>
        <DialogFormItem title={'Realm:'}>
          <Text>{props.realm}</Text>
        </DialogFormItem>
        <DialogFormItem title={'Username:'}>
          <PureInput value={username} onChange={setUsername}></PureInput>
        </DialogFormItem>
        <DialogFormItem className={cx(!props.needPassword && hidden)} title={'Password:'}>
          <PureInput mode="password" value={password} onChange={setPassword}></PureInput>
        </DialogFormItem>
        <Checkbox
          className={cx(!props.maySave && hidden)}
          checked={save}
          onChange={(e) => setSave(e.target.checked ?? false)}
        >
          Save
        </Checkbox>
      </div>
    </Dialog>
  )
}
