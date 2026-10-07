import { Button, Checkbox, Input, Select, TextArea, Toast } from '@douyinfe/semi-ui'
import { css, cx } from '@linaria/core'
import { getTauriVersion, getVersion } from '@tauri-apps/api/app'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { open } from '@tauri-apps/plugin-dialog'
import Bowser from 'bowser'
import dayjs from 'dayjs'
import { useEffect, useState } from 'react'

import { ExtendedVersion } from '@/bindings/ExtendedVersion'
import { ScrollArea } from '@/components/ScrollArea'
import { extendedVersion } from '@/context/Functions'
import { t, useLocale, useT, type MessageKey } from '@/i18n'
import { useCurrentModal } from '@/lib/multi-modal'
import {
  border_box,
  break_all,
  break_word,
  flex,
  flex_1,
  flex_col,
  flex_wrap,
  gap_x_1,
  gap_x_2,
  gap_y_1,
  gap_y_2,
  items_center,
  min_h_0,
} from '@/styles/Classes'
import { ATTACHMENTS_ENABLED, submitFeedback } from '@/utils/Feedback'

import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

export type FeedbackCategory = 'crash' | 'bug' | 'ui' | 'performance' | 'suggestion' | 'other'

type Translate = ReturnType<typeof useT>

const categories: { value: FeedbackCategory; label: MessageKey }[] = [
  { value: 'crash', label: 'feedback.category.crash' },
  { value: 'bug', label: 'feedback.category.bug' },
  { value: 'ui', label: 'feedback.category.ui' },
  { value: 'performance', label: 'feedback.category.performance' },
  { value: 'suggestion', label: 'feedback.category.suggestion' },
  { value: 'other', label: 'feedback.category.other' },
]

export interface FeedbackDiagnostic {
  key: string
  value: string
}

export interface FeedbackReport {
  category?: FeedbackCategory
  content: string
  steps: string
  contact: string
  attachments: string[]
  diagnostics: FeedbackDiagnostic[]
  /// 勾选后由后端把日志目录打包成压缩包，作为附件一起上传
  includeLogs: boolean
}

export function formatDiagnostics(diagnostics: FeedbackDiagnostic[]): string {
  return diagnostics
    .map((entry) => t('feedback.diagnostics.line', { key: entry.key, value: entry.value }))
    .join('\n')
}

function formatSubversion(version: ExtendedVersion, t: Translate): string {
  const { major, minor, patch, tag } = version.version
  const runtime = version.runtimeHost || version.runtimeOsName
  const value = `${major}.${minor}.${patch}${tag ?? ''}`
  return runtime ? t('feedback.diagnostics.subversionValue', { version: value, runtime }) : value
}

function formatWorkingCopies(workingCopies: string[], t: Translate): string {
  if (workingCopies.length === 0) {
    return t('feedback.diagnostics.none')
  }
  const [current, ...rest] = workingCopies
  if (rest.length === 0) {
    return current
  }
  return t('feedback.diagnostics.workingCopiesValue', {
    current,
    count: rest.length,
    rest: rest.join(', '),
  })
}

// 采集随反馈一起提交的环境信息，便于定位问题。任何一项取不到都降级为「未知」而不是让弹窗失败。
async function collectDiagnostics(
  workingCopies: string[],
  t: Translate,
): Promise<FeedbackDiagnostic[]> {
  const [appVersion, tauriVersion, subversion] = await Promise.all([
    getVersion().catch(() => t('feedback.unknown')),
    getTauriVersion().catch(() => t('feedback.unknown')),
    extendedVersion(false).catch(() => null),
  ])
  const browser = Bowser.getParser(navigator.userAgent)
  const os = browser.getOS()
  const engine = browser.getEngine()

  return [
    {
      key: t('feedback.diagnostics.appVersion'),
      value: t('feedback.diagnostics.versionValue', { appVersion, tauriVersion }),
    },
    {
      key: 'Subversion',
      value: subversion ? formatSubversion(subversion, t) : t('feedback.unknown'),
    },
    {
      key: t('feedback.diagnostics.system'),
      value: t('feedback.diagnostics.osValue', {
        os: `${os.name ?? t('feedback.unknown')}${os.version ? ` ${os.version}` : ''}`,
        engine: `${engine.name} ${engine.version}`,
      }),
    },
    // 上报应用当前使用的界面语言，而不是操作系统的语言
    { key: t('feedback.diagnostics.language'), value: useLocale.getState().locale },
    {
      key: t('feedback.diagnostics.timeZone'),
      value: t('feedback.diagnostics.timeZoneValue', {
        zone: Intl.DateTimeFormat().resolvedOptions().timeZone,
        offset: dayjs().format('Z'),
      }),
    },
    {
      key: t('feedback.diagnostics.workingCopies'),
      value: formatWorkingCopies(workingCopies, t),
    },
    {
      key: t('feedback.diagnostics.collectedAt'),
      value: dayjs().format('YYYY-MM-DD HH:mm:ss'),
    },
  ]
}

export function NiceFeedbackDialog(props: { workingCopies?: string[] }) {
  const modal = useCurrentModal()
  const t = useT()

  const submit = async (report: FeedbackReport) => {
    try {
      await submitFeedback(report, report.attachments)
      Toast.success({ content: t('feedback.submit.ok') })
      modal.resolve(report)
      modal.hide()
    } catch (error) {
      // 提交失败时不关弹窗，否则用户刚写的内容就丢了
      Toast.error({
        content: t('feedback.submit.failed', {
          reason: error instanceof Error ? error.message : String(error),
        }),
        stack: true,
      })
    }
  }

  return (
    <FeedbackDialog
      visible={modal.visible}
      afterClose={modal.remove}
      workingCopies={props.workingCopies}
      onOk={submit}
      onCancel={() => {
        modal.resolve(undefined)
        modal.hide()
      }}
    ></FeedbackDialog>
  )
}

export interface FeedbackDialogProps {
  visible: boolean
  afterClose?: () => void
  onOk?: (report: FeedbackReport) => void | Promise<void>
  onCancel?: () => void
  workingCopies?: string[]
}

const diagnostics_box = css`
  border: 1px solid var(--semi-color-border);
  border-radius: 6px;
  background-color: var(--semi-color-fill-0);
  padding: 8px 10px;
`

const diagnostics_text = css`
  margin: 0;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 12px;
  line-height: 1.7;
  white-space: pre-wrap;
  word-break: break-all;
  color: var(--semi-color-text-1);
`

export default function FeedbackDialog(props: FeedbackDialogProps) {
  const t = useT()
  const [category, setCategory] = useState<FeedbackCategory>()
  const [content, setContent] = useState('')
  const [steps, setSteps] = useState('')
  const [contact, setContact] = useState('')
  const [attachments, setAttachments] = useState<string[]>([])
  const [includeLogs, setIncludeLogs] = useState(false)
  const [attachDiagnostics, setAttachDiagnostics] = useState(true)
  const [diagnostics, setDiagnostics] = useState<FeedbackDiagnostic[] | null>(null)

  useEffect(() => {
    if (!props.visible) {
      return
    }
    let cancelled = false
    collectDiagnostics(props.workingCopies ?? [], t).then((entries) => {
      if (!cancelled) {
        setDiagnostics(entries)
      }
    })
    return () => {
      cancelled = true
    }
  }, [props.visible, props.workingCopies])

  const pickAttachments = async () => {
    const picked = await open({
      multiple: true,
      directory: false,
      title: t('feedback.attachments.pick'),
    })
    if (picked === null) {
      return
    }
    setAttachments((previous) => [...new Set([...previous, ...picked])])
  }

  const copyDiagnostics = () => {
    if (!diagnostics) {
      return
    }
    writeText(formatDiagnostics(diagnostics)).catch(() => {
      Toast.error({ content: t('feedback.diagnostics.copyFailed'), stack: true })
    })
  }

  const onOk = () => {
    if (content.trim() === '') {
      Toast.error({ content: t('feedback.content.required'), stack: true })
      return
    }
    // 返回 promise 交给 Dialog，它会自动把确定按钮置为 loading 并禁用取消
    return props.onOk?.({
      category,
      content,
      steps,
      contact,
      attachments,
      diagnostics: attachDiagnostics ? (diagnostics ?? []) : [],
      includeLogs,
    })
  }

  return (
    <Dialog
      size="medium"
      title={t('feedback.title')}
      visible={props.visible}
      afterClose={props.afterClose}
      onOk={onOk}
      onCancel={props.onCancel}
    >
      <ScrollArea className={cx(flex_1, min_h_0)}>
        <div className={cx(flex, flex_col, gap_y_2, border_box)}>
          <DialogFormItem title={t('feedback.category.title')}>
            <Select
              clickToHide
              className={cx(flex_1)}
              placeholder={t('feedback.category.placeholder')}
              value={category}
              onChange={(value) => setCategory(value as FeedbackCategory)}
            >
              {categories.map((item) => (
                <Select.Option key={item.value} value={item.value}>
                  {t(item.label)}
                </Select.Option>
              ))}
            </Select>
          </DialogFormItem>
          <DialogFormItem title={t('feedback.content.title')}>
            <TextArea
              className={cx(flex_1)}
              autosize={{ minRows: 4, maxRows: 8 }}
              placeholder={t('feedback.content.placeholder')}
              value={content}
              onChange={(value) => setContent(value)}
            ></TextArea>
          </DialogFormItem>
          <DialogFormItem title={t('feedback.steps.title')}>
            <TextArea
              className={cx(flex_1)}
              autosize={{ minRows: 3, maxRows: 8 }}
              placeholder={t('feedback.steps.placeholder')}
              value={steps}
              onChange={(value) => setSteps(value)}
            ></TextArea>
          </DialogFormItem>
          <DialogFormItem title={t('feedback.contact.title')}>
            <Input
              className={cx(flex_1)}
              placeholder={t('feedback.contact.placeholder')}
              value={contact}
              onChange={(value) => setContact(value)}
            ></Input>
          </DialogFormItem>
          {ATTACHMENTS_ENABLED ? (
            <DialogFormItem title={t('feedback.attachments.title')}>
              <div className={cx(flex_1, flex_col, gap_y_1)}>
                <div className={cx(flex, gap_x_2, items_center, flex_wrap)}>
                  <Button size="small" onClick={pickAttachments}>
                    {t('feedback.attachments.select')}
                  </Button>
                  {attachments.length > 0 ? (
                    <span
                      className={cx(break_all, break_word)}
                      style={{ color: 'var(--semi-color-text-2)' }}
                    >
                      {t('feedback.attachments.selected', { count: attachments.length })}
                    </span>
                  ) : null}
                </div>
                {attachments.map((file) => (
                  <div
                    key={file}
                    className={cx(flex, gap_x_1, items_center, break_all, break_word)}
                  >
                    <span className={cx(flex_1)} style={{ color: 'var(--semi-color-text-1)' }}>
                      {file}
                    </span>
                    <Button
                      size="small"
                      type="tertiary"
                      theme="borderless"
                      onClick={() =>
                        setAttachments((previous) => previous.filter((e) => e !== file))
                      }
                    >
                      {t('feedback.attachments.remove')}
                    </Button>
                  </div>
                ))}
              </div>
            </DialogFormItem>
          ) : null}
          {ATTACHMENTS_ENABLED ? (
            <div className={cx(flex, items_center, gap_x_2)}>
              <Checkbox
                checked={includeLogs}
                onChange={(e) => setIncludeLogs(e.target.checked ?? false)}
              >
                {t('feedback.logs.attach')}
              </Checkbox>
            </div>
          ) : null}
          <div className={cx(flex, flex_col, gap_y_1)}>
            <div className={cx(flex, gap_x_2, items_center)}>
              <Checkbox
                checked={attachDiagnostics}
                onChange={(e) => setAttachDiagnostics(e.target.checked ?? false)}
              >
                {t('feedback.diagnostics.attach')}
              </Checkbox>
              <div className={cx(flex_1)}></div>
              <Button size="small" type="tertiary" theme="borderless" onClick={copyDiagnostics}>
                {t('feedback.diagnostics.copy')}
              </Button>
            </div>
            <div className={cx(diagnostics_box)}>
              <pre className={cx(diagnostics_text)}>
                {diagnostics ? formatDiagnostics(diagnostics) : t('feedback.diagnostics.loading')}
              </pre>
            </div>
          </div>
        </div>
      </ScrollArea>
    </Dialog>
  )
}
