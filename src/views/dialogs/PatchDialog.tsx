import { Button, Checkbox, InputNumber, Typography } from '@douyinfe/semi-ui'
import { cx } from '@linaria/core'
import { readFile } from '@tauri-apps/plugin-fs'
import { useEffect, useState } from 'react'

import { PatchOptions } from '@/bindings/PatchOptions'
import { LazyEditor } from '@/components/monaco/LazyEditors'
import { Subversion, useSubversion } from '@/context/Subversion'
import { useT } from '@/i18n'
import { useCurrentModal } from '@/lib/multi-modal'
import {
  border_box,
  flex,
  flex_1,
  flex_col,
  gap_x_2,
  gap_y_2,
  min_h_0,
  min_w_0,
  py_2,
} from '@/styles/Classes'

import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

export interface PatchDialogProps {
  patchFile: string
  workingCopyPath: string
  visible: boolean
  onClose: () => void
  afterClose?: () => void
}

export function NicePatchDialog(props: { patchFile: string; workingCopyPath: string }) {
  const modal = useCurrentModal()

  return (
    <PatchDialog
      onClose={() => {
        modal.resolve()
        modal.hide()
      }}
      afterClose={modal.remove}
      visible={modal.visible}
      patchFile={props.patchFile}
      workingCopyPath={props.workingCopyPath}
    ></PatchDialog>
  )
}

export default function PatchDialog(props: PatchDialogProps) {
  const t = useT()
  const [content, setContent] = useState('')
  const [isRunning, setIsRunning] = useState(false)

  const [dryRun, setDryRun] = useState(false)
  const [stripCount, setStripCount] = useState<number>(0)
  const [ignoreWhitespace, setIgnoreWhitespace] = useState(false)
  const [removeTempfiles, setRemoveTempfiles] = useState(true)
  const [reverse, setReverse] = useState(false)

  const subversion = useSubversion()

  useEffect(() => {
    const call = async () => {
      const file = await readFile(props.patchFile)

      const decoder = new TextDecoder('utf-8', { fatal: true })
      setContent(decoder.decode(file))
    }

    call()
  }, [props.patchFile])

  const execute = async () => {
    const options: PatchOptions = {
      patchAbsolutePath: props.patchFile,
      wcAbsolutePath: props.workingCopyPath,
      dryRun,
      stripCount,
      reverse,
      ignoreWhitespace,
      removeTempfiles,
    }
    setIsRunning(true)
    await Subversion.callOnce({
      factory: subversion,
      async call(context) {
        await context.patch(options)
        props.onClose()
      },
    })
    setIsRunning(false)
  }

  return (
    <Dialog
      afterClose={props.afterClose}
      footerMargin="0px"
      fullScreen
      title={t('shared.action.patch')}
      closable={false}
      visible={props.visible}
      footer={<></>}
      closeIconDisabled={isRunning}
      maskClosable={!isRunning}
      closeOnEsc={!isRunning}
    >
      <div className={cx(flex, flex_col, min_h_0, gap_y_2, flex_1, border_box, py_2)}>
        <DialogFormItem title={t('shared.field.path')}>
          <Typography.Text>{props.patchFile}</Typography.Text>
        </DialogFormItem>
        <DialogFormItem
          className={cx(flex_1, min_h_0)}
          wrapperClassName={min_h_0}
          title={t('shared.field.result')}
        >
          <div className={cx(flex, flex_1, min_h_0, gap_x_2)}>
            <div className={cx(flex_1, min_h_0, min_w_0)}>
              <LazyEditor
                value={content}
                options={{
                  readOnly: true,
                }}
              ></LazyEditor>
            </div>
            <div className={cx(flex, flex_col, min_h_0)}>
              <div className={cx(flex_1, min_h_0)}></div>
              <DialogFormItem title={t('shared.field.stripCount')}>
                <InputNumber
                  value={stripCount}
                  onChange={(v) => {
                    if (typeof v === 'number') {
                      setStripCount(v)
                    }
                  }}
                  min={0}
                  parser={(value) => value.replace(/[^\d]/g, '')}
                  formatter={(value) => `${value}`.replace(/[^\d]/g, '')}
                ></InputNumber>
              </DialogFormItem>
              <DialogFormItem title={t('shared.field.settings')}>
                <div className={cx(flex_1, flex, flex_col, min_w_0)}>
                  <Checkbox
                    disabled={isRunning}
                    checked={dryRun}
                    onChange={(e) => setDryRun(e.target.checked ?? false)}
                  >
                    {t('shared.option.dryRun')}
                  </Checkbox>
                  <Checkbox
                    disabled={isRunning}
                    checked={reverse}
                    onChange={(e) => setReverse(e.target.checked ?? false)}
                  >
                    {t('shared.option.reverse')}
                  </Checkbox>
                  <Checkbox
                    disabled={isRunning}
                    checked={ignoreWhitespace}
                    onChange={(e) => setIgnoreWhitespace(e.target.checked ?? false)}
                  >
                    {t('shared.option.ignoreWhitespace')}
                  </Checkbox>
                  <Checkbox
                    disabled={isRunning}
                    checked={removeTempfiles}
                    onChange={(e) => setRemoveTempfiles(e.target.checked ?? false)}
                  >
                    {t('shared.option.removeTempFiles')}
                  </Checkbox>
                </div>
              </DialogFormItem>
              <DialogFormItem title={t('shared.field.actions')}>
                <div className={cx(flex, flex_1, gap_x_2)}>
                  <div className={cx(flex_1)}></div>
                  <Button disabled={isRunning} onClick={props.onClose}>
                    {t('shared.action.close')}
                  </Button>
                  <Button loading={isRunning} onClick={execute} theme={'solid'} type={'primary'}>
                    {t('shared.action.apply')}
                  </Button>
                </div>
              </DialogFormItem>
              <div style={{ height: 10 }}></div>
            </div>
          </div>
        </DialogFormItem>
      </div>
    </Dialog>
  )
}
