import { Button, Checkbox, InputNumber, Typography } from '@douyinfe/semi-ui'
import { cx } from '@linaria/core'
import { Editor } from '@monaco-editor/react'
import { readFile } from '@tauri-apps/plugin-fs'
import { useEffect, useState } from 'react'

import { PatchOptions } from '@/bindings/PatchOptions'
import { Subversion, useSubversion } from '@/context/Subversion'
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
      title="Patch"
      closable={false}
      visible={props.visible}
      footer={<></>}
      closeIconDisabled={isRunning}
      maskClosable={!isRunning}
      closeOnEsc={!isRunning}
    >
      <div className={cx(flex, flex_col, min_h_0, gap_y_2, flex_1, border_box, py_2)}>
        <DialogFormItem title="Path:">
          <Typography.Text>{props.patchFile}</Typography.Text>
        </DialogFormItem>
        <DialogFormItem className={cx(flex_1, min_h_0)} wrapperClassName={min_h_0} title="Result:">
          <div className={cx(flex, flex_1, min_h_0, gap_x_2)}>
            <div className={cx(flex_1, min_h_0, min_w_0)}>
              <Editor
                value={content}
                options={{
                  readOnly: true,
                }}
              ></Editor>
            </div>
            <div className={cx(flex, flex_col, min_h_0)}>
              <div className={cx(flex_1, min_h_0)}></div>
              <DialogFormItem title="Strip count:">
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
              <DialogFormItem title="Settings:">
                <div className={cx(flex_1, flex, flex_col, min_w_0)}>
                  <Checkbox
                    disabled={isRunning}
                    checked={dryRun}
                    onChange={(e) => setDryRun(e.target.checked ?? false)}
                  >
                    Dry run
                  </Checkbox>
                  <Checkbox
                    disabled={isRunning}
                    checked={reverse}
                    onChange={(e) => setReverse(e.target.checked ?? false)}
                  >
                    Reverse
                  </Checkbox>
                  <Checkbox
                    disabled={isRunning}
                    checked={ignoreWhitespace}
                    onChange={(e) => setIgnoreWhitespace(e.target.checked ?? false)}
                  >
                    Ignore whitespace
                  </Checkbox>
                  <Checkbox
                    disabled={isRunning}
                    checked={removeTempfiles}
                    onChange={(e) => setRemoveTempfiles(e.target.checked ?? false)}
                  >
                    Remove temp files
                  </Checkbox>
                </div>
              </DialogFormItem>
              <DialogFormItem title="Action:">
                <div className={cx(flex, flex_1, gap_x_2)}>
                  <div className={cx(flex_1)}></div>
                  <Button disabled={isRunning} onClick={props.onClose}>
                    Close
                  </Button>
                  <Button loading={isRunning} onClick={execute} theme={'solid'} type={'primary'}>
                    Apply
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
