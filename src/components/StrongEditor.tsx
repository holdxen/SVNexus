import { Card, Spin } from '@douyinfe/semi-ui'
import { cx } from '@linaria/core'
import { Editor, OnMount } from '@monaco-editor/react'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { RefObject, useImperativeHandle, useState } from 'react'

import { flex_1, flex, flex_col, border_box, min_w_0, hidden } from '@/styles/Classes'
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

export default function StrongEditor({ ref, ...props }: StrongEditorProps) {
  const [editors, setEditors] = useState(new LimitedDictionary<string, string>(100))
  // const [isLoading, setIsLoading] = useState(false);

  useImperativeHandle(ref, () => ({
    add: (key: string, content: () => Promise<string>) => {
      content().then((result) => {
        editors.add(key, result)
        setEditors(editors.clone())
      })
    },
    clear: () => {
      setEditors(new LimitedDictionary<string, string>(100))
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
    <Card
      className={cx(flex, flex_col, border_box, min_w_0, props.className)}
      bodyStyle={{ display: 'flex', flex: '1', flexDirection: 'column' }}
    >
      <Spin
        wrapperClassName={cx(flex_1, flex)}
        spinning={isLoading}
        childStyle={{ display: 'flex', flex: '1' }}
      >
        {Array.from(editors.dictionary).map(([key, content]) => {
          return (
            <div key={key} className={cx(flex_1, key !== props.currentKey && hidden)}>
              <Editor onMount={onMount} options={{ readOnly: true }} value={content}></Editor>
            </div>
          )
        })}
        {
          <div className={cx(flex_1, !placeholder && hidden)}>
            <Editor options={{ readOnly: true }}></Editor>
          </div>
        }
      </Spin>
    </Card>
  )
}
