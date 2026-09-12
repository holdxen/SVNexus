import { Descriptions } from '@douyinfe/semi-ui'
import { cx } from '@linaria/core'
import { useState } from 'react'

import { replySuccess } from '@/context/Functions'
import { SubversionEventMap } from '@/context/Subversion'
import { flex, flex_col } from '@/styles/Classes'

import { Dialog } from './Dialog'

type MaySavePasswordAsPlainTextProps = SubversionEventMap['savePasswordAsPlainText'] & {
  onClose?: () => void
  afterClose?: () => void
}

export default function MaySavePasswordAsPlainText(props: MaySavePasswordAsPlainTextProps) {
  const [visible, setVisible] = useState(true)

  const descriptions = [{ key: 'Realm', value: props.realm }]

  const close = () => {
    setVisible(false)
    props.onClose?.()
  }

  const onOk = async () => {
    // const msg: ReplyMessage = {
    //   success: true,
    // }
    await replySuccess(props.id, true)
    close()
  }
  const onCancel = async () => {
    // const msg: ReplyMessage = {
    //   success: false,
    // }
    await replySuccess(props.id, false)
    close()
  }

  return (
    <Dialog afterClose={props.afterClose} visible={visible} onOk={onOk} onCancel={onCancel}>
      <div className={cx(flex, flex_col)}>
        <Descriptions data={descriptions}></Descriptions>
      </div>
    </Dialog>
  )
}
