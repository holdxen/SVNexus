import { IconFolderStroked } from '@douyinfe/semi-icons'
import { css, cx } from '@linaria/core'
import { open } from '@tauri-apps/plugin-dialog'

import { cursor_pointer } from '@/styles/Classes'

import PureInput, { PureInputProps } from './PureInput'

export interface PathInputProps extends PureInputProps {
  onSelected: (path: string) => void
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
  const { className, addonAfter, onSelected, ...others } = props
  const selectPath = async () => {
    const selected = await open({
      title: 'Select folder',
      multiple: false,
      directory: true,
    })
    if (selected === null) {
      return
    }
    onSelected(selected)
  }
  return (
    <PureInput
      className={cx(inputIcon, className)}
      addonAfter={<IconFolderStroked onClick={selectPath} className={cx(cursor_pointer, button)} />}
      {...others}
    ></PureInput>
  )
}
