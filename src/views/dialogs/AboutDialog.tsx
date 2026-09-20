import { AboutPanel } from '@/components/AboutPanel'
import { useCurrentModal } from '@/lib/multi-modal'

import { Dialog } from './Dialog'

export function NiceAboutDialog() {
  const modal = useCurrentModal()

  return (
    <AboutDialog
      visible={modal.visible}
      afterClose={modal.remove}
      onOk={() => {
        modal.resolve()
        modal.hide()
      }}
    />
  )
}

interface AboutDialogProps {
  visible: boolean
  onOk: () => void
  afterClose?: () => void
}

export function AboutDialog(props: AboutDialogProps) {
  return (
    <Dialog
      size="medium"
      visible={props.visible}
      onOk={props.onOk}
      onCancel={props.onOk}
      afterClose={props.afterClose}
      footerButtons={(ok) => ok}
      closable={false}
    >
      <AboutPanel />
    </Dialog>
  )
}
