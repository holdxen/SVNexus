import { Button, Progress, Typography } from '@douyinfe/semi-ui'
import { css, cx } from '@linaria/core'
import { relaunch } from '@tauri-apps/plugin-process'
import type { Update } from '@tauri-apps/plugin-updater'
import { openUrl } from '@tauri-apps/plugin-opener'
import { useState } from 'react'

import { useCurrentModal } from '@/lib/multi-modal'
import { flex, flex_col, flex_1, gap_y_1, gap_y_2 } from '@/styles/Classes'
import Logger from '@/utils/Logger'

import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

const GITHUB_RELEASES_URL = 'https://github.com/holdxen/SVNexus/releases'

const notesStyle = css`
  overflow: auto;
  max-height: 280px;
  padding: 8px 12px;
  border-radius: 6px;
  background-color: var(--semi-color-fill-0);
  color: var(--semi-color-text-1);
  font-size: 13px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-word;
`

interface UpdateDialogProps {
  visible: boolean
  update: Update
  afterClose?: () => void
  onCancel?: () => void
}

export function AppUpdateDialog(props: UpdateDialogProps) {
  const [installing, setInstalling] = useState(false)
  const [percent, setPercent] = useState(0)
  const [indeterminate, setIndeterminate] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const startUpdate = async () => {
    if (installing) {
      return
    }
    setInstalling(true)
    setError(null)
    setPercent(0)
    setIndeterminate(false)
    let received = 0
    let total: number | undefined
    try {
      await props.update.downloadAndInstall((event) => {
        if (event.event === 'Started') {
          total = event.data.contentLength
          setIndeterminate(!total)
        } else if (event.event === 'Progress') {
          received += event.data.chunkLength
          if (total) {
            setIndeterminate(false)
            setPercent(Math.min(100, Math.round((received / total) * 100)))
          }
        } else if (event.event === 'Finished') {
          setIndeterminate(false)
          setPercent(100)
        }
      })
      await relaunch()
    } catch (e) {
      Logger.warn('Update failed: ', e)
      setError(e instanceof Error ? e.message : String(e))
      setInstalling(false)
    }
  }

  const handleCancel = () => {
    if (installing) {
      return
    }
    props.onCancel?.()
  }

  return (
    <Dialog
      size="medium"
      title={`发现新版本 v${props.update.version}`}
      visible={props.visible}
      afterClose={props.afterClose}
      onCancel={handleCancel}
      closable={!installing}
      footer={
        <div className={cx(flex, gap_y_1)} style={{ margin: '10px 0 5px' }}>
          <div className={cx(flex_1)}></div>
          <Button
            type="tertiary"
            disabled={installing}
            onClick={handleCancel}
            style={{ marginRight: 8 }}
          >
            取消
          </Button>
          <Button type="primary" theme="solid" loading={installing} onClick={startUpdate}>
            立即更新
          </Button>
        </div>
      }
    >
      <div className={cx(flex, flex_col, gap_y_2)}>
        <Typography.Text>
          {`当前版本 v${props.update.currentVersion}，发现新版本 v${props.update.version}，更新将在下载完成后自动安装并重启应用。`}
        </Typography.Text>
        {props.update.body ? <div className={notesStyle}>{props.update.body}</div> : null}
        <a
          href={GITHUB_RELEASES_URL}
          onClick={(e) => {
            e.preventDefault()
            openUrl(GITHUB_RELEASES_URL)
          }}
          style={{ fontSize: 13, color: 'var(--semi-color-primary)' }}
        >
          在 GitHub 查看更新说明
        </a>
        {installing ? (
          <DialogFormItem title="下载进度:">
            <div className={cx(flex, flex_col, gap_y_1)}>
              <div>{indeterminate ? '下载中…' : `${percent}%`}</div>
              <Progress
                indeterminate={indeterminate}
                percent={indeterminate ? 0 : percent}
                size="large"
              />
            </div>
          </DialogFormItem>
        ) : null}
        {error ? <Typography.Text type="danger">更新失败: {error}</Typography.Text> : null}
      </div>
    </Dialog>
  )
}

export function NiceAppUpdateDialog(props: { update: Update }) {
  const modal = useCurrentModal()

  return (
    <AppUpdateDialog
      visible={modal.visible}
      update={props.update}
      afterClose={modal.remove}
      onCancel={() => {
        modal.hide()
      }}
    />
  )
}
