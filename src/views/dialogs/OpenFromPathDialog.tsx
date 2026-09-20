import { IconFolderStroked } from '@douyinfe/semi-icons'
import { Toast } from '@douyinfe/semi-ui'
import { css, cx } from '@linaria/core'
import { open } from '@tauri-apps/plugin-dialog'
import { stat } from '@tauri-apps/plugin-fs'
import { useRef, useState } from 'react'

import PureInput from '@/components/PureInput'
import { useTabContent } from '@/context/TabContent'
import { useTabManager } from '@/context/TabManager'
import { useCurrentModal } from '@/lib/multi-modal'
import { cursor_pointer, flex, flex_col, gap_y_3 } from '@/styles/Classes'

import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

const button = css`
  color: var(--semi-color-primary);
  &:hover {
    color: var(--semi-color-primary-hover);
  }
  &:active {
    color: var(--semi-color-primary-active);
  }
`

const inputIcon = css`
  .semi-input-append {
    cursor: default;
  }
`

export function OpenFromPathDialog() {
  const modal = useCurrentModal()
  const tabManager = useTabManager()
  const [path, setPath] = useState('')

  const tabContent = useTabContent()

  const selectPath = async () => {
    let defaultPath: string | undefined
    if (path !== '') {
      try {
        const info = await stat(path)
        if (info.isDirectory) {
          defaultPath = path
        }
      } catch {
        // path invalid, ignore
      }
    }
    let selected = await open({
      title: 'Select working copy',
      multiple: false,
      directory: true,
      defaultPath,
    })
    if (selected === null) {
      return
    }
    selected = selected.replace(/\\/g, '/')
    setPath(selected)
  }

  const onOk = () => {
    if (path === '') {
      Toast.error({
        content: 'Path must not be empty',
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
      title={'从路径打开'}
      visible={modal.visible}
      onCancel={onCancel}
      onOk={onOk}
    >
      <div className={cx(flex, flex_col, gap_y_3)}>
        <DialogFormItem title={'Path:'}>
          <PureInput
            ref={focusElement}
            className={inputIcon}
            autoFocus
            value={path}
            onChange={(value) => setPath(value)}
            addonAfter={
              <IconFolderStroked onClick={selectPath} className={cx(cursor_pointer, button)} />
            }
          ></PureInput>
        </DialogFormItem>
      </div>
    </Dialog>
  )
}
