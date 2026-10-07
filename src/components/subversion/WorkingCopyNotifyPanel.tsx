import { Button } from '@douyinfe/semi-ui'
import { css, cx } from '@linaria/core'
import { useEffect } from 'react'

import WorkingCopyNotifyList from '@/components/subversion/WorkingCopyNotifyList'
import { useNotifyLogEntries, useNotifyLogStore } from '@/context/NotifyLog'
import type { Identity } from '@/context/TabContent'
import { useT } from '@/i18n'
import { flex, flex_1 } from '@/styles/Classes'

/** 面板需要固定高度，虚拟列表才有可测量的视口 */
const panel = css`
  display: flex;
  flex-direction: column;
  width: 420px;
  height: 420px;
  min-height: 0;
`

const header = css`
  display: flex;
  align-items: center;
  gap: 8px;
  flex: none;
  padding: 4px;
  border-bottom: 1px solid var(--semi-color-border);
`

const title = css`
  color: var(--semi-color-text-0);
  font-size: 13px;
`

const count = css`
  color: var(--semi-color-text-3);
  font-size: 12px;
`

const body = css`
  flex: 1 1 0%;
  min-height: 0;
`

export interface WorkingCopyNotifyPanelProps {
  identity: Identity
  /** 面板是否可见。可见期间的每条新通知都直接视为已读，避免小红点在面板开着时又亮起来 */
  visible: boolean
}

export default function WorkingCopyNotifyPanel(props: WorkingCopyNotifyPanelProps) {
  const t = useT()
  const entries = useNotifyLogEntries(props.identity)
  const clear = useNotifyLogStore((state) => state.clear)
  const markSeen = useNotifyLogStore((state) => state.markSeen)

  useEffect(() => {
    if (props.visible) {
      markSeen(props.identity)
    }
  }, [props.visible, props.identity, entries.length, markSeen])

  return (
    <div className={panel}>
      <div className={header}>
        <span className={title}>{t('app.notifyLog.title')}</span>
        <span className={count}>{t('app.notifyLog.entries', { count: entries.length })}</span>
        <div className={cx(flex, flex_1)} />
        <Button
          size="small"
          type="tertiary"
          disabled={entries.length === 0}
          onClick={() => clear(props.identity)}
        >
          {t('app.notifyLog.clear')}
        </Button>
      </div>
      <div className={body}>
        <WorkingCopyNotifyList entries={entries} emptyPlaceholder={t('app.notifyLog.empty')} />
      </div>
    </div>
  )
}
