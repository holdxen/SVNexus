import { Button, Card, Checkbox, Radio, RadioGroup, Toast } from '@douyinfe/semi-ui'
import { cx } from '@linaria/core'
import { useEffect, useRef, useState } from 'react'

import { CommitOptions } from '@/bindings/CommitOptions'
import { Depth } from '@/bindings/Depth'
import { StatusOptions } from '@/bindings/StatusOptions'
import PureTextArea from '@/components/PureTextArea'
import { ScrollArea } from '@/components/ScrollArea'
import DepthSelect from '@/components/subversion/DepthSelect'
import { Subversion, useSubversion } from '@/context/Subversion'
import { useCurrentModal } from '@/lib/multi-modal'
import { list_item, list_item_selected } from '@/styles/Components'
import {
  border_box,
  flex,
  flex_1,
  flex_col,
  flex_row_reverse,
  gap_x_2,
  gap_x_4,
  gap_y_1,
  gap_y_2,
  hidden,
  min_h_0,
  min_w_0,
  overflow_hidden,
  pb_3,
  whitespace_nowrap,
} from '@/styles/Classes'
import { SingleTaskQueue } from '@/utils/Queue'

import {
  fromStatusEntry,
  WorkingCopyItem,
  WorkingCopyItemModel,
  WorkingCopyPathItemModel,
} from '../WorkspaceView/WorkingCopyItem'
import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

export const NiceCommitDialog = (props: {
  items: WorkingCopyPathItemModel[]
  relateTo?: string
}) => {
  const { visible, resolve, hide, remove } = useCurrentModal()
  return (
    <CommitDialog
      relateTo={props.relateTo}
      afterClose={() => {
        console.log('remove')
        remove()
      }}
      items={props.items}
      visible={visible}
      onOk={() => {
        resolve(true)
        hide()
      }}
      onCancel={() => {
        resolve(false)
        hide()
      }}
    ></CommitDialog>
  )
}

export interface CommitDialogProps {
  items: WorkingCopyPathItemModel[]

  visible: boolean
  onOk: () => void
  onCancel: () => void
  afterClose?: () => void
  getPopupContainer?: () => HTMLElement
  relateTo?: string
}

export default function CommitDialog(props: CommitDialogProps) {
  const [selected, setSelected] = useState<string | null>(null)
  const [depth, setDepth] = useState<Depth>('infinity')
  const [commitMessage, setCommitMessage] = useState('')
  const [keepLocks, setKeepLocks] = useState(false)
  const [includeExternals, setIncludeExternals] = useState(false)
  const [commitAsOperations, setCommitAsdOperations] = useState(true)
  const subversion = useSubversion()
  const [isRunning, setIsRunning] = useState(false)
  const selectView = 'Select'
  const actualView = 'Actual'

  const [actualItems, setActualItems] = useState<WorkingCopyPathItemModel[]>([])
  const queue = useRef(new SingleTaskQueue())
  const [actualSelected, setActualSelected] = useState<string | null>(null)

  const [display, setDisplay] = useState(selectView)

  const onOk = async () => {
    if (commitMessage === '') {
      Toast.error({
        content: 'Commit message is empty',
        stack: true,
      })
      return
    }

    const options: CommitOptions = {
      targets: props.items.map((e) => e.path),
      depth,
      keepLocks,
      keepChangelist: false,
      commitAsOperations,
      includeDirExternals: includeExternals,
      includeFileExternals: includeExternals,
      changelists: null,
      revisionPropertyTable: null,
      commitMessage,
    }

    setIsRunning(true)
    await Subversion.callOnce({
      factory: subversion,
      async call(context) {
        const revision = await context.commit(options)
        let message = 'Commit successfully'
        if (revision.info) {
          message = `Commit successfully at (revision ${revision.info.revision})`
        }
        Toast.success({
          content: message,
          stack: true,
        })
        props.onOk()
      },
    })
    setIsRunning(false)
  }

  useEffect(() => {
    const call = async (signal: AbortSignal) => {
      if (signal.aborted) return
      const map = new Map<string, WorkingCopyItemModel & { path: string }>()
      await Subversion.callOnce({
        factory: subversion,
        call: async (context) => {
          if (signal.aborted) return

          for (let i of props.items) {
            const statusOptions: StatusOptions = {
              path: i.path,
              revision: 'working',
              depth,
              getAll: false,
              checkOutOfDate: false,
              checkWorkingCopy: false,
              noIgnore: false,
              ignoreExternals: false,
              depthAsSticky: false,
              changelist: null,
            }
            if (signal.aborted) return
            const result = (await context.status(statusOptions)).entries
            for (let j of result) {
              if (j.nodeStatus === 'unversioned' || j.nodeStatus === 'missing') {
                continue
              }
              map.set(j.path, { ...fromStatusEntry(j, false, props.relateTo), path: j.path })
            }
          }

          if (signal.aborted) return

          setActualItems(Array.from(map.values()))
        },
      })
    }

    queue.current.run(call)
  }, [depth, includeExternals])

  return (
    <Dialog
      size="medium"
      afterClose={props.afterClose}
      onCancel={props.onCancel}
      footer={<></>}
      title="Commit"
      visible={props.visible}
      closeIconDisabled={isRunning}
      maskClosable={!isRunning}
      closeOnEsc={!isRunning}
    >
      <div className={cx(flex, flex_1, min_h_0, gap_x_4, border_box, pb_3)}>
        <div className={cx(flex, flex_col, gap_y_2, min_h_0, flex_1)}>
          <div>
            <RadioGroup value={display} onChange={(e) => setDisplay(e.target.value)} type="button">
              <Radio value={selectView}>Select</Radio>
              <Radio value={actualView}>Actual</Radio>
            </RadioGroup>
          </div>
          <Card
            className={cx(min_h_0, flex_1, flex)}
            headerStyle={{ padding: '0.25rem' }}
            bodyStyle={{ padding: '0.5rem', minHeight: '0px', display: 'flex', flex: '1' }}
          >
            <ScrollArea
              className={cx(flex_1, min_h_0, min_w_0)}
              contentClassName={cx(min_w_0, min_h_0)}
            >
              <div className={cx(flex, flex_col, display !== selectView && hidden)}>
                {props.items.map((item) => {
                  return (
                    <WorkingCopyItem
                      key={item.path}
                      showRelativeDirectory={true}
                      onClick={() => {
                        setSelected(item.path)
                      }}
                      {...item}
                      className={cx(
                        list_item,
                        overflow_hidden,
                        whitespace_nowrap,
                        selected === item.path && list_item_selected,
                      )}
                    ></WorkingCopyItem>
                  )
                })}
              </div>
              <div className={cx(flex, flex_col, display !== actualView && hidden)}>
                {actualItems.map((item) => {
                  return (
                    <WorkingCopyItem
                      key={item.path}
                      showRelativeDirectory={true}
                      onClick={() => {
                        setActualSelected(item.path)
                      }}
                      {...item}
                      className={cx(
                        list_item,
                        overflow_hidden,
                        whitespace_nowrap,
                        actualSelected === item.path && list_item_selected,
                      )}
                    ></WorkingCopyItem>
                  )
                })}
              </div>
            </ScrollArea>
          </Card>
        </div>
        <div className={cx(flex, flex_col, min_h_0, gap_y_1)}>
          <div className={cx(flex_1, min_h_0)}></div>
          <DialogFormItem title="Depth:">
            <DepthSelect
              className={cx(flex_1)}
              value={depth}
              onChange={(e) => setDepth(e)}
            ></DepthSelect>
          </DialogFormItem>
          <DialogFormItem title="Options:">
            <div className={cx(flex_1, flex, flex_col)}>
              <Checkbox
                checked={keepLocks}
                onChange={(e) => setKeepLocks(e.target.checked ?? false)}
              >
                Keep locks
              </Checkbox>
              <Checkbox
                checked={includeExternals}
                onChange={(e) => setIncludeExternals(e.target.checked ?? false)}
              >
                Include externals
              </Checkbox>
              <Checkbox
                checked={commitAsOperations}
                onChange={(e) => setCommitAsdOperations(e.target.checked ?? false)}
              >
                Commit as operations
              </Checkbox>
            </div>
          </DialogFormItem>
          <DialogFormItem title="Commit message:">
            <PureTextArea
              autoFocus
              value={commitMessage}
              onChange={(e) => setCommitMessage(e)}
            ></PureTextArea>
          </DialogFormItem>
          <DialogFormItem title="Actions:">
            <div className={cx(flex, flex_1, flex_row_reverse, gap_x_2)}>
              <Button loading={isRunning} onClick={onOk} theme="solid" type="primary">
                Commit
              </Button>
              <Button disabled={isRunning} onClick={props.onCancel}>
                Cancel
              </Button>
            </div>
          </DialogFormItem>
        </div>
      </div>
    </Dialog>
  )
}
