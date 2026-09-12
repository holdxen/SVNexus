import {
  IconAlertCircle,
  IconAlertTriangle,
  IconClose,
  IconHelpCircle,
  IconInfoCircle,
  IconTickCircle,
} from '@douyinfe/semi-icons'
import { Typography } from '@douyinfe/semi-ui'
import { Button } from '@douyinfe/semi-ui'
import { ButtonProps } from '@douyinfe/semi-ui/lib/es/button'
import { type Type } from '@douyinfe/semi-ui/lib/es/button'
import { css, cx } from '@linaria/core'
import { RefObject, useContext, useEffect, useRef, useState } from 'react'
import React from 'react'
import { Modal } from 'react-responsive-modal'

import {
  break_all,
  break_word,
  cursor_grabbing,
  flex,
  flex_1,
  flex_col,
  gap_x_1,
  gap_y_2,
  items_center,
  justify_center,
  min_h_0,
  min_w_0,
} from '@/styles/Classes'

import { ModalDialogContext } from '../DialogContext'

interface ModalFooterProps {
  okText?: string
  okType?: Type
  cancelText?: string
  confirmLoading?: boolean
  confirmDisable?: boolean
  cancelLoading?: boolean
  cancelDisable?: boolean
  hasCancel?: boolean
  footerFill?: boolean
  onCancel?: (e: React.MouseEvent) => void
  onOk?: (e: React.MouseEvent) => void
  cancelButtonProps?: Record<string, any>
  okButtonProps?: Record<string, any>
  locale?: {
    cancel?: string
    confirm?: string
  }
  footerMargin?: string
  footerButtons?: (ok: React.ReactNode, cancel?: React.ReactNode) => React.ReactNode
}

const ModalFooter: React.FC<ModalFooterProps> = ({
  okText,
  okType = 'primary',
  cancelText,
  confirmLoading = false,
  confirmDisable = false,
  cancelLoading = false,
  cancelDisable = false,
  hasCancel = true,
  footerFill = false,
  onCancel,
  onOk,
  cancelButtonProps,
  okButtonProps,
  locale = { cancel: '取消', confirm: '确定' },
  footerMargin,
  footerButtons,
}) => {
  const margin = footerMargin ?? '5px 0px'
  const getCancelButton = () => {
    if (!hasCancel) {
      return null
    }
    return (
      <Button
        aria-label="cancel"
        onClick={onCancel}
        loading={cancelLoading}
        type="tertiary"
        block={footerFill}
        {...cancelButtonProps}
        style={{
          ...(footerFill ? { marginLeft: 'unset' } : {}),
          ...cancelButtonProps?.style,
        }}
        x-semi-children-alias="cancelText"
        disabled={cancelDisable}
      >
        {cancelText || locale.cancel}
      </Button>
    )
  }

  if (footerButtons) {
    const ok = (
      <Button
        aria-label="confirm"
        type={okType}
        theme="solid"
        block={footerFill}
        loading={confirmLoading}
        onClick={onOk}
        {...okButtonProps}
        x-semi-children-alias="okText"
      >
        {okText || locale.confirm}
      </Button>
    )
    const cancel = getCancelButton()
    return (
      <div className={cx(flex, gap_x_1)} style={{ margin: margin }}>
        <div className={cx(flex_1)}></div>
        {footerButtons(ok, cancel)}
      </div>
    )
  } else {
    return (
      <div className={cx(flex, gap_x_1)} style={{ margin: margin }}>
        <div className={cx(flex_1)}></div>
        {getCancelButton()}
        <Button
          aria-label="confirm"
          type={okType}
          theme="solid"
          block={footerFill}
          loading={confirmLoading}
          onClick={onOk}
          {...okButtonProps}
          x-semi-children-alias="okText"
          disabled={confirmDisable}
        >
          {okText || locale.confirm}
        </Button>
      </div>
    )
  }
}

export interface DialogProps {
  children?: React.ReactNode
  visible: boolean
  onOk?: () => void | Promise<any>
  onCancel?: () => void | Promise<any>
  className?: string
  title?: string
  size?: 'small' | 'medium' | 'large' | 'auto' | number
  closable?: boolean
  maskClosable?: boolean
  footer?: React.ReactNode
  okButtonProps?: ButtonProps
  cancelButtonProps?: ButtonProps
  closeOnEsc?: boolean
  afterClose?: () => void
  contentClassName?: string
  fullScreen?: boolean
  footerMargin?: string
  footerButtons?: (ok: React.ReactNode, cancel?: React.ReactNode) => React.ReactNode
  icon?: React.ReactNode | 'info' | 'success' | 'warning' | 'error' | 'confirm'
  closeIcon?: React.ReactNode

  getPopupContainer?: () => HTMLElement

  closeIconDisabled?: boolean

  header?: React.ReactNode
  initialFocusRef?: React.RefObject<HTMLElement | null>
}

export interface DialogBase {
  show: () => void
}

const headerStyle = css`
  display: flex;
  align-items: flex-start;
  margin: 24px 0;
  padding: 0 0;
  font-size: 14px;
  font-weight: 600;
  background-color: transparent;
  color: var(--semi-color-text-0);
  border-bottom: 0 solid transparent;
`
const headerTitle = css`
  display: inline-flex;
  align-items: flex-start;
  justify-content: flex-start;
  width: 100%;
  margin: 0;
`

export function Dialog(props: DialogProps) {
  const modalDialogContext = useContext(ModalDialogContext)
  const [isConfirmLoading, setIsConfirmLoading] = useState(false)
  const [isCancelLoading, setIsCancelLoading] = useState(false)
  const [created, setCreated] = useState(props.visible)
  const [cancelDisable, setCancelDisable] = useState(false)

  useEffect(() => {
    setCreated(true)
  }, [props.visible])

  const onOk = () => {
    if (isConfirmLoading) {
      return
    }
    if (!props.onOk) {
      return
    }
    let promise = props.onOk()
    if (promise instanceof Promise) {
      setIsConfirmLoading(true)
      setCancelDisable(true)
      promise.finally(() => {
        setIsConfirmLoading(false)
        setCancelDisable(false)
      })
    }
  }

  const onCancel = () => {
    if (isCancelLoading) {
      return
    }
    if (!props.onCancel) {
      return
    }
    let promise = props.onCancel()
    if (promise instanceof Promise) {
      setIsCancelLoading(true)
      promise.finally(() => {
        setTimeout(() => {
          setIsCancelLoading(false)
        }, 2000)
      })
    }
  }

  const headerRef = useRef<HTMLDivElement>(null)
  const getPopupContainer = props.getPopupContainer
    ? props.getPopupContainer
    : modalDialogContext === null
      ? undefined
      : modalDialogContext.container
  const container = getPopupContainer?.() ?? document.body

  const modalRef = useRef<HTMLDivElement>(null)

  const { position, handleMouseDown } = useDraggable(container, modalRef)

  let icon: React.ReactNode | undefined
  if (typeof props.icon === 'string') {
    switch (props.icon) {
      case 'info':
        icon = (
          <IconInfoCircle
            size="extra-large"
            className="semi-modal-confirm-icon semi-modal-info-icon"
          />
        )
        break
      case 'success':
        icon = (
          <IconTickCircle
            size="extra-large"
            className="semi-modal-confirm-icon semi-modal-success-icon"
          />
        )
        break
      case 'warning':
        icon = (
          <IconAlertTriangle
            size="extra-large"
            className="semi-modal-confirm-icon semi-modal-warning-icon"
          />
        )
        break
      case 'error':
        icon = (
          <IconAlertCircle
            size="extra-large"
            className="semi-modal-confirm-icon semi-modal-error-icon"
          />
        )
        break
      case 'confirm':
        icon = (
          <IconHelpCircle
            size="extra-large"
            className="semi-modal-confirm-icon semi-modal-confirm-icon"
          />
        )
        break
    }
  } else {
    // 用户直接传入了 ReactNode，直接使用
    icon = props.icon
  }

  let closeIcon = props.closeIcon ? (
    props.closeIcon
  ) : (
    <Button
      aria-label="close"
      className="semi-modal-close"
      onClick={props.onCancel}
      type="tertiary"
      icon={<IconClose />}
      theme="borderless"
      size="small"
      disabled={props.closeIconDisabled || isConfirmLoading || isCancelLoading}
    />
  )

  if (!(props.closable ?? true)) {
    closeIcon = <></>
  }

  let header: React.ReactNode = (
    <div
      onMouseDown={handleMouseDown}
      ref={headerRef}
      className={cx(
        headerStyle,
        items_center,
        gap_x_1,
        !props.fullScreen && cursor_grabbing,
        css`
          margin: 10px 0px;
        `,
      )}
    >
      {icon}
      <Typography.Title heading={5} className={cx(headerTitle, break_all, break_word)}>
        {props.title}
      </Typography.Title>
      {closeIcon}
    </div>
  )
  if (props.header) {
    header = props.header
  }

  // useEffect(() => {
  //   if (modalRef.current === null) {
  //     return
  //   }
  //   console.log('modalRef', modalRef.current.clientTop, modalRef.current.clientLeft, position)
  //   console.log("modalref:", modalRef.current.getBoundingClientRect())
  //   console.log("container:", container.getBoundingClientRect())

  // }, [position])

  if (!created) {
    return <></>
  }

  let w = '448px'
  if (typeof props.size === 'string') {
    switch (props.size) {
      case 'small':
        w = '448px'
        break
      case 'medium':
        w = '684px'
        break
      case 'large':
        w = '920px'
        break
      case 'auto':
        w = 'auto'
        break
    }
  } else if (typeof props.size === 'number') {
    w = `${props.size}px`
  }
  if (props.fullScreen) {
    w = '100%'
  }

  return (
    <Modal
      initialFocusRef={props.initialFocusRef}
      onAnimationEnd={() => {
        if (!props.visible) {
          props.afterClose?.()
        }
      }}
      ref={modalRef}
      animationDuration={200}
      classNames={{
        modalContainer: cx(flex, justify_center, items_center),
      }}
      showCloseIcon={false}
      onClose={() => {
        props.onCancel?.()
      }}
      styles={{
        modal: {
          overflow: 'hidden',
          width: w,
          maxWidth: w,
          display: 'flex',
          height: props.fullScreen ? '100%' : undefined,
          maxHeight: props.fullScreen ? '100%' : '85%',
          margin: props.fullScreen ? 0 : undefined,
          borderRadius: props.fullScreen ? undefined : '10px',
          boxSizing: 'border-box',
          position: 'relative',
          top: `${position.y}px`,
          left: `${position.x}px`,
        },
        root: {
          position: 'absolute',
        },
        overlay: {
          position: 'absolute',
          background: 'var(--semi-color-overlay-bg)',
        },
        modalContainer: {
          position: 'relative',
          overflow: 'hidden',
        },
      }}
      container={container}
      open={props.visible}
    >
      <div className={cx(flex_1, flex, flex_col, min_h_0, min_w_0, gap_y_2)}>
        {header}
        {props.children}
        {props.footer ? (
          props.footer
        ) : (
          <ModalFooter
            footerButtons={props.footerButtons}
            confirmLoading={isConfirmLoading}
            cancelLoading={isCancelLoading}
            cancelDisable={cancelDisable}
            onCancel={onCancel}
            onOk={onOk}
            okButtonProps={props.okButtonProps}
            cancelButtonProps={props.cancelButtonProps}
            footerMargin={props.footerMargin}
          ></ModalFooter>
        )}
      </div>
    </Modal>
  )

  // return (
  //   <Modal
  //     modalContentClass={cx(!props.fullScreen && maxHeightOfParent)}
  //     header={header}
  //     fullScreen={props.fullScreen}
  //     bodyStyle={{
  //       display: 'flex',
  //       flexDirection: 'column',
  //       minHeight: '0px',
  //       // maxHeight: props.fullScreen ? 'auto' : '70vh',
  //     }}
  //     footer={
  //       props.footer ? (
  //         props.footer
  //       ) : (
  //         <ModalFooter
  //           footerButtons={props.footerButtons}
  //           confirmLoading={isConfirmLoading}
  //           cancelLoading={isCancelLoading}
  //           cancelDisable={cancelDisable}
  //           onCancel={onCancel}
  //           onOk={onOk}
  //           okButtonProps={props.okButtonProps}
  //           cancelButtonProps={props.cancelButtonProps}
  //           footerMargin={props.footerMargin}
  //         ></ModalFooter>
  //       )
  //     }
  //     afterClose={props.afterClose}
  //     closeOnEsc={props.closeOnEsc && !isConfirmLoading && !isCancelLoading}
  //     okButtonProps={props.okButtonProps}
  //     cancelButtonProps={props.cancelButtonProps}
  //     maskClosable={props.maskClosable && !isConfirmLoading && !isCancelLoading}
  //     closable={props.closable}
  //     size={props.size}
  //     className={cx(!props.fullScreen && containerSize, props.className)}
  //     visible={props.visible}
  //     modalRender={(modal) =>
  //       (props.fullScreen || props.header) ? (
  //         modal
  //       ) : (
  //         <DragMove
  //           handler={() => {
  //             if (headerRef.current === null) {
  //               throw new Error('Should not be null')
  //             }
  //             return headerRef.current
  //           }}
  //           constrainer={getPopupContainer}
  //           positionStrategy={'relative'}
  //         >
  //           {modal}
  //         </DragMove>
  //       )
  //     }
  //     title={props.title}
  //     onOk={props.onOk}
  //     onCancel={props.onCancel}
  //     getPopupContainer={getPopupContainer}
  //     centered={true}
  //   >
  //     {props.children}
  //   </Modal>
  // )
}

interface Position {
  x: number
  y: number
}

function useDraggable(
  container: HTMLElement,
  modal: RefObject<HTMLElement | null>,
  initialPosition: Position = { x: 0, y: 0 },
) {
  const [position, setPosition] = useState<Position>(initialPosition)
  const isDragging = useRef(false)
  const startPos = useRef<Position>({ x: 0, y: 0 })
  const startMouse = useRef<Position>({ x: 0, y: 0 })

  const handleMouseDown = (e: React.MouseEvent) => {
    isDragging.current = true
    startMouse.current = { x: e.clientX, y: e.clientY }
    startPos.current = { ...position }
    e.preventDefault()

    const handleMouseMove = (e: MouseEvent) => {
      if (!isDragging.current) return
      if (modal.current === null) return
      const target = modal.current
      const targetReact = target.getBoundingClientRect()
      const containerRect = container.getBoundingClientRect()

      const minX = -(containerRect.width - targetReact.width) / 2

      const minY = -(containerRect.height - targetReact.height) / 2

      let x = Math.max(minX, startPos.current.x + (e.clientX - startMouse.current.x))
      x = Math.min(-minX, x)

      let y = Math.max(minY, startPos.current.y + (e.clientY - startMouse.current.y))
      y = Math.min(-minY, y)

      setPosition({
        x,
        y,
      })
    }

    const handleMouseUp = () => {
      isDragging.current = false
      document.removeEventListener('mousemove', handleMouseMove)
      document.removeEventListener('mouseup', handleMouseUp)
    }

    document.addEventListener('mousemove', handleMouseMove)
    document.addEventListener('mouseup', handleMouseUp)
  }

  const resetPosition = () => setPosition(initialPosition)

  return { position, setPosition, handleMouseDown, resetPosition }
}
