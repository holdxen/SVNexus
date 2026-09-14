import { Button } from '@douyinfe/semi-ui'
import { cx } from '@linaria/core'
import { RefObject, useEffect, useImperativeHandle } from 'react'
import { RotatingLines } from 'react-loader-spinner'

import { useCurrentModal } from '@/lib/multi-modal'
import { flex, flex_1, flex_col, gap_y_2, items_center } from '@/styles/Classes'

import { Dialog } from './Dialog'

export interface LoadingDialogRef {
  close: () => void
}

export function NiceLoadingDialog(args: {
  cancelable: boolean
  onCancel?: () => Promise<boolean>
  onLoad?: () => void
  ref?: RefObject<LoadingDialogRef | null>
}) {
  const modal = useCurrentModal()

  let { ref, ...props } = args

  useImperativeHandle(ref, () => ({
    close: () => {
      modal.resolve()
      modal.hide()
    },
  }))

  return (
    <LoadingDialog
      onLoad={props.onLoad}
      onClose={() => {
        modal.resolve()
        modal.hide()
      }}
      cancelable={props.cancelable}
      onCancel={props.onCancel}
      afterClose={modal.remove}
      visible={modal.visible}
    ></LoadingDialog>
  )
}

export interface LoadingDialogProps {
  visible: boolean
  cancelable: boolean
  onCancel?: () => Promise<boolean>
  afterClose?: () => void
  onClose?: () => void
  onLoad?: () => void
}

export default function LoadingDialog(props: LoadingDialogProps) {
  const onCancel = async () => {
    if (props.onCancel) {
      const result = await props.onCancel()
      if (!result) {
        return
      }
    }
    props.onClose?.()
  }
  useEffect(() => {
    props.onLoad?.()
  }, [])
  return (
    <Dialog
      size={'auto'}
      closeOnEsc={false}
      onOk={onCancel}
      afterClose={props.afterClose}
      footer={<></>}
      header={<div></div>}
      visible={props.visible}
    >
      <div className={cx(flex_1, flex, flex_col, items_center, gap_y_2)}>
        <RotatingLines
          visible={true}
          height="200"
          width="200"
          color="grey"
          strokeWidth="5"
          animationDuration="0.75"
          ariaLabel="rotating-lines-loading"
          wrapperStyle={{}}
          wrapperClass=""
        />
        {props.cancelable && <Button onClick={onCancel}>Cancel</Button>}
      </div>
    </Dialog>
  )
}
