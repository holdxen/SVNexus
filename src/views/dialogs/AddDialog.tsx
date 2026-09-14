import { Card, Checkbox, Toast } from '@douyinfe/semi-ui'
import { cx } from '@linaria/core'
import { useState } from 'react'

import { AddOptions } from '@/bindings/AddOptions'
import { Depth } from '@/bindings/Depth'
import { ScrollArea } from '@/components/ScrollArea'
import { Subversion, useSubversion } from '@/context/Subversion'
import { useCurrentModal } from '@/lib/multi-modal'
import {
  flex,
  flex_1,
  flex_col,
  gap_y_2,
  min_h_0,
  min_w_0,
  overflow_hidden,
  overflow_visible,
  whitespace_nowrap,
} from '@/styles/Classes'
import { disable_move, list_item, list_item_selected } from '@/styles/Components'
import errorHumanString from '@/utils/Error'

import DepthSelect from '../../components/subversion/DepthSelect'
import { WorkingCopyItem, WorkingCopyPathItemModel } from '../WorkspaceView/WorkingCopyItem'
import { Dialog } from './Dialog'

export function NiceAddDialog(props: { items: WorkingCopyPathItemModel[] }) {
  const modal = useCurrentModal()

  return (
    <AddDialog
      visible={modal.visible}
      onOk={() => {
        modal.resolve(true)
        modal.hide()
      }}
      onCancel={() => {
        modal.resolve(true)
        modal.hide()
      }}
      afterClose={modal.remove}
      items={props.items}
    ></AddDialog>
  )
}

export interface AddDialogProps {
  items: WorkingCopyPathItemModel[]
  visible: boolean
  onOk: () => void
  onCancel: () => void
  afterClose?: () => void
}

export default function AddDialog(props: AddDialogProps) {
  const [selected, setSelected] = useState<string | null>(null)
  const [depth, setDepth] = useState<Depth>('infinity')
  // const [visible, setVisible] = useState(true)
  const [force, setForce] = useState(false)
  const [noIgnore, setNoIgnore] = useState(false)
  const [noAutoProperties, setNoAutoProperties] = useState(false)
  const [addParents, setAddParents] = useState(false)
  const subversion = useSubversion()

  const onCancel = () => {
    props.onCancel()
  }

  const onOk = async () => {
    let currentItem = ''
    await Subversion.callOnce({
      factory: subversion,
      async call(context) {
        for (let i of props.items) {
          currentItem = i.path
          const options: AddOptions = {
            path: i.path,
            depth,
            force,
            noIgnore,
            noAutoProperties,
            addParents,
          }
          await context.add(options)
        }
        props.onOk()
      },
      onError(error) {
        Toast.error({
          content: `Failed to add file(${currentItem}): ${errorHumanString(error)}`,
          stack: true,
        })
      },
    })
    // const context = await subversion.context()
    // let currentItem = ''
    // try {
    //   for (let i of props.items) {
    //     currentItem = i.path
    //     const options: AddOptions = {
    //       path: i.path,
    //       depth,
    //       force,
    //       noIgnore,
    //       noAutoProperties,
    //       addParents,
    //     }
    //     await context.add(options)
    //   }
    //   props.onOk()
    // } catch (error) {
    //   Toast.error({
    //     content: `Failed to add file(${currentItem}): ${errorHumanString(error)}`,
    //     stack: true,
    //   })
    // } finally {
    // }
  }

  return (
    <Dialog
      afterClose={props.afterClose}
      title={'Add'}
      visible={props.visible}
      onCancel={onCancel}
      onOk={onOk}
    >
      <div className={cx(flex, min_h_0, flex_col, gap_y_2, disable_move)}>
        <Card
          className={cx(min_h_0, flex)}
          headerStyle={{ padding: '0.25rem' }}
          bodyStyle={{
            padding: '0.25rem',
            minHeight: '0px',
            overflowY: 'auto',
            display: 'flex',
            flex: 1,
          }}
        >
          <ScrollArea
            className={cx(flex_1, min_w_0, min_h_0)}
            contentClassName={cx(flex, flex_col)}
          >
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
          </ScrollArea>
        </Card>
        {/*<Text strong style={{ color: 'rgba(var(--semi-grey-9), 1)' }}>Items:</Text>*/}
        <div style={{ color: 'rgba(var(--semi-grey-9), 1)' }}>Depth:</div>
        <DepthSelect
          className={cx(overflow_visible)}
          value={depth}
          onChange={(value) => setDepth(value)}
        ></DepthSelect>
        <Checkbox checked={force} onChange={(e) => setForce(e.target.checked ?? false)}>
          Force
        </Checkbox>
        <Checkbox checked={noIgnore} onChange={(e) => setNoIgnore(e.target.checked ?? false)}>
          No ignore
        </Checkbox>
        <Checkbox
          checked={noAutoProperties}
          onChange={(e) => setNoAutoProperties(e.target.checked ?? false)}
        >
          No auto properties
        </Checkbox>
        <Checkbox checked={addParents} onChange={(e) => setAddParents(e.target.checked ?? false)}>
          Add parent
        </Checkbox>
      </div>
    </Dialog>
  )
}
