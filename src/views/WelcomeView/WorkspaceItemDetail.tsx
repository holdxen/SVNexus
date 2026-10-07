import { IconCopy } from '@douyinfe/semi-icons'
import { Button, Card, Divider, Tag, Typography } from '@douyinfe/semi-ui'
import { css, cx } from '@linaria/core'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import dayjs from 'dayjs'
import { Fragment, useEffect, useRef, useState } from 'react'

import HoverTooltip from '@/components/HoverTooltip'
import { ScrollArea } from '@/components/ScrollArea'
import { useDatabase } from '@/context/Database'
import { useT } from '@/i18n'
import { IconButton } from '@/icons/IconButton'
import { cursor_default, flex, flex_1, flex_col, h_full, min_h_0, p_2 } from '@/styles/Classes'
import { workspaceItemIdentity } from '@/utils/WorkspaceItem'

const { Title, Text } = Typography

const panel = css`
  display: flex;
  flex-direction: column;
  gap: 18px;
  box-sizing: border-box;
  padding: 12px 14px 16px;
`

const header = css`
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-width: 0;
`

const header_name = css`
  margin: 0;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
`

const tags = css`
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
`

const section = css`
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
`

const section_title = css`
  color: var(--semi-color-text-2);
  font-size: 11px;
  line-height: 16px;
  letter-spacing: 0.08em;
`

const fields = css`
  display: grid;
  align-items: start;
  grid-template-columns: max-content minmax(0, 1fr);
  column-gap: 14px;
  row-gap: 8px;
`

const field_label = css`
  color: var(--semi-color-text-2);
  font-size: 12px;
  line-height: 18px;
  white-space: nowrap;
`

const field_value = css`
  display: flex;
  align-items: start;
  gap: 4px;
  min-width: 0;
  color: var(--semi-color-text-0);
  font-size: 12px;
  line-height: 18px;
`

const value_text = css`
  min-width: 0;
  overflow-wrap: anywhere;
  user-select: text;
`

const value_empty = css`
  color: var(--semi-color-text-3);
`

const copy_button = css`
  flex-shrink: 0;
  margin-top: 1px;
  opacity: 0.25;
  transition: opacity 120ms ease;

  &:hover {
    opacity: 1;
  }
`

const footer = css`
  display: flex;
  padding: 10px 14px;
  border-top: 1px solid var(--semi-color-border);
`

function Field(props: { label: string; value: React.ReactNode }) {
  return (
    <>
      <div className={field_label}>{props.label}</div>
      <div className={field_value}>{props.value}</div>
    </>
  )
}

function Section(props: { title: string; children: React.ReactNode }) {
  return (
    <div className={section}>
      <div className={section_title}>{props.title}</div>
      <div className={fields}>{props.children}</div>
    </div>
  )
}

function Plain(props: { value: string }) {
  return <span className={value_text}>{props.value}</span>
}

function Empty(props: { children: string }) {
  return <span className={value_empty}>{props.children}</span>
}

// 在分隔符后插入软换行点，长路径换行时对齐目录边界，而不是从单词中间断开
function Path(props: { value: string }) {
  const parts = props.value.split(/([/\\])/)
  return (
    <span className={value_text} title={props.value}>
      {parts.map((part, index) => (
        <Fragment key={index}>
          {part}
          {index % 2 === 1 && <wbr />}
        </Fragment>
      ))}
    </span>
  )
}

function CopyButton(props: { value: string }) {
  const t = useT()
  const [copied, setCopied] = useState(false)
  const timer = useRef<number | undefined>(undefined)

  useEffect(() => {
    return () => window.clearTimeout(timer.current)
  }, [])

  return (
    <HoverTooltip content={copied ? t('welcome.detail.copied') : t('welcome.detail.copy')}>
      <IconButton
        size={18}
        className={copy_button}
        onClick={() => {
          writeText(props.value)
          setCopied(true)
          window.clearTimeout(timer.current)
          timer.current = window.setTimeout(() => setCopied(false), 1200)
        }}
      >
        <IconCopy size="small" />
      </IconButton>
    </HoverTooltip>
  )
}

function CopyableValue(props: { value: string }) {
  return (
    <>
      <Path value={props.value} />
      <CopyButton value={props.value} />
    </>
  )
}

function TimeValue(props: { time: number | null }) {
  if (props.time === null) {
    return <Empty>—</Empty>
  }
  return <Plain value={dayjs(props.time).format('YYYY-MM-DD HH:mm:ss')} />
}

export interface WorkspaceItemDetailProps {
  identity: string | null
  onOpen?: () => void
}

export function WorkspaceItemDetail(props: WorkspaceItemDetailProps) {
  const t = useT()
  const workspaceItems = useDatabase((state) => state.workspaceItems)
  const workspaceItemStates = useDatabase((state) => state.workspaceItemStates)

  const item = workspaceItems.find((e) => workspaceItemIdentity(e) === props.identity)

  if (!item) {
    return (
      <Card
        shadows="always"
        className={cx(h_full, min_h_0, flex, flex_col, cursor_default)}
        bodyStyle={{ minHeight: 0, flex: 1, display: 'flex', flexDirection: 'column' }}
      >
        <div
          className={cx(flex, flex_col, p_2, flex_1)}
          style={{ justifyContent: 'center', alignItems: 'center' }}
        >
          <Text type="tertiary">{t('welcome.detail.noSelection')}</Text>
        </div>
      </Card>
    )
  }

  const identity = workspaceItemIdentity(item)
  const state = workspaceItemStates.get(identity)

  const statusTags: React.ReactNode[] = []

  if (state) {
    if (state.revision) {
      const revision = state.revision
      const text =
        revision.min === revision.max ? `r${revision.min}` : `r${revision.min}:r${revision.max}`
      statusTags.push(
        <Tag key="revision" type="solid" shape="circle" color="purple">
          {text}
        </Tag>,
      )
    }
    if (state.isInvalid === true) {
      statusTags.push(
        <Tag key="invalid" type="solid" shape="circle" color="purple">
          {t('shared.status.invalid')}
        </Tag>,
      )
    }
    if (state.isConflicted === true) {
      statusTags.push(
        <Tag key="conflicted" type="solid" shape="circle" color="violet">
          {t('shared.status.conflicted')}
        </Tag>,
      )
    }
    if (state.isModified === true) {
      statusTags.push(
        <Tag key="modified" type="solid" shape="circle" color="amber">
          {t('shared.status.modified')}
        </Tag>,
      )
    }
    if (state.isLocked === true) {
      statusTags.push(
        <Tag key="locked" type="solid" shape="circle" color="red">
          {t('shared.status.locked')}
        </Tag>,
      )
    }
    if (state.isClean === true) {
      statusTags.push(
        <Tag key="clean" type="solid" shape="circle" color="green">
          {t('shared.status.clean')}
        </Tag>,
      )
    }
  }

  const workingCopy = 'workingCopy' in item ? item.workingCopy : null
  const repository = 'repository' in item ? item.repository : null

  return (
    <Card
      shadows="always"
      className={cx(h_full, min_h_0, flex, flex_col, cursor_default)}
      bodyStyle={{ minHeight: 0, flex: 1, display: 'flex', flexDirection: 'column' }}
    >
      <ScrollArea className={cx(flex_1, min_h_0)}>
        <div className={panel}>
          <div className={header}>
            <Title
              heading={4}
              className={header_name}
              title={workingCopy?.name ?? repository?.name}
            >
              {workingCopy?.name ?? repository?.name}
            </Title>
            <div className={tags}>
              <Tag color={workingCopy ? 'blue' : 'green'} shape="circle">
                {workingCopy ? t('welcome.detail.workingCopy') : t('welcome.detail.repository')}
              </Tag>
              {statusTags}
            </div>
          </div>

          <Divider margin={0} />

          {workingCopy && (
            <>
              <Section title={t('welcome.detail.path')}>
                <Field
                  label={t('welcome.detail.rootDirectory')}
                  value={<CopyableValue value={workingCopy.workingCopyRoot} />}
                />
                {workingCopy.workingCopyPath !== workingCopy.workingCopyRoot && (
                  <Field
                    label={t('welcome.detail.openPath')}
                    value={<CopyableValue value={workingCopy.workingCopyPath} />}
                  />
                )}
                <Field
                  label={t('welcome.detail.repositoryUrl')}
                  value={
                    workingCopy.repositoryRootUrl ? (
                      <CopyableValue value={workingCopy.repositoryRootUrl} />
                    ) : (
                      <Empty>{t('welcome.detail.notAssociated')}</Empty>
                    )
                  }
                />
              </Section>

              <Section title={t('welcome.detail.info')}>
                <Field
                  label={t('welcome.detail.lastUsed')}
                  value={<TimeValue time={workingCopy.lastUsedTime} />}
                />
                <Field
                  label={t('shared.action.checkout')}
                  value={
                    workingCopy.checkout === null ? (
                      <Empty>—</Empty>
                    ) : (
                      <Plain value={`r${workingCopy.checkout}`} />
                    )
                  }
                />
                <Field
                  label={t('welcome.detail.favourite')}
                  value={
                    workingCopy.star ? (
                      <Plain value={t('welcome.detail.yes')} />
                    ) : (
                      <Empty>{t('welcome.detail.no')}</Empty>
                    )
                  }
                />
                <Field
                  label={t('welcome.detail.remark')}
                  value={
                    workingCopy.remark ? <Plain value={workingCopy.remark} /> : <Empty>—</Empty>
                  }
                />
              </Section>
            </>
          )}

          {repository && (
            <>
              <Section title={t('welcome.detail.path')}>
                <Field
                  label={t('welcome.detail.repositoryUrl')}
                  value={<CopyableValue value={repository.repositoryRootUrl} />}
                />
                <Field
                  label={t('welcome.detail.repositoryUuid')}
                  value={<CopyableValue value={repository.repositoryUuid} />}
                />
              </Section>

              <Section title={t('welcome.detail.info')}>
                <Field
                  label={t('welcome.detail.lastUsed')}
                  value={<TimeValue time={repository.lastUsedTime} />}
                />
                <Field
                  label={t('welcome.detail.favourite')}
                  value={
                    repository.star ? (
                      <Plain value={t('welcome.detail.yes')} />
                    ) : (
                      <Empty>{t('welcome.detail.no')}</Empty>
                    )
                  }
                />
                <Field
                  label={t('welcome.detail.remark')}
                  value={repository.remark ? <Plain value={repository.remark} /> : <Empty>—</Empty>}
                />
              </Section>
            </>
          )}
        </div>
      </ScrollArea>
      {workingCopy && (
        <div className={footer}>
          <Button block theme="solid" type="primary" onClick={props.onOpen}>
            {t('welcome.detail.open')}
          </Button>
        </div>
      )}
    </Card>
  )
}
