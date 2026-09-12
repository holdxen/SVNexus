import { Card, Checkbox, Toast } from '@douyinfe/semi-ui'
import { cx } from '@linaria/core'
import { useState } from 'react'

import { Depth } from '@/bindings/Depth'
import { Revision } from '@/bindings/Revision'
import { UpdateOptions } from '@/bindings/UpdateOptions'
import DepthSelect from '@/components/subversion/DepthSelect'
import RevisionSelect from '@/components/subversion/RevisionSelect'
import { Subversion, useSubversion } from '@/context/Subversion'
import { useCurrentModal } from '@/lib/multi-modal'
import { disable_move, list_item, list_item_selected } from '@/styles/Components'
import {
  flex,
  min_h_0,
  flex_col,
  gap_y_2,
  min_w_0,
  overflow_hidden,
  whitespace_nowrap,
  flex_1,
} from '@/styles/Classes'
import errorHumanString from '@/utils/Error'
import { delay } from '@/utils/Time'

import { WorkingCopyItem, WorkingCopyPathItemModel } from '../WorkspaceView/WorkingCopyItem'
import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

export function NiceUpdateDialog(props: { items: WorkingCopyPathItemModel[] }) {
  const modal = useCurrentModal()

  return (
    <UpdateDialog
      visible={modal.visible}
      items={props.items}
      onCancel={() => {
        modal.resolve(false)
        modal.hide()
      }}
      onOk={() => {
        modal.resolve(true)
        modal.hide()
      }}
      afterClose={modal.remove}
    ></UpdateDialog>
  )
}

export interface UpdateDialogProps {
  items: WorkingCopyPathItemModel[]
  visible: boolean
  onOk: () => void
  onCancel: () => void
  afterClose?: () => void
}

export default function UpdateDialog(props: UpdateDialogProps) {
  const [depth, setDepth] = useState<Depth>('infinity')
  const [depthIsSticky, setDepthIsSticky] = useState(false)
  const [ignoreExternals, setIgnoreExternals] = useState(false)
  const [addsAsModification, setAddsAsModification] = useState(false)
  const [makeParents, setMakeParents] = useState(false)
  const [allowUnverObstructions, setAllowUnverObstructions] = useState(false)
  const [revision, setRevision] = useState<Revision>('head')
  const [selected, setSelected] = useState<string | null>(null)
  const subversion = useSubversion()

  const onCancel = () => {
    props.onCancel()
  }

  const onOk = async () => {
    await Subversion.callOnce({
      factory: subversion,
      call: async (context) => {
        const options: UpdateOptions = {
          paths: props.items.map((e) => e.path),
          depth,
          revision,
          depthIsSticky,
          ignoreExternals,
          addsAsModification,
          makeParents,
          allowUnverObstructions,
        }

        const updatedRevision = await context.update(options)
        if (updatedRevision.length === props.items.length) {
          for (let i = 0; i < props.items.length; i++) {
            const number = updatedRevision[i]
            const path = props.items[i].path
            if (number === null) {
              Toast.success({
                content: `Update ${path} successfully`,
                stack: true,
              })
            } else {
              Toast.success({
                content: `Update ${path} successfully at r${number}`,
                stack: true,
              })
            }
            await delay(200)
          }
        }
        props.onOk()
      },
      onError: (error) => {
        Toast.error({
          content: `Failed to update: ${errorHumanString(error)}`,
          stack: true,
        })
      },
    })
    // const context = await subversion.context()
    // try {
    //   const options: UpdateOptions = {
    //     paths: props.items.map((e) => e.path),
    //     depth,
    //     revision,
    //     depthIsSticky,
    //     ignoreExternals,
    //     addsAsModification,
    //     makeParents,
    //     allowUnverObstructions,
    //   }

    //   const updatedRevision = await context.update(options)
    //   if (updatedRevision.length === props.items.length) {
    //     for (let i = 0; i < props.items.length; i++) {
    //       const number = updatedRevision[i]
    //       const path = props.items[i].path
    //       if (number === null) {
    //         Toast.success({
    //           content: `Update ${path} successfully`,
    //           stack: true,
    //         })
    //       } else {
    //         Toast.success({
    //           content: `Update ${path} successfully at r${number}`,
    //           stack: true,
    //         })
    //       }
    //       await delay(200)
    //     }
    //   }
    //   props.onOk()
    // } catch (error) {
    //   Toast.error({
    //     content: `Failed to update: ${errorHumanString(error)}`,
    //     stack: true,
    //   })
    // } finally {
    // }
  }

  return (
    <Dialog
      size="medium"
      afterClose={props.afterClose}
      title={'Update'}
      visible={props.visible}
      onCancel={onCancel}
      onOk={onOk}
    >
      <div className={cx(flex, min_h_0, flex_col, gap_y_2, disable_move)}>
        <Card
          className={cx(min_h_0, flex)}
          headerStyle={{ padding: '0.25rem' }}
          bodyStyle={{ padding: '0.5rem', minHeight: '0px', overflowY: 'auto', flex: '1' }}
        >
          <div className={cx(min_w_0, min_h_0)} style={{ maxHeight: 150 }}>
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
        </Card>
        {/*<Text strong style={{ color: 'rgba(var(--semi-grey-9), 1)' }}>Items:</Text>*/}
        <DialogFormItem title="Revision:">
          <RevisionSelect
            className={flex_1}
            disableLayout={true}
            kinds={['head', 'number', 'committed', 'previous', 'date']}
            value={revision}
            onChange={setRevision}
          ></RevisionSelect>
        </DialogFormItem>
        <DialogFormItem title="Depth:">
          <DepthSelect
            className={flex_1}
            value={depth}
            onChange={(value) => setDepth(value)}
          ></DepthSelect>
        </DialogFormItem>
        <Checkbox
          checked={depthIsSticky}
          onChange={(e) => setDepthIsSticky(e.target.checked ?? false)}
        >
          Depth is sticky
        </Checkbox>
        <Checkbox
          checked={ignoreExternals}
          onChange={(e) => setIgnoreExternals(e.target.checked ?? false)}
        >
          Ignore externals
        </Checkbox>
        <Checkbox
          checked={allowUnverObstructions}
          onChange={(e) => setAllowUnverObstructions(e.target.checked ?? false)}
        >
          AllowUnverObstructions
        </Checkbox>
        <Checkbox
          checked={addsAsModification}
          onChange={(e) => setAddsAsModification(e.target.checked ?? false)}
        >
          Add as modification
        </Checkbox>
        <Checkbox checked={makeParents} onChange={(e) => setMakeParents(e.target.checked ?? false)}>
          Make parents
        </Checkbox>
      </div>
    </Dialog>
  )
}
