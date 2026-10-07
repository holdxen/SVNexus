import { Checkbox, Toast } from '@douyinfe/semi-ui'
import { cx } from '@linaria/core'
import { useRef, useState } from 'react'

import { Depth } from '@/bindings/Depth'
import { Revision } from '@/bindings/Revision'
import { SwitchOptions } from '@/bindings/SwitchOptions'
import PureInput from '@/components/PureInput'
import { ScrollArea } from '@/components/ScrollArea'
import DepthSelect from '@/components/subversion/DepthSelect'
import RevisionSelect from '@/components/subversion/RevisionSelect'
import { Subversion, useSubversion } from '@/context/Subversion'
import { useT } from '@/i18n'
import { useCurrentModal } from '@/lib/multi-modal'
import { flex, flex_1, flex_col, gap_y_2 } from '@/styles/Classes'
import errorHumanString from '@/utils/Error'

import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

export function NiceSwitchDialog(props: { path: string; url?: string }) {
  const modal = useCurrentModal()
  return (
    <SwitchDialog
      onCancel={() => {
        modal.resolve(false)
        modal.hide()
      }}
      onOk={() => {
        modal.resolve(true)
        modal.hide()
      }}
      afterClose={modal.remove}
      visible={modal.visible}
      path={props.path}
      url={props.url ?? ''}
    ></SwitchDialog>
  )
}

export interface SwitchDialogProps {
  visible: boolean
  onOk?: () => void
  onCancel?: () => void
  afterClose?: () => void
  path: string
  url: string
}

export default function SwitchDialog(props: SwitchDialogProps) {
  const t = useT()
  const [url, setUrl] = useState(props.url)
  const [revision, setRevision] = useState<Revision>('unspecified')
  const [pegRevision, setPegRevision] = useState<Revision>('unspecified')
  const [depth, setDepth] = useState<Depth>('infinity')
  const [depthIsSticky, setDepthIsSticky] = useState(false)
  const [ignoreExternals, setIgnoreExternals] = useState(false)
  const [allowUnversionedObstructions, setAllowUnversionedObstructions] = useState(false)
  const [ignoreAncestry, setIgnoreAncestry] = useState(false)

  const subversion = useSubversion()
  const focusRef = useRef<HTMLInputElement>(null)

  const onOk = async () => {
    await Subversion.callOnce({
      factory: subversion,
      async call(context) {
        const options: SwitchOptions = {
          path: props.path,
          url,
          pegRevision,
          revision,
          depth,
          depthIsSticky,
          ignoreExternals,
          allowUnversionedObstructions,
          ignoreAncestry,
        }
        const rev = await context.switch(options)
        Toast.success({
          content: t('svnDialogs.switch.success', { revision: rev }),
          stack: true,
        })
        props.onOk?.()
      },
      onError: (error) => {
        Toast.error({
          content: t('svnDialogs.switch.failed', { error: errorHumanString(error) }),
          stack: true,
        })
      },
    })
  }

  return (
    <Dialog
      size={'medium'}
      initialFocusRef={focusRef}
      title={t('shared.action.switch')}
      onOk={onOk}
      onCancel={props.onCancel}
      visible={props.visible}
      afterClose={props.afterClose}
    >
      <ScrollArea className={cx(flex_1)}>
        <div className={cx(flex, flex_col, gap_y_2)}>
          <DialogFormItem title={t('shared.field.workingCopy')}>
            <PureInput disabled value={props.path} />
          </DialogFormItem>
          <DialogFormItem title={t('shared.field.url')}>
            <PureInput ref={focusRef} value={url} onChange={(e) => setUrl(e)} />
          </DialogFormItem>
          <DialogFormItem title={t('shared.field.revision')}>
            <RevisionSelect
              className={flex_1}
              disableLayout={true}
              kinds={['unspecified', 'head', 'number', 'committed', 'previous', 'date']}
              value={revision}
              onChange={setRevision}
            ></RevisionSelect>
          </DialogFormItem>
          <DialogFormItem title={t('shared.field.pegRevision')}>
            <RevisionSelect
              className={flex_1}
              disableLayout={true}
              kinds={['unspecified', 'number', 'date', 'base', 'working', 'head']}
              value={pegRevision}
              onChange={setPegRevision}
            ></RevisionSelect>
          </DialogFormItem>
          <DialogFormItem title={t('shared.field.depth')}>
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
            {t('shared.option.depthIsSticky')}
          </Checkbox>
          <Checkbox
            checked={ignoreExternals}
            onChange={(e) => setIgnoreExternals(e.target.checked ?? false)}
          >
            {t('shared.option.ignoreExternals')}
          </Checkbox>
          <Checkbox
            checked={allowUnversionedObstructions}
            onChange={(e) => setAllowUnversionedObstructions(e.target.checked ?? false)}
          >
            {t('shared.option.allowUnversionedObstructions')}
          </Checkbox>
          <Checkbox
            checked={ignoreAncestry}
            onChange={(e) => setIgnoreAncestry(e.target.checked ?? false)}
          >
            {t('shared.option.ignoreAncestry')}
          </Checkbox>
        </div>
      </ScrollArea>
    </Dialog>
  )
}
