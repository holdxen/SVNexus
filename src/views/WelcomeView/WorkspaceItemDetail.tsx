import { Typography, Tag, Descriptions, Card, Button } from '@douyinfe/semi-ui'
import { css, cx } from '@linaria/core'

import { ScrollArea } from '@/components/ScrollArea'
import { useDatabase } from '@/context/Database'
import {
  h_full,
  flex,
  flex_col,
  flex_1,
  p_2,
  gap_y_2,
  cursor_default,
  min_h_0,
  flex_row_reverse,
  border_box,
  p_4,
} from '@/styles/Classes'
import { workspaceItemIdentity } from '@/utils/WorkspaceItem'

const detail_descriptions = css`
  & .semi-descriptions-value {
    white-space: normal !important;
    word-break: break-all !important;
  }
`

const { Title, Text } = Typography

export interface WorkspaceItemDetailProps {
  identity: string | null
  onOpen?: () => void
}

export function WorkspaceItemDetail(props: WorkspaceItemDetailProps) {
  const workspaceItems = useDatabase((state) => state.workspaceItems)
  const workspaceItemStates = useDatabase((state) => state.workspaceItemStates)

  const item = workspaceItems.find((e) => workspaceItemIdentity(e) === props.identity)

  if (!item) {
    return (
      <Card shadows="always" className={cx(h_full, flex, cursor_default)}>
        <div
          className={cx(flex, flex_col, p_2, flex_1)}
          style={{ justifyContent: 'center', alignItems: 'center' }}
        >
          <Text type="tertiary">未选中任何项目</Text>
        </div>
      </Card>
    )
  }

  const identity = workspaceItemIdentity(item)
  const state = workspaceItemStates.get(identity)

  const tags: React.ReactNode[] = []

  if (state) {
    if (state.revision) {
      const revision = state.revision
      const text =
        revision.min === revision.max ? `r${revision.min}` : `r${revision.min}:r${revision.max}`
      tags.push(
        <Tag key="revision" type="solid" shape="circle" color="purple">
          {text}
        </Tag>,
      )
    }
    if (state.isInvalid === true) {
      tags.push(
        <Tag key="invalid" type="solid" shape="circle" color="purple">
          Invalid
        </Tag>,
      )
    }
    if (state.isConflicted === true) {
      tags.push(
        <Tag key="conflicted" type="solid" shape="circle" color="violet">
          Conflicted
        </Tag>,
      )
    }
    if (state.isModified === true) {
      tags.push(
        <Tag key="modified" type="solid" shape="circle" color="amber">
          Modified
        </Tag>,
      )
    }
    if (state.isLocked === true) {
      tags.push(
        <Tag key="locked" type="solid" shape="circle" color="red">
          Locked
        </Tag>,
      )
    }
    if (state.isClean === true) {
      tags.push(
        <Tag key="clean" type="solid" shape="circle" color="green">
          Clean
        </Tag>,
      )
    }
  }

  if ('workingCopy' in item) {
    const wc = item.workingCopy
    return (
      <Card
        shadows="always"
        className={cx(h_full, min_h_0, flex, flex_col, cursor_default)}
        bodyStyle={{ minHeight: 0, flex: 1, display: 'flex', flexDirection: 'column' }}
      >
        <ScrollArea className={cx(flex_1, min_h_0)}>
          <div className={cx(flex, flex_col, p_2, gap_y_2)}>
            <div>
              <Title heading={4}>{wc.name}</Title>
              <Tag color="blue" shape="circle">
                工作副本
              </Tag>
            </div>

            {tags.length > 0 && (
              <div className={cx(flex)} style={{ gap: '4px', flexWrap: 'wrap' }}>
                {tags}
              </div>
            )}

            <Descriptions
              className={detail_descriptions}
              data={[
                { key: '根目录', value: <Text size="small">{wc.workingCopyRoot}</Text> },
                { key: '上次打开路径', value: <Text size="small">{wc.workingCopyPath}</Text> },
                { key: '仓库 URL', value: <Text size="small">{wc.repositoryRootUrl ?? '-'}</Text> },
                { key: 'Checkout', value: wc.checkout?.toString() ?? '-' },
                { key: '收藏', value: <Text size="small">{wc.star ? '是' : '否'}</Text> },
                { key: '备注', value: wc.remark ?? '-' },
                {
                  key: '最后使用',
                  value: wc.lastUsedTime ? new Date(wc.lastUsedTime).toLocaleString() : '-',
                },
              ]}
              row
            />
          </div>
        </ScrollArea>
        <div className={cx(flex, flex_row_reverse, border_box, p_4)}>
          <Button theme="solid" type="primary" onClick={props.onOpen}>
            打开
          </Button>
        </div>
      </Card>
    )
  } else {
    const repo = item.repository
    return (
      <Card
        shadows="always"
        className={cx(h_full, min_h_0, flex, flex_col, cursor_default)}
        bodyStyle={{ minHeight: 0, flex: 1, display: 'flex', flexDirection: 'column' }}
      >
        <ScrollArea className={cx(flex_1, min_h_0)}>
          <div className={cx(flex, flex_col, p_2, gap_y_2)}>
            <div>
              <Title heading={4}>{repo.name}</Title>
              <Tag color="green" shape="circle">
                仓库
              </Tag>
            </div>

            {tags.length > 0 && (
              <div className={cx(flex)} style={{ gap: '4px', flexWrap: 'wrap' }}>
                {tags}
              </div>
            )}

            <Descriptions
              className={detail_descriptions}
              data={[
                { key: '仓库 URL', value: <Text size="small">{repo.repositoryRootUrl}</Text> },
                { key: '仓库 UUID', value: <Text size="small">{repo.repositoryUuid}</Text> },
                { key: '收藏', value: <Text size="small">{repo.star ? '是' : '否'}</Text> },
                { key: '备注', value: repo.remark ?? '-' },
                {
                  key: '最后使用',
                  value: repo.lastUsedTime ? new Date(repo.lastUsedTime).toLocaleString() : '-',
                },
              ]}
              row
            />
          </div>
        </ScrollArea>
        <div className={cx(flex, flex_row_reverse, border_box, p_4)}>
          <Button theme="solid" type="primary" onClick={props.onOpen}>
            打开
          </Button>
        </div>
      </Card>
    )
  }
}
