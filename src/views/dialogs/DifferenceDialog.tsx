import { Button, Checkbox, Select, Typography } from '@douyinfe/semi-ui'
import { cx } from '@linaria/core'
import { save } from '@tauri-apps/plugin-dialog'
import { writeTextFile } from '@tauri-apps/plugin-fs'
import { useEffect, useMemo, useState } from 'react'

import { ClientDifferenceOptions } from '@/bindings/ClientDifferenceOptions'
import { ClientDifferenceSource } from '@/bindings/ClientDifferenceSource'
import { Depth } from '@/bindings/Depth'
import { LazyEditor } from '@/components/monaco/LazyEditors'
import DepthSelect from '@/components/subversion/DepthSelect'
import { Subversion, useSubversion } from '@/context/Subversion'
import { useCurrentModal } from '@/lib/multi-modal'
import {
  border_box,
  flex,
  flex_1,
  flex_col,
  gap_x_2,
  gap_y_2,
  hidden,
  min_h_0,
  min_w_0,
  py_2,
} from '@/styles/Classes'
import { SingleTaskQueue } from '@/utils/Queue'

import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

export interface DifferenceDialogProps {
  source: ClientDifferenceSource
  visible: boolean
  onClose: () => void
  afterClose?: () => void
  workspace: string
  workingCopy: string
}

export function NiceDifferenceDialog(props: {
  source: ClientDifferenceSource
  workspace: string
  workingCopy: string
}) {
  const modal = useCurrentModal()

  return (
    <DifferenceDialog
      workingCopy={props.workingCopy}
      workspace={props.workspace}
      onClose={() => {
        modal.resolve()
        modal.hide()
      }}
      afterClose={modal.remove}
      visible={modal.visible}
      source={props.source}
    ></DifferenceDialog>
  )
}

type DisplayContent = 'Both' | 'ContentOnly' | 'PropertyOnly'
type RelativePath = 'None' | 'Path' | 'Root'

export default function DifferenceDialog(props: DifferenceDialogProps) {
  const displayContentAll: DisplayContent[] = ['Both', 'ContentOnly', 'PropertyOnly']
  const [content, setContent] = useState('')
  const [ignoreAncestry, setIgnoreAncestry] = useState(false)
  const [noAdded, setNoAdded] = useState(false)
  const [showCopiesAsAdds, setShowCopiesAsAdds] = useState(false)
  const [ignoreContentType, setIgnoreContentType] = useState(false)
  const [gitFormat, setGitFormat] = useState(false)
  const [pretty, setPretty] = useState(false)
  // const [relateToRoot, setRelateToRoot] = useState(false)
  const [displayContent, setDisplayContent] = useState<DisplayContent>(displayContentAll[0])
  const [depth, setDepth] = useState<Depth>('infinity')
  const [noDeleted, setNoDeleted] = useState(false)
  const [isExporting, setIsExporting] = useState(false)
  const [relateTo, setRelateTo] = useState<RelativePath>('None')

  const path = props.workingCopy
  const root = props.workspace

  const relateToPath = (relate: RelativePath) => {
    switch (relate) {
      case 'None':
        return null
      case 'Path':
        return path
      case 'Root':
        return root
      default:
        throw new Error('Unknown path: ' + path)
    }
  }

  const subversion = useSubversion()

  let path1 = ''
  let path2: string | null = null
  if ('peg' in props.source) {
    path1 = props.source.peg.path
  } else if ('target' in props.source) {
    path1 = props.source.target.path1
    path2 = props.source.target.path2
  }

  const queue = useMemo(() => {
    let queue = new SingleTaskQueue()
    queue.single = true
    return queue
  }, [])

  const execute = async (signal: AbortSignal) => {
    if (signal.aborted) {
      return
    }

    const options: ClientDifferenceOptions = {
      options: null,
      source: props.source,
      relateTo: relateToPath(relateTo),
      depth,
      ignoreAncestry,
      noAdded,
      noDeleted,
      showCopiesAsAdds,
      ignoreContentType,
      ignoreProperties: displayContent === 'ContentOnly',
      propertiesOnly: displayContent === 'PropertyOnly',
      useGitFormat: gitFormat,
      prettyPrintMergeInfo: pretty,
      headerEncoding: 'UTF-8',
      changelists: null,
    }

    await Subversion.callOnce({
      factory: subversion,
      call: async (context) => {
        if (signal.aborted) {
          return
        }
        const result = await context.difference(options)
        if (signal.aborted) {
          return
        }
        const textDecoder = new TextDecoder('utf-8', {
          fatal: true,
        })
        const decoded = textDecoder.decode(result.out)
        setContent(decoded)
      },
    })

    // try {
    //   const context = await subversion.context()
    //   const result = await context.difference(options)
    //   const textDecoder = new TextDecoder('utf-8', {
    //     fatal: true,
    //   })

    //   console.log('diff result:', result)

    //   const decoded = textDecoder.decode(result.out)
    //   setContent(decoded)
    // } catch (error) {
    //   console.error('Failed to execute difference:', error)
    // }
    // if (signal.aborted) {
    //   return
    // }
  }

  const exportTask = async () => {
    const filePath = await save({
      title: '保存文件',
      defaultPath: 'export.patch',
    })

    try {
      if (filePath) {
        await writeTextFile(filePath, content)
      }
    } catch (error) {
      console.error('Failed to execute difference for export:', error)
      return
    }
  }

  const doExport = async () => {
    setIsExporting(true)
    try {
      await queue.runAndWait(exportTask)
    } catch (error) {
      console.error('Failed to export file:', error)
    } finally {
      setIsExporting(false)
    }
  }

  useEffect(() => {
    return () => {
      queue.dispose()
    }
  })

  useEffect(() => {
    queue.run(execute)
  }, [
    depth,
    ignoreAncestry,
    noAdded,
    showCopiesAsAdds,
    ignoreContentType,
    gitFormat,
    pretty,
    relateTo,
    displayContent,
    props.source,
  ])

  return (
    <Dialog
      afterClose={props.afterClose}
      footerMargin="0px"
      fullScreen
      title="Difference"
      closable={false}
      visible={props.visible}
      footer={<></>}
    >
      <div className={cx(flex, flex_col, min_h_0, gap_y_2, flex_1, border_box, py_2)}>
        <DialogFormItem title="Path:">
          <div className={cx(flex_1, flex_col, flex)}>
            <Typography.Text>{path1}</Typography.Text>
            <Typography.Text className={cx(path2 === null && hidden)}>{path2}</Typography.Text>
          </div>
        </DialogFormItem>
        <DialogFormItem className={cx(flex_1, min_h_0)} wrapperClassName={min_h_0} title="Result:">
          <div className={cx(flex, flex_1, min_h_0, gap_x_2)}>
            <div className={cx(flex_1, min_h_0, min_w_0)}>
              <LazyEditor
                value={content}
                options={{
                  readOnly: true,
                  fixedOverflowWidgets: true,
                }}
              ></LazyEditor>
            </div>
            <div className={cx(flex, flex_col, min_h_0)}>
              <div className={cx(flex_1, min_h_0)}></div>
              <DialogFormItem title="Depth:">
                <DepthSelect
                  disable={isExporting}
                  value={depth}
                  onChange={setDepth}
                  className={cx(flex_1)}
                ></DepthSelect>
              </DialogFormItem>
              <DialogFormItem title="Display:">
                <Select
                  clickToHide
                  disabled={isExporting}
                  className={flex_1}
                  value={displayContent}
                  onSelect={(value) => setDisplayContent(value as DisplayContent)}
                >
                  {displayContentAll.map((e) => {
                    return <Select.Option key={e} value={e}></Select.Option>
                  })}
                </Select>
              </DialogFormItem>
              <DialogFormItem title="Relate:">
                <Select
                  clickToHide
                  className={cx(flex_1)}
                  value={relateTo}
                  onSelect={(e) => {
                    setRelateTo(e as RelativePath)
                  }}
                >
                  <Select.Option value={'None'}></Select.Option>
                  <Select.Option value={'Path'}></Select.Option>
                  <Select.Option value={'Root'}></Select.Option>
                </Select>
              </DialogFormItem>
              <DialogFormItem title="Settings:">
                <div className={cx(flex_1, flex, flex_col, min_w_0)}>
                  <Checkbox
                    disabled={isExporting}
                    checked={ignoreAncestry}
                    onChange={(e) => setIgnoreAncestry(e.target.checked ?? false)}
                  >
                    Ignore ancestry
                  </Checkbox>
                  <Checkbox
                    disabled={isExporting}
                    checked={noAdded}
                    onChange={(e) => setNoAdded(e.target.checked ?? false)}
                  >
                    No added
                  </Checkbox>
                  <Checkbox
                    disabled={isExporting}
                    checked={noDeleted}
                    onChange={(e) => setNoDeleted(e.target.checked ?? false)}
                  >
                    No deleted
                  </Checkbox>
                  <Checkbox
                    disabled={isExporting}
                    checked={showCopiesAsAdds}
                    onChange={(e) => setShowCopiesAsAdds(e.target.checked ?? false)}
                  >
                    Show copies as adds
                  </Checkbox>
                  <Checkbox
                    disabled={isExporting}
                    checked={ignoreContentType}
                    onChange={(e) => setIgnoreContentType(e.target.checked ?? false)}
                  >
                    Ignore content type
                  </Checkbox>
                  <Checkbox
                    disabled={isExporting}
                    checked={gitFormat}
                    onChange={(e) => setGitFormat(e.target.checked ?? false)}
                  >
                    Git format
                  </Checkbox>
                  <Checkbox
                    disabled={isExporting}
                    checked={pretty}
                    onChange={(e) => setPretty(e.target.checked ?? false)}
                  >
                    Pretty
                  </Checkbox>
                  {/*<Checkbox
                    disabled={isExporting}
                    checked={relateToRoot}
                    onChange={(e) => setRelateToRoot(e.target.checked ?? false)}
                  >
                    Relate to root
                  </Checkbox>*/}
                </div>
              </DialogFormItem>
              <DialogFormItem title="Action:">
                <div className={cx(flex, flex_1, gap_x_2)}>
                  <div className={cx(flex_1)}></div>
                  <Button disabled={isExporting} onClick={props.onClose}>
                    Close
                  </Button>
                  <Button loading={isExporting} onClick={doExport} theme={'solid'} type={'primary'}>
                    Export
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
