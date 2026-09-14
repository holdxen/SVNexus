import BinaryFileIconRaw from '@icons/BinaryFile.svg?raw'
import { css, cx } from '@linaria/core'
import type { DiffOnMount, MonacoDiffEditor } from '@monaco-editor/react'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import type * as monaco from 'monaco-editor'
import { useCallback, useEffect, useLayoutEffect, useRef } from 'react'
import * as uuid from 'uuid'

import { LazyDiffEditor } from '@/components/monaco/LazyEditors'
import { formatSize } from '@/context/Functions'
import {
  flex,
  flex_1,
  flex_col,
  gap_y_1,
  hidden,
  items_center,
  justify_center,
  relative,
  text_center,
  select_none,
} from '@/styles/Classes'

export interface BinaryFile {
  name: string
  size: number
}

export interface DifferenceModel {
  oldContent?: string | BinaryFile | undefined
  newContent?: string | BinaryFile | undefined
}

export interface DifferenceEditorProps extends DifferenceModel {
  sideBySide: boolean
  hideUnChanged: boolean
  className?: string
}

const size40 = css`
  svg {
    width: 80px;
    height: 80px;
  }
`

class BinaryFileOverlayWidget implements monaco.editor.IOverlayWidget {
  onDidLayout: monaco.IEvent<void> = () => ({ dispose() {} })
  allowEditorOverflow?: boolean = false

  id: string

  node?: HTMLElement

  file?: BinaryFile

  fileName?: HTMLDivElement

  sizeText?: HTMLDivElement

  constructor(id: string, file?: BinaryFile) {
    this.id = id
    this.file = file
  }

  setFile(file?: BinaryFile) {
    this.file = file
    if (this.node === undefined) {
      return
    }
    if (file === undefined) {
      this.node.classList.add(hidden)
      return
    }

    this.node.classList.remove(hidden)

    if (this.fileName) {
      this.fileName.innerText = file.name
    }
    formatSize(file.size).then((v) => {
      if (this.sizeText) {
        this.sizeText.innerText = v
      }
    })
  }

  getId(): string {
    return this.id
  }
  getDomNode(): HTMLElement {
    if (this.node) {
      return this.node
    }

    const root = document.createElement('div')
    root.style.backgroundColor = 'var(--semi-color-bg-0)'
    root.style.position = 'absolute'
    // root.style.inset = '0px'
    // root.style.transition = 'none'  // 禁用过渡动画
    // root.style.transform = 'none'   // 禁用变换
    root.style.setProperty('top', '0', 'important')
    root.style.setProperty('left', '0', 'important')
    root.style.setProperty('right', '0', 'important')
    root.style.setProperty('bottom', '0', 'important')
    this.node = root

    const center = document.createElement('div')
    center.classList.add(flex)
    center.classList.add(flex_col)
    center.classList.add(gap_y_1)
    center.classList.add(items_center)
    {
      const svg = document.createElement('div')
      svg.innerHTML = BinaryFileIconRaw
      svg.classList.add(size40)
      center.appendChild(svg)
    }

    {
      const fileName = document.createElement('div')
      fileName.innerText = this.file?.name ?? ''
      fileName.classList.add(text_center)
      center.appendChild(fileName)
      this.fileName = fileName
    }

    {
      const size = document.createElement('div')
      size.innerText = String(this.file?.size)
      size.classList.add(text_center)

      if (this.file) {
        formatSize(this.file.size).then((v) => {
          size.innerText = v
        })
      }

      this.sizeText = size

      center.appendChild(size)
    }

    root.classList.add(flex)
    root.classList.add(items_center)
    root.classList.add(justify_center)
    root.classList.add(select_none)
    if (this.file === undefined) {
      root.classList.add(hidden)
    }

    root.appendChild(center)

    return root
  }
  getPosition(): monaco.editor.IOverlayWidgetPosition | null {
    return null
  }
  getMinContentWidthInPx?(): number {
    return 0
  }
}

const originalOverlayWidget = css`
  .original-in-monaco-diff-editor .overlayWidgets {
    inset: 0px;
    /* monaco 在 rAF 渲染帧里写内联 width（隐藏期间会塌缩成 ~5px），
       必须用 !important 覆盖，否则显示后的第一帧容器仍是旧宽度，导致闪烁 */
    width: auto !important;
  }
`

const modifiedOverlayWidget = css`
  .modified-in-monaco-diff-editor .overlayWidgets {
    inset: 0px;
    width: auto !important;
  }
`

export default function DifferenceEditor(props: DifferenceEditorProps) {
  let oldContent: string | undefined
  if (typeof props.oldContent === 'string') {
    oldContent = props.oldContent
  }
  let newContent: string | undefined
  if (typeof props.newContent === 'string') {
    newContent = props.newContent
  }

  // 接管 DiffEditor 的 dispose 顺序：先销毁 widget，再销毁 model
  // 避免 @monaco-editor/react 默认先销毁 model 触发 Monaco 0.52+ 的断言错误
  // "TextModel got disposed before DiffEditorWidget model got reset"
  const editorRef = useRef<MonacoDiffEditor | null>(null)
  const modelsRef = useRef<{ dispose(): void }[]>([])

  const isNewBinary = typeof props.newContent === 'object'
  const isOldBinary = typeof props.oldContent === 'object'

  const isSideBySide = props.sideBySide || isNewBinary || isOldBinary

  const onCopy = useCallback((e: ClipboardEvent) => {
    if (editorRef.current === null) {
      return
    }

    const modifiedEditor = editorRef.current.getModifiedEditor()
    const originalEditor = editorRef.current.getOriginalEditor()

    let editor
    if (modifiedEditor.hasTextFocus()) {
      editor = modifiedEditor
    } else if (originalEditor.hasTextFocus()) {
      editor = originalEditor
    } else {
      return
    }

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
    e.preventDefault()
  }, [])

  const onMount: DiffOnMount = (editor) => {
    editorRef.current = editor
    const model = editor.getModel()
    modelsRef.current = model ? [model.original, model.modified] : []
    editorRef.current.getContainerDomNode().addEventListener('copy', onCopy, true)

    if (oldOverlayWidget.current === null) {
      const overlay = new BinaryFileOverlayWidget(uuid.v4())
      oldOverlayWidget.current = overlay
      editorRef.current.getOriginalEditor().addOverlayWidget(overlay)
    }
    if (newOverlayWidget.current === null) {
      const overlay = new BinaryFileOverlayWidget(uuid.v4())
      newOverlayWidget.current = overlay
      editorRef.current.getModifiedEditor().addOverlayWidget(overlay)
    }
    if (typeof props.oldContent === 'object') {
      oldOverlayWidget.current?.setFile(props.oldContent)
    }
    if (typeof props.newContent === 'object') {
      newOverlayWidget.current?.setFile(props.newContent)
    }
  }

  const newOverlayWidget = useRef<BinaryFileOverlayWidget>(null)
  const oldOverlayWidget = useRef<BinaryFileOverlayWidget>(null)

  // useLayoutEffect：在绘制前同步切换显隐，避免晚一帧造成闪烁
  useLayoutEffect(() => {
    if (editorRef.current === null) {
      return
    }

    oldOverlayWidget.current?.setFile(
      typeof props.oldContent === 'object' ? props.oldContent : undefined,
    )
    newOverlayWidget.current?.setFile(
      typeof props.newContent === 'object' ? props.newContent : undefined,
    )
  }, [props.newContent, props.oldContent])

  useEffect(() => {
    return () => {
      editorRef.current?.getContainerDomNode().removeEventListener('copy', onCopy, true)
      editorRef.current?.dispose()
      editorRef.current = null
      for (const model of modelsRef.current) model.dispose()
      modelsRef.current = []
    }
  }, [])

  return (
    <div className={cx(flex, props.className)}>
      <div className={cx(flex_1)}>
        <LazyDiffEditor
          className={cx(
            relative,
            isNewBinary && modifiedOverlayWidget,
            isOldBinary && originalOverlayWidget,
          )}
          keepCurrentOriginalModel
          keepCurrentModifiedModel

          onMount={onMount}
          options={{
            ignoreTrimWhitespace: true,
            originalEditable: false,
            hideUnchangedRegions: {
              enabled: props.hideUnChanged,
              contextLineCount: 3, // 每处更改前后保留的上下文行数
              minimumLineCount: 3, // 未更改区域至少多少行才折叠
              revealLineCount: 10, // 点击展开时显示多少行
            },
            readOnly: true,
            useInlineViewWhenSpaceIsLimited: false,
            renderSideBySide: isSideBySide,
            experimental: {
              showEmptyDecorations: false,
            },
          }}
          original={oldContent}
          modified={newContent}
        ></LazyDiffEditor>
      </div>
    </div>
  )
}
