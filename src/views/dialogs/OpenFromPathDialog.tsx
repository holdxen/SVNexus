import { Toast } from '@douyinfe/semi-ui'
import { cx } from '@linaria/core'
import { useRef, useState } from 'react'

import PathInput from '@/components/PathInput'
import { useTabContent } from '@/context/TabContent'
import { useTabManager } from '@/context/TabManager'
import { useT } from '@/i18n'
import { useCurrentModal } from '@/lib/multi-modal'
import { flex, flex_col, gap_y_3 } from '@/styles/Classes'

import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

export function OpenFromPathDialog() {
  const t = useT()
  const modal = useCurrentModal()
  const tabManager = useTabManager()
  const [path, setPath] = useState('')

  const tabContent = useTabContent()

  const onOk = () => {
    if (path === '') {
      Toast.error({
        content: t('shared.error.pathRequired'),
        stack: true,
      })
      return
    }
    tabManager.openWorkingCopy(tabContent.identity, path)
    modal.resolve(true)
    modal.hide()
  }

  const onCancel = () => {
    modal.resolve(false)
    modal.hide()
  }

  const focusElement = useRef<HTMLInputElement>(null)

  return (
    <Dialog
      initialFocusRef={focusElement}
      afterClose={modal.remove}
      size="small"
      title={t('dialogs.openFromPath')}
      visible={modal.visible}
      onCancel={onCancel}
      onOk={onOk}
    >
      <div className={cx(flex, flex_col, gap_y_3)}>
        <DialogFormItem title={t('shared.field.path')}>
          <PathInput
            ref={focusElement}
            autoFocus
            value={path}
            onChange={setPath}
            onSelected={setPath}
          ></PathInput>
        </DialogFormItem>
      </div>
    </Dialog>
  )
}
