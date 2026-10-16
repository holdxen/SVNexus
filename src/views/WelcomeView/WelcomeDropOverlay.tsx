import { css, cx } from '@linaria/core'
import type { Event } from '@tauri-apps/api/event'
import type { DragDropEvent } from '@tauri-apps/api/webview'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { useMemoizedFn } from 'ahooks'
import { useEffect, useState } from 'react'

import { useT } from '@/i18n'

const overlay = css`
  position: absolute;
  inset: 8px;
  z-index: 10;
  pointer-events: none;
  display: flex;
  align-items: center;
  justify-content: center;
  border: 2px dashed var(--semi-color-primary);
  border-radius: 6px;
  background-color: rgba(var(--semi-light-blue-0), 1);
  color: var(--semi-color-primary);
  font-size: 14px;
  user-select: none;
`

export interface WelcomeDropOverlayProps {
  /** 当前 tab 是欢迎页才接受拖放：拖进工作副本页不该开新 tab */
  enabled: boolean
  onDrop: (path: string) => void
}

/**
 * 监听 Tauri 的原生文件拖放。dragDropEnabled 默认开启时 webview 收不到 HTML5 的
 * drop 事件，拖进来的路径只能从这条事件拿。
 */
export function WelcomeDropOverlay({ enabled, onDrop }: WelcomeDropOverlayProps) {
  const t = useT()
  const [dragging, setDragging] = useState(false)

  // 监听只注册一次，用 useMemoizedFn 让回调始终读到最新的 enabled / onDrop
  const onEvent = useMemoizedFn((event: Event<DragDropEvent>) => {
    const payload = event.payload

    if (payload.type === 'drop') {
      setDragging(false)
      if (!enabled) {
        return
      }
      const path = payload.paths[0]
      if (path !== undefined) {
        onDrop(path)
      }
      return
    }

    if (!enabled) {
      return
    }

    // over 会连续发，enter 偶尔漏掉，两者都算「正在拖」；leave 结束
    setDragging(payload.type === 'enter' || payload.type === 'over')
  })

  useEffect(() => {
    const unlisten = getCurrentWebview().onDragDropEvent(onEvent)

    return () => {
      unlisten.then((fn) => fn())
    }
  }, [])

  if (!dragging) {
    return <></>
  }

  return <div className={cx(overlay)}>{t('welcomeUi.drop.hint')}</div>
}
