import { IconFolderStroked } from '@douyinfe/semi-icons'
import { css, cx } from '@linaria/core'
import { open } from '@tauri-apps/plugin-dialog'
import { stat } from '@tauri-apps/plugin-fs'
import { RefObject } from 'react'

import { useT } from '@/i18n'
import { cursor_pointer } from '@/styles/Classes'

import PureInput, { PureInputProps } from './PureInput'

export interface PathInputProps extends PureInputProps {
  onSelected: (path: string) => void
  ref?: RefObject<HTMLInputElement | null>
}

const inputIcon = css`
  .semi-input-append {
    cursor: default;
  }
`

const button = css`
  color: var(--semi-color-primary);
  &:hover {
    color: var(--semi-color-primary-hover);
  }
  &:active {
    color: var(--semi-color-primary-active);
  }
`

export default function PathInput(props: PathInputProps) {
  const t = useT()
  const { className, addonAfter, onSelected, ref, ...others } = props
  const selectPath = async () => {
    let defaultPath: string | undefined
    if (typeof props.value === 'string') {
      if (props.value !== '') {
        try {
          const info = await stat(props.value)
          if (info.isDirectory) {
            defaultPath = props.value
          }
        } catch {
          // path invalid, ignore
        }
      }
    }
    const selected = await open({
      title: t('shared.action.selectFolder'),
      multiple: false,
      directory: true,
      defaultPath,
    })
    if (selected === null) {
      return
    }
    onSelected(selected)
  }
  return (
    <PureInput
      ref={ref}
      className={cx(inputIcon, className)}
      addonAfter={<IconFolderStroked onClick={selectPath} className={cx(cursor_pointer, button)} />}
      {...others}
    ></PureInput>
  )
}
