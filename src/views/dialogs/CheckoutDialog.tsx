import { IconFolderStroked } from '@douyinfe/semi-icons'
import { Button, Card, Checkbox, Descriptions, Toast } from '@douyinfe/semi-ui'
import { Progress } from '@douyinfe/semi-ui'
import { css, cx } from '@linaria/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useMemoizedFn } from 'ahooks'
import React, { useCallback, useEffect, useRef, useState } from 'react'

import { CheckoutOptions } from '@/bindings/CheckoutOptions'
import { Depth } from '@/bindings/Depth'
import { Revision } from '@/bindings/Revision'
import PureInput from '@/components/PureInput'
import { ScrollArea } from '@/components/ScrollArea'
import RevisionSelect, { RevisionKind } from '@/components/subversion/RevisionSelect'
import { Subversion, SubversionEventMap, useSubversion } from '@/context/Subversion'
import { useTabContent } from '@/context/TabContent'
import { useTabManager } from '@/context/TabManager'
import { useCurrentModal, useModal } from '@/lib/multi-modal'
import { disable_move } from '@/styles/Components'
import { cursor_pointer, flex, flex_1, flex_col, gap_y_3, min_h_0 } from '@/styles/Classes'
import errorHumanString from '@/utils/Error'
import { localPath } from '@/utils/Path'

import DepthSelect from '../../components/subversion/DepthSelect'
import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

// export interface CheckoutDialogProps {
//   ref?: Ref<DialogBase>
// }

const button = css`
  color: var(--semi-color-primary);
  &:hover {
    color: var(--semi-color-primary-hover);
  }
  &:active {
    color: var(--semi-color-primary-active);
  }
`

// const indeterminate = css`
//   .semi-progress-track-inner {
//     width: 30% !important;
//     animation: indeterminate 1.5s infinite ease-in-out;
//   }
//   @keyframes indeterminate {
//     0% {
//       transform: translateX(-100%);
//     }
//     100% {
//       transform: translateX(400%);
//     }
//   }
//   .semi-progress-track {
//     overflow: hidden;
//   }
// `
// const descriptionsItem = css`
//   .semi-descriptions-item {
//     padding-bottom: 0px;
//   }
// `

function CheckoutingDialog({ options }: { options: CheckoutOptions }) {
  const subversion = useSubversion()
  const modal = useCurrentModal()

  const [revision, setRevsion] = useState<number | null>(null)
  const [current, setCurrent] = useState('')
  const [pos, setPos] = useState<number>(-1)
  const [total, setTotal] = useState<number>(-1)
  // const [visible, setVisible] = useState(true)
  const [finished, setFinished] = useState(false)
  // const [promise, setPromise] = useState<Promise<number> | null>(null)
  const [cancel, setCancel] = useState<((msg: string) => Promise<any>) | null>(null)
  const [hasError, setHasError] = useState(false)
  const tabManager = useTabManager()

  const onProgressNotify = useCallback((data: SubversionEventMap['progressNotify']) => {
    setPos(data.pos)
    setTotal(data.total)
  }, [])
  const onWorkingNotify = useCallback((data: SubversionEventMap['workingCopyNotify']) => {
    setCurrent(localPath.getFileName(data.notify.path) ?? '')
  }, [])

  const cleanup = useMemoizedFn(() => {
    if (cancel !== null) {
      console.log('reload to cancel')
      cancel('Destroyed').catch((error) => {
        console.log('Failed to destroy task: ', error)
      })
    }
  })

  useEffect(() => {
    if (finished) {
      console.log('Task has finished, you should create another dialog to exec this task')
      return
    }
    async function call() {
      await Subversion.callOnce({
        factory: subversion,
        call: async (context) => {
          context.on('progressNotify', onProgressNotify)
          context.on('workingCopyNotify', onWorkingNotify)
          const promise = context.checkout(options)
          console.log('cancel')
          setCancel(() => async (msg: string) => {
            console.log('cancel now on set', msg)
            await context?.cancel(msg)
            await promise
          })
          let number = await promise
          setRevsion(number)
        },
        onError: (error) => {
          Toast.error({
            content: `Failed to checkout: ${errorHumanString(error)}`,
            stack: true,
          })
          setHasError(true)
        },
        onFinally: (context) => {
          context?.off('progressNotify', onProgressNotify)
          context?.off('workingCopyNotify', onWorkingNotify)
          setFinished(true)
          setCancel(null)
        },
      })

      // console.log('checkout now', options)
      // let context: Subversion | null = null
      // try {
      //   context = await subversion.context()

      //   context.on('progressNotify', onProgressNotify)
      //   context.on('workingCopyNotify', onWorkingNotify)
      //   const promise = context.checkout(options)
      //   console.log('cancel')
      //   setCancel(() => async (msg: string) => {
      //     console.log('cancel now on set', msg)
      //     await context?.cancel(msg)
      //     await promise
      //   })
      //   let number = await promise
      //   setRevsion(number)
      // } catch (error) {
      //   console.log('Failed to checkout:', error)
      //   Toast.error({
      //     content: `Failed to checkout: ${errorHumanString(error)}`,
      //     stack: true,
      //   })
      //   setHasError(true)
      // } finally {
      //   context?.off('progressNotify', onProgressNotify)
      //   context?.off('workingCopyNotify', onWorkingNotify)
      //   setFinished(true)
      //   setCancel(null)
      //   if (context) {
      //     console.log('release subversion context:', context.id)
      //     subversion.release(context)
      //   }
      // }
    }

    call()
    return cleanup
    // return () => {
    //   if (cancel !== null) {
    //     console.log('reload to cancel')
    //     cancel('Destroyed').catch((error) => {
    //       console.log('Failed to destroy task: ', error)
    //     })
    //   }
    // }
  }, [])

  // useImperativeHandle(ref, () => ({
  //   show() {
  //     setVisible(true)
  //   },
  // }))

  const descriptions = [
    { key: 'Url', value: options.url },
    { key: 'Path', value: options.path },
    { key: 'Revision', value: revision?.toString() ?? null },
  ]

  // const close = () => {
  //   modal.resolve(true)
  //   modal.hide()
  //   // setVisible(false)
  //   // onClose()
  // }

  const onCancel = async () => {
    console.log('click to cancel')
    if (cancel !== null) {
      try {
        await cancel('Cancelled by user')
      } catch (error) {
        console.log('Cancel error: ', error)
      }
    }
    setCancel(null)
  }

  const onOk = () => {
    console.log('hasError:', hasError)
    modal.resolve(!hasError)
    modal.hide()
  }

  const percent = finished ? 100 : Math.max(Number((pos / total) * 100), 0)

  const isIndeterminate = finished ? false : pos < 0 || total < 0

  const tabContent = useTabContent()

  const openButton =
    finished && !hasError ? (
      <Button
        type="primary"
        theme="solid"
        onClick={() => {
          console.log('open working copy:', options)
          tabManager.openWorkingCopy(tabContent.identity, options.path)
          modal.resolve(true)
          modal.hide()
        }}
      >
        打开
      </Button>
    ) : (
      <></>
    )

  const footerButtons = (ok: React.ReactNode, cancel?: React.ReactNode) => {
    return (
      <>
        {cancel}
        {ok}
        {openButton}
      </>
    )
  }

  return (
    <Dialog
      afterClose={modal.remove}
      footerButtons={footerButtons}
      closeOnEsc={false}
      okButtonProps={finished ? undefined : { style: { display: 'none' } }}
      cancelButtonProps={finished ? { style: { display: 'none' } } : undefined}
      maskClosable={false}
      closable={false}
      size="medium"
      title={'Checkout...'}
      visible={modal.visible}
      onCancel={onCancel}
      onOk={onOk}
    >
      <div className={cx(flex, flex_col, gap_y_3, disable_move)}>
        <Card>
          <Descriptions size="medium" data={descriptions}></Descriptions>
        </Card>
        <DialogFormItem title={'Progress:'}>
          <div className={cx(flex_1, flex_col)}>
            <div>{`Current: ${current}`}</div>
            <div className={cx(flex)}>
              <div>{`Percentage:${percent}%`}</div>
              <div className={cx(flex_1)}></div>
              <div>{`${pos < 0 ? 'unknown' : String(pos)}/${total < 0 ? 'unknown' : String(total)}`}</div>
            </div>
            <Progress
              indeterminate={isIndeterminate}
              percent={percent}
              size="large"
            />
          </div>
        </DialogFormItem>
      </div>
    </Dialog>
  )
}

const inputIcon = css`
  .semi-input-append {
    cursor: default;
  }
`

export function NiceCheckoutDialog() {
  const modal = useCurrentModal()
  return (
    <CheckoutDialog
      visible={modal.visible}
      onOk={() => {
        modal.resolve(true)
        modal.hide()
      }}
      afterClose={modal.remove}
      onCancel={() => {
        modal.resolve(false)
        modal.hide()
      }}
    ></CheckoutDialog>
  )
}

export interface CheckoutDialogProps {
  visible: boolean
  onOk?: () => void
  onCancel?: () => void
  afterClose?: () => void
}

export default function CheckoutDialog(props: CheckoutDialogProps) {
  const revisionKinds: RevisionKind[] = ['head', 'number', 'date']

  const [pegRevision, setPegRevision] = useState<Revision>('unspecified')
  const [revision, setRevision] = useState<Revision>('head')

  const [ignoreExternals, setIgnoreExternals] = useState(false)
  const [depth, setDepth] = useState<Depth>('infinity')
  const [url, setUrl] = useState('')
  const [path, setPath] = useState('')
  const focusElement = useRef(null)
  // const [checkoutOptions, setCheckoutOptions] = useState<CheckoutOptions | null>(null)
  // const [checkoutingDialogVersion, setCheckoutingDialogVersion] = useState(0)

  const selectPath = async () => {
    const selected = await open({
      title: 'Select folder',
      multiple: false,
      directory: true,
    })
    console.log(selected)
    if (selected === null) {
      return
    }
    setPath(selected)
  }

  const onCancel = () => {
    props.onCancel?.()
    // setVisible(false)
  }

  const modal = useModal()

  const onOk = async () => {
    if (url === '') {
      Toast.error({
        content: 'Url must not be empty',
        stack: true,
      })
      return
    }
    if (path === '') {
      Toast.error({
        content: 'Path must not be empty',
        stack: true,
      })
      return
    }

    const options: CheckoutOptions = {
      url,
      path,
      pegRevision: revision,
      revision: revision,
      depth: depth,
      ignoreExternals,
      allowUnversionedObstructions: false,
      storePristine: null,
    }

    console.log('Start checkout: ', options)

    const result = await modal.show(CheckoutingDialog, { options }).as<boolean>()

    if (result) {
      props.onOk?.()
    }

    // setCheckoutOptions(options)
    // setCheckoutingDialogVisible(true)
    // setCheckoutingDialogVersion((v) => v + 1)
    // setTimeout(() => {
    //   checkoutingDialog.current?.show()
    // }, 0)
  }

  // const checkoutingDialog =
  //   checkoutOptions === null ? (
  //     <></>
  //   ) : (
  //     <CheckoutingDialog
  //       key={checkoutingDialogVersion}
  //       options={checkoutOptions}
  //     ></CheckoutingDialog>
  //   )

  return (
    <Dialog
      afterClose={props.afterClose}
      size="medium"
      title={'Checkout'}
      visible={props.visible}
      onCancel={onCancel}
      onOk={onOk}
      initialFocusRef={focusElement}
    >
      <ScrollArea className={cx(flex_1, min_h_0)} contentClassName={cx(flex)}>
        <div className={cx(flex_1, flex, gap_y_3, flex_col)}>
          <DialogFormItem title={'Url:'}>
            <PureInput
              ref={focusElement}
              autoFocus
              value={url}
              onChange={(value) => setUrl(value)}
            ></PureInput>
          </DialogFormItem>
          <DialogFormItem title={'Path:'}>
            <PureInput
              className={inputIcon}
              value={path}
              onChange={(value) => setPath(value)}
              addonAfter={
                <IconFolderStroked onClick={selectPath} className={cx(cursor_pointer, button)} />
              }
            ></PureInput>
          </DialogFormItem>
          <DialogFormItem title={'Peg revision:'}>
            <RevisionSelect
              disableLayout={true}
              className={cx(flex_1)}
              kinds={['unspecified', 'head', 'number', 'date']}
              value={pegRevision}
              onChange={setPegRevision}
            ></RevisionSelect>
          </DialogFormItem>
          <DialogFormItem title={'Revision:'}>
            <RevisionSelect
              disableLayout={true}
              className={cx(flex_1)}
              kinds={revisionKinds}
              value={revision}
              onChange={setRevision}
            ></RevisionSelect>
          </DialogFormItem>
          <DialogFormItem title={'Depth:'}>
            <DepthSelect
              value={depth}
              onChange={(value) => setDepth(value)}
              className={flex_1}
            ></DepthSelect>
          </DialogFormItem>
          <DialogFormItem title={'Options:'}>
            <Checkbox
              checked={ignoreExternals}
              onChange={(e) => setIgnoreExternals(e.target.checked ?? false)}
            >
              Ignore externals
            </Checkbox>
          </DialogFormItem>
        </div>
      </ScrollArea>
    </Dialog>
  )
}
