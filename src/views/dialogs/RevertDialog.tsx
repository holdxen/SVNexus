import { Card, Checkbox } from '@douyinfe/semi-ui'
import { cx } from '@linaria/core'
import { useState } from 'react'

import { Depth } from '@/bindings/Depth'
import { RevertOptions } from '@/bindings/RevertOptions'
import { Subversion, useSubversion } from '@/context/Subversion'
import { useCurrentModal } from '@/lib/multi-modal'
import { disable_move, list_item, list_item_selected } from '@/styles/Components'
import {
  flex,
  flex_col,
  gap_y_2,
  min_h_0,
  min_w_0,
  overflow_hidden,
  whitespace_nowrap,
} from '@/styles/Classes'

import DepthSelect from '../../components/subversion/DepthSelect'
import { WorkingCopyItem, WorkingCopyPathItemModel } from '../WorkspaceView/WorkingCopyItem'
import { Dialog } from './Dialog'

export function NiceRevertDialog(props: { items: WorkingCopyPathItemModel[] }) {
  const modal = useCurrentModal()

  return (
    <RevertDialog
      afterClose={modal.remove}
      onCancel={() => {
        modal.resolve(false)
        modal.hide()
      }}
      onOk={() => {
        modal.resolve(true)
        modal.hide()
      }}
      items={props.items}
      visible={modal.visible}
    ></RevertDialog>
  )
}

export interface RevertDialogProps {
  items: WorkingCopyPathItemModel[]
  visible: boolean
  onOk: () => void
  onCancel: () => void
  afterClose?: () => void
}

export default function RevertDialog(props: RevertDialogProps) {
  const [addedKeepLocal, setAddedKeepLocal] = useState(true)
  const [clearChangelists, setCleaChangelists] = useState(false)
  const [selected, setSelected] = useState<string | null>(null)
  // const [visible, setVisible] = useState(true)
  const [depth, setDepth] = useState<Depth>('empty')
  const [metadataOnly, setMetadataOnly] = useState(false)
  const subversion = useSubversion()
  const onCancel = () => {
    props.onCancel()
  }
  const onOk = async () => {
    const options: RevertOptions = {
      paths: props.items.map((e) => e.path),
      depth,
      changelists: null,
      clearChangelists,
      metadataOnly,
      addedKeepLocal,
    }
    await Subversion.callOnce({
      factory: subversion,
      async call(context) {
        await context.revert(options)
        props.onOk()
      },
    })
    // try {
    //   const options: RevertOptions = {
    //     paths: props.items.map((e) => e.path),
    //     depth,
    //     changelists: null,
    //     clearChangelists,
    //     metadataOnly,
    //     addedKeepLocal,
    //   }
    //   const context = await subversion.context()
    //   await context.revert(options)
    //   props.onOk()
    // } catch (error) {
    //   Toast.error({
    //     content: `Failed to revert: ${errorHumanString(error)}`,
    //     stack: true,
    //   })
    // }
  }
  return (
    <Dialog
      afterClose={props.afterClose}
      title={'Revert'}
      visible={props.visible}
      onCancel={onCancel}
      onOk={onOk}
    >
      <div className={cx(flex, min_h_0, flex_col, gap_y_2, disable_move)}>
        <Card
          className={cx(min_h_0)}
          headerStyle={{ padding: '0.25rem' }}
          bodyStyle={{ padding: '0.5rem', minHeight: '0px', overflowY: 'auto' }}
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
        <div style={{ color: 'rgba(var(--semi-grey-9), 1)' }}>Depth:</div>
        <DepthSelect value={depth} onChange={(value) => setDepth(value)}></DepthSelect>
        <Checkbox
          checked={clearChangelists}
          onChange={(e) => setCleaChangelists(e.target.checked ?? false)}
        >
          Clear Change lists
        </Checkbox>
        <Checkbox
          checked={metadataOnly}
          onChange={(e) => setMetadataOnly(e.target.checked ?? false)}
        >
          No ignore
        </Checkbox>
        <Checkbox
          checked={addedKeepLocal}
          onChange={(e) => setAddedKeepLocal(e.target.checked ?? false)}
        >
          Keep local
        </Checkbox>
      </div>
    </Dialog>
  )
}
