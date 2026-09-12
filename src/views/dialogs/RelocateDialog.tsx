import { Checkbox } from '@douyinfe/semi-ui'
import { useRef, useState } from 'react'

import { RelocateOptions } from '@/bindings/RelocateOptions'
import PureInput from '@/components/PureInput'
import { Subversion, useSubversion } from '@/context/Subversion'
import { useCurrentModal } from '@/lib/multi-modal'

import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

export function NiceRelocateDialog(props: { workingCopy: string; from: string }) {
  const modal = useCurrentModal()
  return (
    <RelocateDialog
      onCancel={() => {
        modal.resolve(false)
        modal.hide()
      }}
      onOk={() => {
        modal.resolve(true)
        modal.hide()
      }}
      afterClose={modal.remove}
      visible={modal.visible}
      workingCopy={props.workingCopy}
      from={props.from}
    ></RelocateDialog>
  )
}

export interface RelocateDialogProps {
  visible: boolean
  onOk?: () => void
  onCancel?: () => void
  afterClose?: () => void
  workingCopy: string
  from: string
}

export default function RelocateDialog(props: RelocateDialogProps) {
  const [from, setFrom] = useState(props.from)
  const [to, setTo] = useState('')
  const [ignoreExternals, setIgnoreExternals] = useState(false)

  const subversion = useSubversion()
  const focusRef = useRef<HTMLInputElement>(null)

  const onOk = async () => {
    await Subversion.callOnce({
      factory: subversion,
      async call(context) {
        const options: RelocateOptions = {
          wcRootDir: props.workingCopy,
          fromPrefix: from,
          toPrefix: to,
          ignoreExternals: ignoreExternals,
        }
        await context.relocate(options)
        props.onOk?.()
      },
    })
  }

  return (
    <Dialog
      initialFocusRef={focusRef}
      title="Relocate"
      onOk={onOk}
      onCancel={props.onCancel}
      visible={props.visible}
      afterClose={props.afterClose}
    >
      <DialogFormItem title="Working copy:">
        <PureInput disabled value={props.workingCopy} />
      </DialogFormItem>
      <DialogFormItem title="From(prefix):">
        <PureInput value={from} onChange={(e) => setFrom(e)} />
      </DialogFormItem>
      <DialogFormItem title="To(prefix):">
        <PureInput ref={focusRef} value={to} onChange={(e) => setTo(e)} />
      </DialogFormItem>
      <DialogFormItem title="Options:">
        <Checkbox
          checked={ignoreExternals}
          onChange={(e) => setIgnoreExternals(e.target.checked ?? false)}
        >
          Ignore externals
        </Checkbox>
      </DialogFormItem>
    </Dialog>
  )
}
