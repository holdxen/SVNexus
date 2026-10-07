import { Card, Checkbox, Toast } from '@douyinfe/semi-ui'
import { css, cx } from '@linaria/core'
import { useRef, useState } from 'react'

import { CopyOptions } from '@/bindings/CopyOptions'
import PureInput from '@/components/PureInput'
import PureTextArea from '@/components/PureTextArea'
import { ScrollArea } from '@/components/ScrollArea'
import FileKindIcon from '@/components/subversion/FileKindIcon'
import { Subversion, useSubversion } from '@/context/Subversion'
import { useT } from '@/i18n'
import { useCurrentModal } from '@/lib/multi-modal'
import {
  block_minimizable,
  border_box,
  flex,
  flex_1,
  flex_col,
  gap_x_2,
  gap_y_2,
  hidden,
  items_center,
  min_h_0,
  minimizable,
  p_2,
  visibility_hidden,
} from '@/styles/Classes'
import { list_item, list_item_selected } from '@/styles/Components'

import { CopySourceItem } from '../WorkspaceView/CopyMoveContext'
import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

export interface CopyDialogProps {
  afterClose?: () => void
  onCancel?: () => void
  onOK?: () => void
  visible: boolean

  /** 预填的源路径（可为空，由用户手动输入） */
  sources?: CopySourceItem[]
  defaultDestination?: string
  needCommitMessage?: boolean
}

export function NiceCopyDialog(props: {
  defaultDestination?: string
  sources?: CopySourceItem[]
  needCommitMessage?: boolean
}) {
  const modal = useCurrentModal()

  return (
    <CopyDialog
      afterClose={modal.remove}
      visible={modal.visible}
      onCancel={() => {
        modal.resolve(false)
        modal.hide()
      }}
      onOK={() => {
        modal.resolve(true)
        modal.hide()
      }}
      sources={props.sources}
      defaultDestination={props.defaultDestination}
      needCommitMessage={props.needCommitMessage}
    ></CopyDialog>
  )
}

export default function CopyDialog(props: CopyDialogProps) {
  const t = useT()
  // const [sources, setSources] = useState<string[]>(
  //   props.defaultSources && props.defaultSources.length > 0 ? props.defaultSources : [''],
  // )
  const [destination, setDestination] = useState(props.defaultDestination ?? '')
  const [commitMessage, setCommitMessage] = useState('')

  const [copyAsChild, setCopyAsChild] = useState(true)
  const [makeParents, setMakeParents] = useState(false)
  const [ignoreExternals, setIgnoreExternals] = useState(false)
  const [pinExternals, setPinExternals] = useState(false)
  const [metadataOnly, setMetadataOnly] = useState(false)

  const subversion = useSubversion()
  const focusRef = useRef<HTMLInputElement>(null)
  const [selected, setSelected] = useState<string>()

  //   const setSourceAt = (index: number, value: string) => {
  //     setSources((prev) => prev.map((e, i) => (i === index ? value : e)))
  //   }
  //
  //   const removeSourceAt = (index: number) => {
  //     setSources((prev) => {
  //       const next = prev.filter((_, i) => i !== index)
  //       return next.length > 0 ? next : ['']
  //     })
  //   }
  //
  //   const addSource = () => {
  //     setSources((prev) => [...prev, ''])
  //   }

  const onOk = async () => {
    await Subversion.callOnce({
      factory: subversion,
      async call(context) {
        // const paths = props.sources.map((e) => e.trim()).filter((e) => e !== '')
        // if (paths.length === 0) {
        //   Toast.error({
        //     content: 'Source must not be empty',
        //     stack: true,
        //   })
        //   return
        // }
        if (destination.trim() === '') {
          Toast.error({
            content: t('shared.error.destinationRequired'),
            stack: true,
          })
          return
        }
        if ((props.needCommitMessage ?? false) && commitMessage === '') {
          Toast.error({
            content: t('shared.error.commitMessageRequired'),
            stack: true,
          })
          return
        }

        const options: CopyOptions = {
          sources: props.sources ?? [],
          destination: destination.trim(),
          copyAsChild,
          makeParents,
          ignoreExternals,
          metadataOnly,
          pinExternals,
          revisionPropertyTable: null,
          commitMessage,
        }
        await context.copy(options)
        props.onOK?.()
      },
    })
  }

  return (
    <Dialog
      afterClose={props.afterClose}
      onCancel={props.onCancel}
      onOk={onOk}
      size="medium"
      title={t('shared.action.copy')}
      visible={props.visible}
      initialFocusRef={focusRef}
    >
      <div className={cx(flex, flex_1, flex_col, min_h_0, gap_y_2, border_box)}>
        <DialogFormItem className={cx(minimizable)} title={t('shared.field.sourcePathOrUrl')}>
          <Card
            className={cx(flex, flex_1)}
            bodyStyle={{ display: 'flex', flex: 1, padding: '0.5rem' }}
          >
            <ScrollArea className={cx(flex_1, flex, gap_y_2, block_minimizable)}>
              {props.sources?.map((source, index) => (
                <span
                  onClick={() => setSelected(source.path)}
                  key={index}
                  className={cx(
                    flex,
                    border_box,
                    p_2,
                    gap_x_2,
                    items_center,
                    list_item,
                    source.path === selected && list_item_selected,
                    css`
                      svg {
                        width: 18px;
                        height: 18px;
                      }
                    `,
                  )}
                >
                  <FileKindIcon
                    className={cx(source.nodeKind === undefined && visibility_hidden)}
                    kind={source.nodeKind ?? 'file'}
                  ></FileKindIcon>
                  {source.path}
                </span>
              ))}
            </ScrollArea>
          </Card>
        </DialogFormItem>
        <DialogFormItem title={t('shared.field.destinationPathOrUrl')}>
          <PureInput
            value={destination}
            placeholder={t('shared.field.pathOrUrl')}
            onChange={(value) => setDestination(value)}
          ></PureInput>
        </DialogFormItem>
        <DialogFormItem title={t('shared.field.options')}>
          <div className={cx(flex_1, flex, flex_col, gap_y_2)}>
            <Checkbox
              disabled
              checked={copyAsChild}
              onChange={(e) => setCopyAsChild(e.target.checked ?? false)}
            >
              {t('shared.option.copyAsChild')}
            </Checkbox>
            <Checkbox
              checked={makeParents}
              onChange={(e) => setMakeParents(e.target.checked ?? false)}
            >
              {t('shared.option.makeParents')}
            </Checkbox>
            <Checkbox
              checked={ignoreExternals}
              onChange={(e) => setIgnoreExternals(e.target.checked ?? false)}
            >
              {t('shared.option.ignoreExternals')}
            </Checkbox>
            <Checkbox
              checked={pinExternals}
              onChange={(e) => setPinExternals(e.target.checked ?? false)}
            >
              {t('shared.option.pinExternals')}
            </Checkbox>
            <Checkbox
              checked={metadataOnly}
              onChange={(e) => setMetadataOnly(e.target.checked ?? false)}
            >
              {t('shared.option.metadataOnly')}
            </Checkbox>
          </div>
        </DialogFormItem>
        <DialogFormItem
          className={cx(!(props.needCommitMessage ?? false) && hidden)}
          title={t('shared.field.commitMessage')}
        >
          <PureTextArea value={commitMessage} onChange={(e) => setCommitMessage(e)}></PureTextArea>
        </DialogFormItem>
      </div>
    </Dialog>
  )
}
