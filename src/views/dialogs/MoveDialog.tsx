import { Card, Checkbox, TextArea, Toast } from '@douyinfe/semi-ui'
import { css, cx } from '@linaria/core'
import { useRef, useState } from 'react'

import { MoveOptions } from '@/bindings/MoveOptions'
import PureInput from '@/components/PureInput'
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

import { MoveSourceItem } from '../WorkspaceView/CopyMoveContext'
import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

export interface MoveDialogProps {
  afterClose?: () => void
  onCancel?: () => void
  onOK?: () => void
  visible: boolean

  /** 预填的源路径（可为空，由用户手动输入） */
  sources?: MoveSourceItem[]
  needCommitMessage?: boolean
}

export function NiceMoveDialog(props: { sources?: MoveSourceItem[]; needCommitMessage?: boolean }) {
  const modal = useCurrentModal()

  return (
    <MoveDialog
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
      needCommitMessage={props.needCommitMessage}
    ></MoveDialog>
  )
}

export default function MoveDialog(props: MoveDialogProps) {
  const t = useT()
  const [destination, setDestination] = useState('')
  const [commitMessage, setCommitMessage] = useState('')

  const [moveAsChild, setMoveAsChild] = useState(true)
  const [makeParents, setMakeParents] = useState(false)
  const [allowMixedRevisions, setAllowMixedRevisions] = useState(false)
  const [metadataOnly, setMetadataOnly] = useState(false)

  const subversion = useSubversion()
  const focusRef = useRef<HTMLInputElement>(null)

  const [selected, setSelected] = useState<string>()

  const onOk = async () => {
    await Subversion.callOnce({
      factory: subversion,
      async call(context) {
        const paths = props.sources?.map((e) => e.path.trim()).filter((e) => e !== '') ?? []
        if (paths.length === 0) {
          Toast.error({
            content: t('shared.error.sourceRequired'),
            stack: true,
          })
          return
        }
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

        const options: MoveOptions = {
          srcPaths: paths,
          destination: destination.trim(),
          moveAsChild,
          makeParents,
          allowMixedRevisions,
          metadataOnly,
          revisionPropertyTable: null,
          commitMessage,
        }
        await context.move(options)
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
      title={t('shared.action.move')}
      visible={props.visible}
      initialFocusRef={focusRef}
    >
      <div className={cx(flex, flex_1, flex_col, min_h_0, gap_y_2, border_box)}>
        <DialogFormItem title={t('shared.field.sourcePathOrUrl')} className={cx(minimizable)}>
          <Card
            className={cx(flex, flex_1)}
            bodyStyle={{ display: 'flex', flex: 1, padding: '0.5rem' }}
          >
            <ScrollArea className={cx(flex_1, flex, flex_col, gap_y_2, block_minimizable)}>
              {props.sources?.map((source, index) => (
                <span
                  onClick={() => setSelected(source.path)}
                  key={index}
                  className={cx(
                    flex,
                    gap_x_2,
                    items_center,
                    list_item,
                    p_2,
                    border_box,
                    selected === source.path && list_item_selected,
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
              checked={moveAsChild}
              onChange={(e) => setMoveAsChild(e.target.checked ?? false)}
            >
              {t('shared.option.moveAsChild')}
            </Checkbox>
            <Checkbox
              checked={makeParents}
              onChange={(e) => setMakeParents(e.target.checked ?? false)}
            >
              {t('shared.option.makeParents')}
            </Checkbox>
            <Checkbox
              checked={allowMixedRevisions}
              onChange={(e) => setAllowMixedRevisions(e.target.checked ?? false)}
            >
              {t('shared.option.allowMixedRevision')}
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
          <TextArea value={commitMessage} onChange={(e) => setCommitMessage(e)}></TextArea>
        </DialogFormItem>
      </div>
    </Dialog>
  )
}
