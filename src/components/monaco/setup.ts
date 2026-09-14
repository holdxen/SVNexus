import type * as Monaco from 'monaco-editor'

let setupPromise: Promise<typeof Monaco> | null = null

/**
 * 按需加载 Monaco 并完成一次性全局配置。
 *
 * 所有依赖（monaco-editor / loader / worker）都通过动态 import 引入，
 * 因此只有在真正要渲染编辑器时才会加载，不会进入主包。
 *
 * `loader.config({ monaco })` 必须在 `@monaco-editor/react` 被 import 之前执行，
 * 否则 loader 会退回到从 CDN 加载（Tauri 离线环境下不可用）。
 */
export function setupMonaco(): Promise<typeof Monaco> {
  if (setupPromise === null) {
    setupPromise = doSetup()
  }
  return setupPromise
}

async function doSetup(): Promise<typeof Monaco> {
  const [
    monaco,
    loaderModule,
    editorWorkerModule,
    cssWorkerModule,
    htmlWorkerModule,
    jsonWorkerModule,
    tsWorkerModule,
  ] = await Promise.all([
    import('monaco-editor'),
    import('@monaco-editor/loader'),
    import('monaco-editor/editor/editor.worker?worker'),
    import('monaco-editor/language/css/css.worker?worker'),
    import('monaco-editor/language/html/html.worker?worker'),
    import('monaco-editor/language/json/json.worker?worker'),
    import('monaco-editor/language/typescript/ts.worker?worker'),
  ])

  const editorWorker = editorWorkerModule.default
  const cssWorker = cssWorkerModule.default
  const htmlWorker = htmlWorkerModule.default
  const jsonWorker = jsonWorkerModule.default
  const tsWorker = tsWorkerModule.default

  self.MonacoEnvironment = {
    getWorker(_workerId, label) {
      if (label === 'json') {
        return new jsonWorker()
      }
      if (label === 'css' || label === 'scss' || label === 'less') {
        return new cssWorker()
      }
      if (label === 'html' || label === 'handlebars' || label === 'razor') {
        return new htmlWorker()
      }
      if (label === 'typescript' || label === 'javascript') {
        return new tsWorker()
      }
      return new editorWorker()
    },
  }

  loaderModule.default.config({ monaco })

  return monaco
}
