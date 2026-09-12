import { Descriptions } from '@douyinfe/semi-ui'
import { Data } from '@douyinfe/semi-ui/lib/es/descriptions'
import { css, cx } from '@linaria/core'

import { useCurrentModal } from '@/lib/multi-modal'
import { flex, flex_1 } from '@/styles/Classes'

import { Dialog } from './Dialog'

export function NiceInfoDialog(props: { data: Data[] }) {
  const modal = useCurrentModal()

  return (
    <InfoDialog
      afterClose={modal.remove}
      onOk={() => {
        modal.resolve()
        modal.hide()
      }}
      visible={modal.visible}
      data={props.data}
    ></InfoDialog>
  )
}

export interface InfoDialogProps {
  data?: Data[]
  visible: boolean
  onOk: () => void
  afterClose?: () => void
}

const wrap = css`
  .semi-descriptions-value {
    word-break: break-all;
    word-wrap: break-word;
  }
`

export default function InfoDialog(props: InfoDialogProps) {
  return (
    <Dialog
      size="medium"
      afterClose={props.afterClose}
      footerButtons={(ok) => ok}
      visible={props.visible}
      onOk={props.onOk}
      onCancel={props.onOk}
    >
      <Descriptions className={cx(wrap, flex, flex_1)} data={props.data}></Descriptions>
    </Dialog>
  )
}
