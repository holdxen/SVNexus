import { Checkbox, TextArea, Toast } from '@douyinfe/semi-ui'
import { cx } from '@linaria/core'
import { useRef, useState } from 'react'

import { MoveOptions } from '@/bindings/MoveOptions'
import PureInput from '@/components/PureInput'
import { Subversion, useSubversion } from '@/context/Subversion'
import { useT } from '@/i18n'
import { useCurrentModal } from '@/lib/multi-modal'
import {
  border_box,
  break_all,
  break_word,
  flex,
  flex_1,
  flex_col,
  gap_y_2,
  hidden,
  min_h_0,
} from '@/styles/Classes'
import { localPath } from '@/utils/Path'

import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

export interface MoveRenameDialogProps {
  afterClose?: () => void
  onCancel?: () => void
  onOK?: () => void
  visible: boolean

  /** 要重命名的文件或目录路径 */
  path: string
  needCommitMessage?: boolean
}

export function NiceMoveRenameDialog(props: { path: string; needCommitMessage?: boolean }) {
  const modal = useCurrentModal()

  return (
    <MoveRenameDialog
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
      path={props.path}
      needCommitMessage={props.needCommitMessage}
    ></MoveRenameDialog>
  )
}

export default function MoveRenameDialog(props: MoveRenameDialogProps) {
  const t = useT()
  const [name, setName] = useState(localPath.getFileName(props.path) ?? '')
  const [commitMessage, setCommitMessage] = useState('')
  const [makeParents, setMakeParents] = useState(false)
  const [allowMixedRevisions, setAllowMixedRevisions] = useState(false)

  const subversion = useSubversion()
  const focusRef = useRef<HTMLInputElement>(null)

  const parent = localPath.getParent(props.path)

  const onOk = async () => {
    await Subversion.callOnce({
      factory: subversion,
      async call(context) {
        const newName = name.trim()
        if (newName === '') {
          Toast.error({
            content: t('shared.error.nameRequired'),
            stack: true,
          })
          return
        }
        if (newName.includes('/') || newName.includes('\\')) {
          Toast.error({
            content: t('shared.error.nameHasSeparator'),
            stack: true,
          })
          return
        }
        if (parent === null) {
          Toast.error({
            content: t('shared.error.cannotDetermineParent'),
            stack: true,
          })
          return
        }
        if (newName === (localPath.getFileName(props.path) ?? '')) {
          props.onOK?.()
          return
        }
        if ((props.needCommitMessage ?? false) && commitMessage === '') {
          Toast.error({
            content: t('shared.error.commitMessageRequired'),
            stack: true,
          })
          return
        }

        const destination = parent === '' ? newName : `${parent.replace(/[\\/]+$/, '')}/${newName}`

        const options: MoveOptions = {
          srcPaths: [props.path],
          destination,
          moveAsChild: true,
          makeParents,
          allowMixedRevisions,
          metadataOnly: false,
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
      title={t('shared.action.rename')}
      visible={props.visible}
      initialFocusRef={focusRef}
    >
      <div className={cx(flex, flex_1, flex_col, min_h_0, gap_y_2, border_box)}>
        <DialogFormItem title={t('shared.field.source')}>
          <span className={cx(break_all, break_word)}>{props.path}</span>
        </DialogFormItem>
        <DialogFormItem title={t('shared.field.newName')}>
          <PureInput ref={focusRef} value={name} onChange={(value) => setName(value)}></PureInput>
        </DialogFormItem>
        <DialogFormItem title={t('shared.field.options')}>
          <div className={cx(flex_1, flex, flex_col, gap_y_2)}>
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
