import { Card, Spin } from '@douyinfe/semi-ui'
import { css, cx } from '@linaria/core'
import type { OnMount } from '@monaco-editor/react'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { RefObject, useImperativeHandle, useState } from 'react'

import { LazyEditor } from '@/components/monaco/LazyEditors'
import { flex_1, flex, hidden, minimizable, block_minimizable } from '@/styles/Classes'
import { LimitedDictionary } from '@/utils/LimitedDictionary'

export interface StrongEditorRef {
  add: (key: string, content: () => Promise<string>) => void
  clear: () => void
}

export interface StrongEditorProps {
  className?: string
  currentKey?: string
  ref?: RefObject<StrongEditorRef | null>
}

const editor = css``

// const minimizable = css`
//   min-width: 0px;
//   min-height: 0px;
//   & *:not(.${editor} *) {
//     min-width: 0px;
//     min-height: 0px;
//   }
// `

const box = css`
  & *:not(.${editor}, .${editor} *) {
    display: flex;
    flex: 1;
  }
`

export default function StrongEditor({ ref, ...props }: StrongEditorProps) {
  const [editors, setEditors] = useState(new LimitedDictionary<string, string>(10))
  // const [isLoading, setIsLoading] = useState(false);

  useImperativeHandle(ref, () => ({
    add: (key: string, content: () => Promise<string>) => {
      content().then((result) => {
        editors.add(key, result)
        setEditors(editors.clone())
      })
    },
    clear: () => {
      setEditors(new LimitedDictionary<string, string>(10))
    },
  }))

  const isLoading = props.currentKey !== undefined && !editors.dictionary.has(props.currentKey)
  const placeholder = editors.dictionary.size === 0 || isLoading

  const onMount: OnMount = (editor) => {
    editor.getContainerDomNode().addEventListener(
      'copy',
      (e) => {
        e.preventDefault()

        const selection = editor.getSelection()
        if (selection === null) {
          return
        }
        const text = editor.getModel()?.getValueInRange(selection)
        if (text) {
          writeText(text).catch((error) => {
            console.warn('Failed to copy text:', error)
          })
        }
      },
      true,
    )
  }

  return (
    <Card className={cx(flex, minimizable, box, props.className)}>
      <Spin spinning={isLoading}>
        {props.currentKey && editors.dictionary.has(props.currentKey) ? (
          <div className={cx(flex_1, editor, block_minimizable)}>
            <LazyEditor
              onMount={onMount}
              options={{ readOnly: true }}
              value={editors.dictionary.get(props.currentKey)}
            />
          </div>
        ) : (
          <div className={cx(flex_1, !placeholder && hidden)}></div>
        )}
      </Spin>
    </Card>
  )
}
