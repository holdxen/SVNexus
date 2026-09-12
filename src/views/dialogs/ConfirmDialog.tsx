import { useCurrentModal } from '@/lib/multi-modal'

import { Dialog } from './Dialog'

interface ConfirmDialogProps {
  icon?: string
  children: React.ReactNode
  title?: string
}

const ConfirmDialog = (props: ConfirmDialogProps) => {
  const modal = useCurrentModal()

  return (
    <Dialog
      icon={props.icon}
      visible={modal.visible}
      title={props.title}
      onOk={() => {
        modal.resolve(true)
        modal.hide()
      }}
      onCancel={() => {
        modal.resolve(false)
        modal.hide()
      }}
      afterClose={() => modal.remove()}
    >
      {props.children}
    </Dialog>
  )
}

export default ConfirmDialog
