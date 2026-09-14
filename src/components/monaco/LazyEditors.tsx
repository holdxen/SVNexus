import type { DiffEditorProps, EditorProps } from '@monaco-editor/react'
import { Suspense, lazy } from 'react'

import { setupMonaco } from './setup'

/**
 * Monaco 编辑器的懒加载包装。
 *
 * 工厂函数先 `await setupMonaco()`（配置 loader 使用本地 monaco），
 * 再动态 import `@monaco-editor/react`，从而保证：
 * 1. monaco-editor 及其 worker 不会进入主包；
 * 2. loader 一定在编辑器挂载前完成本地配置。
 */
const Editor = lazy(async () => {
  await setupMonaco()
  const module = await import('@monaco-editor/react')
  return { default: module.Editor }
})

const DiffEditor = lazy(async () => {
  await setupMonaco()
  const module = await import('@monaco-editor/react')
  return { default: module.DiffEditor }
})

function EditorFallback({ className }: { className?: string }) {
  return <div className={className} />
}

export function LazyEditor(props: EditorProps) {
  return (
    <Suspense fallback={<EditorFallback className={props.className} />}>
      <Editor {...props} />
    </Suspense>
  )
}

export function LazyDiffEditor(props: DiffEditorProps) {
  return (
    <Suspense fallback={<EditorFallback className={props.className} />}>
      <DiffEditor {...props} />
    </Suspense>
  )
}
