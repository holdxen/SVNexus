import { IconFolderStroked } from '@douyinfe/semi-icons'
import { Card, Checkbox, Descriptions, Select, Toast } from '@douyinfe/semi-ui'
import { Progress } from '@douyinfe/semi-ui'
import { css, cx } from '@linaria/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useMemoizedFn } from 'ahooks'
import { useCallback, useEffect, useRef, useState } from 'react'

import { Depth } from '@/bindings/Depth'
import { ExportOptions } from '@/bindings/ExportOptions'
import { NativeEOL } from '@/bindings/NativeEOL'
import { Revision } from '@/bindings/Revision'
import PureInput from '@/components/PureInput'
import { ScrollArea } from '@/components/ScrollArea'
import RevisionSelect, { RevisionKind } from '@/components/subversion/RevisionSelect'
import { Subversion, SubversionEventMap, useSubversion } from '@/context/Subversion'
import { useT } from '@/i18n'
import { useCurrentModal, useModal } from '@/lib/multi-modal'
import { cursor_pointer, flex, flex_1, flex_col, gap_y_2, gap_y_3, min_h_0 } from '@/styles/Classes'
import { disable_move } from '@/styles/Components'
import errorHumanString from '@/utils/Error'
import { localPath } from '@/utils/Path'

import DepthSelect from '../../components/subversion/DepthSelect'
import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

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

// Export 进度对话框：执行期间显示进度，支持取消
function ExportingDialog({ options }: { options: ExportOptions }) {
  const t = useT()
  const subversion = useSubversion()
  const modal = useCurrentModal()

  const [revision, setRevision] = useState<number | null>(null)
  const [current, setCurrent] = useState('')
  const [pos, setPos] = useState<number>(-1)
  const [total, setTotal] = useState<number>(-1)
  const [finished, setFinished] = useState(false)
  const [cancel, setCancel] = useState<((msg: string) => Promise<any>) | null>(null)
  const [hasError, setHasError] = useState(false)

  const onProgressNotify = useCallback((data: SubversionEventMap['progressNotify']) => {
    setPos(data.pos)
    setTotal(data.total)
  }, [])
  const onWorkingNotify = useCallback((data: SubversionEventMap['workingCopyNotify']) => {
    setCurrent(localPath.getFileName(data.notify.path) ?? '')
  }, [])

  const cleanup = useMemoizedFn(() => {
    if (cancel !== null) {
      cancel('Destroyed').catch(() => {})
    }
  })

  useEffect(() => {
    if (finished) {
      return
    }
    async function call() {
      await Subversion.callOnce({
        factory: subversion,
        call: async (context: Subversion) => {
          context.on('progressNotify', onProgressNotify)
          context.on('workingCopyNotify', onWorkingNotify)
          const promise = context.export(options)
          setCancel(() => async (msg: string) => {
            await context?.cancel(msg)
            await promise
          })
          const number = await promise
          setRevision(number)
        },
        onError: (error) => {
          Toast.error({
            content: t('advancedDialogs.export.failed', { error: errorHumanString(error) }),
            stack: true,
          })
          setHasError(true)
        },
        onFinally: (context) => {
          context?.off('progressNotify', onProgressNotify)
          context?.off('workingCopyNotify', onWorkingNotify)
        },
      })
      setFinished(true)
      setCancel(null)
      // let context: Subversion | null = null
      // try {
      //   context = await subversion.context()

      //   context.on('progressNotify', onProgressNotify)
      //   context.on('workingCopyNotify', onWorkingNotify)
      //   const promise = context.export(options)
      //   setCancel(() => async (msg: string) => {
      //     await context?.cancel(msg)
      //     await promise
      //   })
      //   const number = await promise
      //   setRevision(number)
      // } catch (error) {
      //   console.log('Failed to export:', error)
      //   Toast.error({
      //     content: `Failed to export: ${errorHumanString(error)}`,
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
    //     cancel('Destroyed').catch((error) => {
    //       console.log('Failed to destroy task: ', error)
    //     })
    //   }
    // }
  }, [])

  const descriptions = [
    { key: t('advancedDialogs.progress.from'), value: options.fromPathOrUrl },
    { key: t('advancedDialogs.progress.to'), value: options.toPath },
    { key: t('shared.column.revision'), value: revision?.toString() ?? null },
  ]

  const onCancel = async () => {
    if (cancel !== null) {
      try {
        await cancel('Cancelled by user')
      } catch (error) {}
    }
    setCancel(null)
  }

  const onOk = () => {
    modal.resolve(!hasError)
    modal.hide()
  }

  const percent = finished ? 100 : Math.max(Number((pos / total) * 100), 0)

  const isIndeterminate = finished ? false : pos < 0 || total < 0

  return (
    <Dialog
      afterClose={modal.remove}
      closeOnEsc={false}
      okButtonProps={finished ? undefined : { style: { display: 'none' } }}
      cancelButtonProps={finished ? { style: { display: 'none' } } : undefined}
      maskClosable={false}
      closable={false}
      size="medium"
      title={t('advancedDialogs.export.progressTitle')}
      visible={modal.visible}
      onCancel={onCancel}
      onOk={onOk}
    >
      <div className={cx(flex, flex_col, gap_y_3, disable_move)}>
        <Card>
          <Descriptions size="medium" data={descriptions}></Descriptions>
        </Card>
        <DialogFormItem title={t('shared.field.progress')}>
          <div className={cx(flex_1, flex_col)}>
            <div>{t('advancedDialogs.progress.current', { name: current })}</div>
            <div className={cx(flex)}>
              <div>{t('advancedDialogs.progress.percentage', { percent })}</div>
              <div className={cx(flex_1)}></div>
              <div>{`${pos < 0 ? t('advancedDialogs.progress.unknown') : String(pos)}/${total < 0 ? t('advancedDialogs.progress.unknown') : String(total)}`}</div>
            </div>
            <Progress indeterminate={isIndeterminate} percent={percent} size="large" />
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

export function NiceExportDialog(props: { defaultPath?: string; defaultRevision?: Revision }) {
  const modal = useCurrentModal()
  return (
    <ExportDialog
      defaultPath={props.defaultPath}
      defaultRevision={props.defaultRevision}
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
    ></ExportDialog>
  )
}

export interface ExportDialogProps {
  visible: boolean
  onOk?: () => void
  onCancel?: () => void
  afterClose?: () => void
  defaultPath?: string
  defaultRevision?: Revision
}

export default function ExportDialog(props: ExportDialogProps) {
  const t = useT()
  const revisionKinds: RevisionKind[] = ['head', 'number', 'date', 'base', 'working']

  const [pegRevision, setPegRevision] = useState<Revision>('unspecified')
  const [revision, setRevision] = useState<Revision>(props.defaultRevision ?? 'head')

  const [override, setOverride] = useState(false)
  const [ignoreExternals, setIgnoreExternals] = useState(false)
  const [ignoreKeywords, setIgnoreKeywords] = useState(false)
  const [depth, setDepth] = useState<Depth>('infinity')
  // 'none' 表示使用平台默认的换行符（后端映射为 NULL）
  const [nativeEol, setNativeEol] = useState<NativeEOL>('none')
  const [fromPathOrUrl, setFromPathOrUrl] = useState(props.defaultPath ?? '')
  const [toPath, setToPath] = useState('')

  const selectFrom = async () => {
    const selected = await open({
      title: t('shared.action.selectWorkingCopy'),
      multiple: false,
      directory: true,
    })
    if (selected === null) {
      return
    }
    setFromPathOrUrl(selected)
  }

  const selectToPath = async () => {
    const selected = await open({
      title: t('shared.action.selectFolder'),
      multiple: false,
      directory: true,
    })
    if (selected === null) {
      return
    }
    setToPath(selected)
  }

  const onCancel = () => {
    props.onCancel?.()
  }

  const modal = useModal()

  const onOk = async () => {
    if (fromPathOrUrl === '') {
      Toast.error({
        content: t('shared.error.fromPathOrUrlRequired'),
        stack: true,
      })
      return
    }
    if (toPath === '') {
      Toast.error({
        content: t('shared.error.toPathRequired'),
        stack: true,
      })
      return
    }

    const options: ExportOptions = {
      fromPathOrUrl,
      toPath,
      pegRevision,
      revision,
      override,
      ignoreExternals,
      ignoreKeywords,
      depth,
      nativeEol,
    }

    const result = await modal.show(ExportingDialog, { options }).as<boolean>()

    if (result) {
      props.onOk?.()
    }
  }

  const focusElement = useRef(null)

  return (
    <Dialog
      afterClose={props.afterClose}
      size="medium"
      title={t('shared.action.export')}
      visible={props.visible}
      onCancel={onCancel}
      onOk={onOk}
      initialFocusRef={focusElement}
    >
      <ScrollArea className={cx(flex_1, min_h_0)} contentClassName={cx(flex)}>
        <div className={cx(flex_1, flex, gap_y_3, flex_col)} style={{ paddingRight: 2 }}>
          <DialogFormItem title={t('shared.field.fromPathOrUrl')}>
            <PureInput
              ref={focusElement}
              className={inputIcon}
              autoFocus
              value={fromPathOrUrl}
              onChange={(value) => setFromPathOrUrl(value)}
              addonAfter={
                <IconFolderStroked onClick={selectFrom} className={cx(cursor_pointer, button)} />
              }
            ></PureInput>
          </DialogFormItem>
          <DialogFormItem title={t('shared.field.to')}>
            <PureInput
              className={inputIcon}
              value={toPath}
              onChange={(value) => setToPath(value)}
              addonAfter={
                <IconFolderStroked onClick={selectToPath} className={cx(cursor_pointer, button)} />
              }
            ></PureInput>
          </DialogFormItem>
          <DialogFormItem title={t('shared.field.pegRevision')}>
            <RevisionSelect
              disableLayout={true}
              className={cx(flex_1)}
              kinds={['unspecified', 'head', 'number', 'date']}
              value={pegRevision}
              onChange={setPegRevision}
            ></RevisionSelect>
          </DialogFormItem>
          <DialogFormItem title={t('shared.field.revision')}>
            <RevisionSelect
              disableLayout={true}
              className={cx(flex_1)}
              kinds={revisionKinds}
              value={revision}
              onChange={setRevision}
            ></RevisionSelect>
          </DialogFormItem>
          <DialogFormItem title={t('shared.field.depth')}>
            <DepthSelect
              value={depth}
              onChange={(value) => setDepth(value)}
              className={flex_1}
            ></DepthSelect>
          </DialogFormItem>
          <DialogFormItem title={t('shared.field.lineEnding')}>
            <Select
              value={nativeEol}
              onChange={(value) => setNativeEol(value as NativeEOL)}
              clickToHide
              className={cx(flex_1)}
            >
              <Select.Option value={'none'}>{t('shared.option.platformDefault')}</Select.Option>
              <Select.Option value={'lF'}>{t('advancedDialogs.export.eolLf')}</Select.Option>
              <Select.Option value={'cRLF'}>{t('advancedDialogs.export.eolCrlf')}</Select.Option>
              <Select.Option value={'cR'}>CR</Select.Option>
            </Select>
          </DialogFormItem>
          <DialogFormItem title={t('shared.field.options')}>
            <div className={cx(flex, flex_col, gap_y_2)}>
              <Checkbox checked={override} onChange={(e) => setOverride(e.target.checked ?? false)}>
                {t('shared.option.overwriteExisting')}
              </Checkbox>
              <Checkbox
                checked={ignoreExternals}
                onChange={(e) => setIgnoreExternals(e.target.checked ?? false)}
              >
                {t('shared.option.ignoreExternals')}
              </Checkbox>
              <Checkbox
                checked={ignoreKeywords}
                onChange={(e) => setIgnoreKeywords(e.target.checked ?? false)}
              >
                {t('shared.option.ignoreKeywords')}
              </Checkbox>
            </div>
          </DialogFormItem>
        </div>
      </ScrollArea>
    </Dialog>
  )
}
